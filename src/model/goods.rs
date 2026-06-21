//! The trade-goods registry (#678): the long tail of the economy's wares.
//!
//! The ~25 core `ItemType` variants are the goods the *code* reasons about —
//! food eaten, tools that wear, hides that tan to leather. But a civilization
//! of millions trades in far more than the code needs special behaviour for:
//! amber and salt and saffron, lamp-oil and parchment and dye-root, a thousand
//! wares that are simply *bought, carried, priced, and sold*. Hard-coding each
//! as an enum variant does not scale and floods every exhaustive `match`.
//!
//! So those live here, in data (`data/goods.ron`), behind a single
//! `ItemType::Good(GoodId)` variant. A `GoodId` is an index into the embedded
//! registry; it serialises as the good's stable **slug** (not the index), so
//! saves survive the registry growing or being reordered, and old saves — which
//! only ever held core variants — parse byte-for-byte as before.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// One trade good, as authored in `data/goods.ron`.
#[derive(Debug, Clone, Deserialize)]
pub struct GoodDef {
    /// Stable identifier — what a save stores. Never reused for another good.
    pub slug: String,
    /// Display name.
    pub name: String,
    /// Base price in the abstract coin, before any market lean.
    pub price: u32,
    /// Free-form tags (e.g. "luxury", "spice", "metal") for grouping, colour,
    /// and future production wiring. Not behaviour-bearing yet.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// The loaded registry: the authored goods plus a slug→index lookup.
pub struct GoodRegistry {
    defs: Vec<GoodDef>,
    by_slug: HashMap<String, u32>,
}

impl GoodRegistry {
    fn parse(ron_src: &str) -> GoodRegistry {
        let defs: Vec<GoodDef> =
            ron::from_str(ron_src).expect("data/goods.ron must parse as Vec<GoodDef>");
        let mut by_slug = HashMap::with_capacity(defs.len());
        for (i, d) in defs.iter().enumerate() {
            if by_slug.insert(d.slug.clone(), i as u32).is_some() {
                panic!("duplicate good slug in data/goods.ron: {}", d.slug);
            }
        }
        GoodRegistry { defs, by_slug }
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }
    pub fn def(&self, id: GoodId) -> Option<&GoodDef> {
        self.defs.get(id.0 as usize)
    }
    pub fn id_of(&self, slug: &str) -> Option<GoodId> {
        self.by_slug.get(slug).copied().map(GoodId)
    }
    /// All goods, in registry order — for menus and tests.
    pub fn all(&self) -> impl Iterator<Item = (GoodId, &GoodDef)> {
        self.defs
            .iter()
            .enumerate()
            .map(|(i, d)| (GoodId(i as u32), d))
    }
}

const GOODS_RON: &str = include_str!("../../data/goods.ron");

/// The process-wide registry, parsed once from the embedded data on first use.
pub fn goods() -> &'static GoodRegistry {
    static REG: OnceLock<GoodRegistry> = OnceLock::new();
    REG.get_or_init(|| GoodRegistry::parse(GOODS_RON))
}

/// A handle to a trade good. Internally an index; on the wire a stable slug.
///
/// `u32::MAX` is the reserved "unknown good" sentinel — produced when a save
/// names a slug no longer in the registry, so loading never fails outright.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GoodId(u32);

const UNKNOWN: u32 = u32::MAX;

impl GoodId {
    pub fn slug(self) -> &'static str {
        goods()
            .def(self)
            .map(|d| d.name_slug())
            .unwrap_or("unknown-good")
    }
    pub fn name(self) -> &'static str {
        goods().def(self).map(|d| d.name.as_str()).unwrap_or("?")
    }
    pub fn price(self) -> u32 {
        goods().def(self).map(|d| d.price).unwrap_or(1)
    }
    pub fn has_tag(self, tag: &str) -> bool {
        goods()
            .def(self)
            .map(|d| d.tags.iter().any(|t| t == tag))
            .unwrap_or(false)
    }
}

impl GoodDef {
    fn name_slug(&self) -> &str {
        &self.slug
    }
}

/// Look a good up by slug — the public way to mint an `ItemType::Good`.
pub fn good_id(slug: &str) -> Option<GoodId> {
    goods().id_of(slug)
}

impl Serialize for GoodId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // Always the stable slug, never the index.
        let slug = goods().def(*self).map(|d| d.slug.as_str());
        match slug {
            Some(slug) => s.serialize_str(slug),
            None => s.serialize_str("unknown-good"),
        }
    }
}

impl<'de> Deserialize<'de> for GoodId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let slug = String::deserialize(d)?;
        match goods().id_of(&slug) {
            Some(id) => Ok(id),
            None if slug == "unknown-good" => Ok(GoodId(UNKNOWN)),
            // A slug that has left the registry: keep the save loadable rather
            // than hard-failing on one retired good.
            None => Ok(GoodId(UNKNOWN)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads_and_is_substantial() {
        assert!(
            goods().len() >= 40,
            "the goods registry should carry a real shelf, got {}",
            goods().len()
        );
    }

    #[test]
    fn slugs_are_unique_and_resolvable() {
        for (id, d) in goods().all() {
            assert_eq!(good_id(&d.slug), Some(id), "slug {} round-trips", d.slug);
            assert!(!d.name.is_empty());
            assert!(d.price >= 1, "{} needs a price", d.slug);
        }
    }

    #[test]
    fn good_id_serialises_as_slug_and_round_trips() {
        let id = good_id("salt").expect("salt is in the starter registry");
        let ron = ron::ser::to_string(&id).unwrap();
        assert_eq!(ron, "\"salt\"", "a GoodId serialises as its slug");
        let back: GoodId = ron::from_str(&ron).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn unknown_slug_loads_as_sentinel_not_error() {
        let back: GoodId = ron::from_str("\"a-good-that-was-retired\"").unwrap();
        assert_eq!(back, GoodId(UNKNOWN));
        assert_eq!(back.name(), "?");
    }
}
