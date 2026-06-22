use crate::model::ItemType;
use crate::rng::SeedRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildKind {
    Tarp,
    LeanTo,
    TarpTent,
    Laavu,
    Kota,
    Cabin,
    Longhouse,
    Home,
    /// A god-site, raised as devotional practice — not a summons. The god is
    /// chosen at the raising and kept in Structure::name.
    Shrine,
    /// A walked-in path, laid by one pair of hands. Cuts travel on its tile
    /// while it lasts — and it does not last untended.
    Trail,
    /// Planks over a stream. Water becomes crossable while the bridge stands;
    /// an untended footbridge is a private Velkarmoss.
    Footbridge,
    /// A dug and stone-lined well: water where the land gives none.
    Well,
    /// A stacked cairn that marks the way — the ground around it stays known.
    Waymarker,
    /// A timber palisade: a quieter night inside its line.
    Palisade,
    /// A signal fire on open ground: seen from far, and the lit dark stays
    /// quieter near it while someone keeps it fed.
    Beacon,
}

impl BuildKind {
    pub fn all() -> &'static [BuildKind] {
        &[
            BuildKind::Tarp,
            BuildKind::LeanTo,
            BuildKind::TarpTent,
            BuildKind::Laavu,
            BuildKind::Kota,
            BuildKind::Cabin,
            BuildKind::Longhouse,
            BuildKind::Home,
            BuildKind::Shrine,
            BuildKind::Trail,
            BuildKind::Footbridge,
            BuildKind::Well,
            BuildKind::Waymarker,
            BuildKind::Palisade,
            BuildKind::Beacon,
        ]
    }

    /// The land each kind can stand on. A tarp goes anywhere you can lie
    /// down; a cabin wants real land. None of them rise on water, bare
    /// mountain, a road, or someone else's street (settlement residency is
    /// gated separately).
    pub fn stands_on(self, terrain: crate::model::Terrain) -> bool {
        use crate::model::Terrain as T;
        match self {
            BuildKind::Tarp | BuildKind::LeanTo => {
                !matches!(terrain, T::Water | T::Mountain | T::Road | T::Settlement)
            }
            BuildKind::TarpTent | BuildKind::Laavu => matches!(
                terrain,
                T::Forest | T::Grass | T::Tundra | T::Swamp | T::Coast
            ),
            BuildKind::Kota => matches!(terrain, T::Grass | T::Tundra | T::Farmland),
            BuildKind::Cabin | BuildKind::Longhouse | BuildKind::Home => matches!(
                terrain,
                T::Grass | T::Farmland | T::Forest | T::Coast | T::Settlement
            ),
            // A shrine stands on quiet ground — not in someone's street.
            BuildKind::Shrine => matches!(
                terrain,
                T::Grass | T::Tundra | T::Farmland | T::Forest | T::Coast
            ),
            // A trail is laid on walkable wild ground; roads and streets
            // already are one.
            BuildKind::Trail => terrain.passable() && !matches!(terrain, T::Road | T::Settlement),
            // The only thing that stands on water.
            BuildKind::Footbridge => matches!(terrain, T::Water),
            // A well belongs to the dry lands.
            BuildKind::Well => matches!(
                terrain,
                T::Grass | T::Farmland | T::Sand | T::Tundra | T::DeepDesert
            ),
            BuildKind::Waymarker => {
                terrain.passable() && !matches!(terrain, T::Road | T::Settlement)
            }
            BuildKind::Palisade => matches!(
                terrain,
                T::Grass | T::Farmland | T::Forest | T::Coast | T::Tundra
            ),
            // Open sight-line ground — a fire no forest swallows.
            BuildKind::Beacon => matches!(terrain, T::Mountain | T::Grass | T::Tundra | T::Coast),
        }
    }

    /// Real construction needs a real tool.
    pub fn needs_tool(self) -> bool {
        matches!(
            self,
            BuildKind::Cabin
                | BuildKind::Longhouse
                | BuildKind::Home
                | BuildKind::Footbridge
                | BuildKind::Well
                | BuildKind::Palisade
                | BuildKind::Beacon
        )
    }

    /// Big builds advance only while someone works the site.
    pub fn needs_labor(self) -> bool {
        self.needs_tool()
    }

    /// Parse a kind from the player's word.
    pub fn from_name(name: &str) -> Option<BuildKind> {
        match name.to_ascii_lowercase().as_str() {
            "tarp" => Some(BuildKind::Tarp),
            "leanto" | "lean-to" => Some(BuildKind::LeanTo),
            "tarptent" | "tent" => Some(BuildKind::TarpTent),
            "laavu" => Some(BuildKind::Laavu),
            "kota" => Some(BuildKind::Kota),
            "cabin" => Some(BuildKind::Cabin),
            "longhouse" => Some(BuildKind::Longhouse),
            "home" | "house" => Some(BuildKind::Home),
            "shrine" => Some(BuildKind::Shrine),
            "trail" => Some(BuildKind::Trail),
            "footbridge" | "bridge" => Some(BuildKind::Footbridge),
            "well" => Some(BuildKind::Well),
            "waymarker" | "cairn" => Some(BuildKind::Waymarker),
            "palisade" => Some(BuildKind::Palisade),
            "beacon" | "signalfire" | "signal-fire" => Some(BuildKind::Beacon),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            BuildKind::Tarp => "tarp",
            BuildKind::LeanTo => "lean-to",
            BuildKind::TarpTent => "tarp-tent",
            BuildKind::Laavu => "laavu",
            BuildKind::Kota => "kota",
            BuildKind::Cabin => "cabin",
            BuildKind::Longhouse => "longhouse",
            BuildKind::Home => "home",
            BuildKind::Shrine => "shrine",
            BuildKind::Trail => "trail",
            BuildKind::Footbridge => "footbridge",
            BuildKind::Well => "well",
            BuildKind::Waymarker => "waymarker",
            BuildKind::Palisade => "palisade",
            BuildKind::Beacon => "signal fire",
        }
    }

    /// How much a complete shelter of this kind cuts the cold's bite when the
    /// player is in it (#694): the factor the cold-weather penalty is multiplied
    /// by, after worn gear — lower is warmer. A roof and walls answer the frost
    /// the way a coat does; humbler shelters less, a real house most. `None`
    /// for the things you cannot shelter in (a trail, a well, a beacon, a
    /// shrine, a waymarker, a footbridge, a palisade). The fire/hearth that
    /// warms beyond a roof is a separate, fuel-fed thing (later slice).
    pub fn shelter_warmth(self) -> Option<f64> {
        match self {
            // Open shelters: a wall against the wind, little more.
            BuildKind::Tarp | BuildKind::LeanTo => Some(0.85),
            // Closed tents and frames: a real roof and a banked space.
            BuildKind::TarpTent | BuildKind::Laavu | BuildKind::Kota => Some(0.65),
            // Timber walls and a fixed hearth-space: a winter is survivable here.
            BuildKind::Cabin | BuildKind::Longhouse | BuildKind::Home => Some(0.4),
            BuildKind::Shrine
            | BuildKind::Trail
            | BuildKind::Footbridge
            | BuildKind::Well
            | BuildKind::Waymarker
            | BuildKind::Palisade
            | BuildKind::Beacon => None,
        }
    }

    /// Whether a complete shelter of this kind puts a roof between the player
    /// and the sun (#694): the heat mirror of `shelter_warmth`. Anything you can
    /// shelter in for warmth also shades you in the bake.
    pub fn roofed(self) -> bool {
        self.shelter_warmth().is_some()
    }

    pub fn glyph(self) -> char {
        match self {
            BuildKind::Tarp => '.',
            BuildKind::LeanTo => '.',
            BuildKind::TarpTent => '.',
            BuildKind::Laavu => '.',
            BuildKind::Kota => '.',
            BuildKind::Cabin => '#',
            BuildKind::Longhouse => '▒',
            BuildKind::Home => '▓',
            BuildKind::Shrine => '†',
            BuildKind::Trail => ':',
            BuildKind::Footbridge => '=',
            BuildKind::Well => 'o',
            BuildKind::Waymarker => '^',
            BuildKind::Palisade => '#',
            BuildKind::Beacon => '!',
        }
    }

    pub fn build_hours(self) -> u32 {
        match self {
            BuildKind::Tarp => 0,
            BuildKind::LeanTo => 1,
            BuildKind::TarpTent => 1,
            BuildKind::Laavu => 6,
            BuildKind::Kota => 12,
            BuildKind::Cabin => 72,
            BuildKind::Longhouse => 168,
            BuildKind::Home => 336,
            BuildKind::Shrine => 12,
            BuildKind::Trail => 8,
            BuildKind::Footbridge => 24,
            BuildKind::Well => 16,
            BuildKind::Waymarker => 2,
            BuildKind::Palisade => 48,
            BuildKind::Beacon => 24,
        }
    }

    pub fn rest_quality(self) -> &'static str {
        match self {
            BuildKind::Tarp => "campfire",
            BuildKind::LeanTo => "lean-to",
            BuildKind::TarpTent => "tent",
            BuildKind::Laavu => "laavu",
            BuildKind::Kota => "kota",
            BuildKind::Cabin => "cabin",
            BuildKind::Longhouse => "longhouse",
            BuildKind::Home => "home",
            BuildKind::Shrine => "shrine",
            BuildKind::Trail => "trail",
            BuildKind::Footbridge => "footbridge",
            BuildKind::Well => "well",
            BuildKind::Waymarker => "waymarker",
            BuildKind::Palisade => "palisade",
            BuildKind::Beacon => "beacon",
        }
    }

    pub fn cost(self) -> Vec<(ItemType, u32)> {
        match self {
            BuildKind::Tarp => vec![(ItemType::Cloth, 1)],
            BuildKind::LeanTo => vec![(ItemType::Branches, 4)],
            BuildKind::TarpTent => vec![(ItemType::Cloth, 1), (ItemType::Cordage, 2)],
            BuildKind::Laavu => vec![(ItemType::Branches, 8), (ItemType::Cordage, 2)],
            BuildKind::Kota => vec![
                (ItemType::Branches, 12),
                (ItemType::Stone, 4),
                (ItemType::Tinder, 2),
            ],
            BuildKind::Cabin => vec![
                (ItemType::Wood, 40),
                (ItemType::Nails, 20),
                (ItemType::Stone, 12),
            ],
            BuildKind::Longhouse => vec![
                (ItemType::Wood, 160),
                (ItemType::Nails, 80),
                (ItemType::Stone, 30),
                (ItemType::Thatch, 20),
            ],
            BuildKind::Home => vec![
                (ItemType::Wood, 280),
                (ItemType::Nails, 80),
                (ItemType::Stone, 70),
                (ItemType::Thatch, 20),
                (ItemType::Glass, 3),
            ],
            // Same materials as the NPC kind: stone and cloth, nothing grand.
            BuildKind::Shrine => vec![(ItemType::Stone, 6), (ItemType::Cloth, 3)],
            BuildKind::Trail => vec![(ItemType::Stone, 4), (ItemType::Branches, 4)],
            BuildKind::Footbridge => vec![
                (ItemType::Wood, 24),
                (ItemType::Nails, 12),
                (ItemType::Cordage, 4),
            ],
            BuildKind::Well => vec![(ItemType::Stone, 10), (ItemType::Wood, 6)],
            BuildKind::Waymarker => vec![(ItemType::Stone, 3)],
            BuildKind::Palisade => vec![(ItemType::Wood, 60), (ItemType::Nails, 20)],
            BuildKind::Beacon => vec![(ItemType::Wood, 30), (ItemType::Stone, 8)],
        }
    }

    pub fn is_short_build(self) -> bool {
        self.build_hours() <= 1
    }

    pub fn decay_years(self) -> Option<f64> {
        match self {
            BuildKind::Tarp | BuildKind::LeanTo => None,
            BuildKind::TarpTent => Some(3.0),
            BuildKind::Laavu => Some(3.0),
            BuildKind::Kota => Some(5.0),
            BuildKind::Cabin | BuildKind::Longhouse | BuildKind::Home => Some(30.0),
            BuildKind::Shrine => Some(10.0),
            // Infrastructure decays from day one — that is the whole point.
            // A path grows over in a couple of years; planks rot in five.
            BuildKind::Trail => Some(2.0),
            BuildKind::Footbridge => Some(5.0),
            BuildKind::Well => Some(8.0),
            BuildKind::Waymarker => Some(4.0),
            BuildKind::Palisade => Some(10.0),
            // A fire-stack weathers fast; the frame chars and the stack slumps.
            BuildKind::Beacon => Some(3.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildStatus {
    NotStarted,
    InProgress { hours_done: u32 },
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSite {
    pub kind: BuildKind,
    pub region_idx: usize,
    pub x: u32,
    pub y: u32,
    pub hours_done: u32,
    pub started_tick: u64,
    /// For a Shrine: the god it is raised to, carried into Structure::name
    /// at completion.
    #[serde(default)]
    pub dedication: Option<String>,
}

impl BuildSite {
    pub fn progress_fraction(&self, kind: BuildKind) -> f64 {
        let total = kind.build_hours().max(1) as f64;
        (self.hours_done as f64 / total).min(1.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Structure {
    pub kind: BuildKind,
    pub region_idx: usize,
    pub x: u32,
    pub y: u32,
    pub built_tick: u64,
    pub last_maintenance_tick: u64,
    pub name: Option<String>,
    pub is_npc_built: bool,
    /// What's kept inside (Cabin and better). Stays with the house — heirs
    /// inherit the building and everything in it.
    #[serde(default)]
    pub stash: crate::model::Inventory,
}

impl Structure {
    pub fn at_position(&self, region_idx: usize, x: u32, y: u32) -> bool {
        self.region_idx == region_idx && self.x == x && self.y == y
    }

    pub fn decay_tick(&self, current_tick: u64, ticks_per_day: u64) -> Option<f64> {
        let decay_years = self.kind.decay_years()?;
        // Compressed to the game's aging scale (see STRUCTURE_DAYS_PER_YEAR) so
        // structures actually weather within a playthrough rather than over
        // unreachable real-calendar centuries.
        let ticks_per_year = ticks_per_day as f64 * STRUCTURE_DAYS_PER_YEAR;
        let age_ticks = current_tick.saturating_sub(self.last_maintenance_tick) as f64;
        let age_years = age_ticks / ticks_per_year;
        Some(age_years / decay_years)
    }
}

/// Calendar days per year of a structure's decay life — compressed to match the
/// aging scale so a tarp-tent (3y) weathers in ~a season and a home (30y) lasts
/// roughly a player's lifetime.
pub const STRUCTURE_DAYS_PER_YEAR: f64 = 3.0;

pub fn generate_npc_structures(
    seed: u64,
    region_idx: usize,
    _region_type: &str,
    people_kinds: &[String],
    settlement_count: usize,
    terrain_width: usize,
    terrain_height: usize,
) -> Vec<Structure> {
    let mut rng = SeedRng::new(seed).fork_for(&format!("npc-struct-{region_idx}"));
    let mut structures = Vec::new();

    let has_metsik = people_kinds.iter().any(|p| p == "Metsik");
    let has_vayla = people_kinds.iter().any(|p| p == "Väylä");

    if has_metsik && settlement_count > 0 && terrain_width > 0 && terrain_height > 0 {
        let x = rng.gen_range(terrain_width as u32);
        let y = rng.gen_range(terrain_height as u32);
        structures.push(Structure {
            kind: BuildKind::Kota,
            region_idx,
            x,
            y,
            built_tick: 0,
            last_maintenance_tick: 0,
            name: None,
            is_npc_built: true,
            stash: Default::default(),
        });
    }

    if (has_metsik || has_vayla) && settlement_count > 0 && terrain_width > 0 && terrain_height > 0
    {
        let laavus = if settlement_count > 2 { 2 } else { 1 };
        for _ in 0..laavus {
            let x = rng.gen_range(terrain_width as u32);
            let y = rng.gen_range(terrain_height as u32);
            structures.push(Structure {
                kind: BuildKind::Laavu,
                region_idx,
                x,
                y,
                built_tick: 0,
                last_maintenance_tick: 0,
                name: None,
                is_npc_built: true,
                stash: Default::default(),
            });
        }
    }

    structures
}

pub fn generate_world_structures(seed: u64, world: &mut crate::model::World) {
    for (i, region) in world.regions.iter_mut().enumerate() {
        let people_kinds: Vec<String> = region
            .settlements
            .iter()
            .flat_map(|s| s.people.iter().map(|p| p.people.clone()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let npc_structs = generate_npc_structures(
            seed,
            i,
            &region.region_type,
            &people_kinds,
            region.settlements.len(),
            region.terrain.width,
            region.terrain.height,
        );
        region.structures = npc_structs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_kind_cost_not_empty() {
        for kind in BuildKind::all() {
            assert!(!kind.cost().is_empty(), "{:?} has no cost", kind);
        }
    }

    #[test]
    fn build_kind_short_flag() {
        assert!(BuildKind::Tarp.is_short_build());
        assert!(BuildKind::LeanTo.is_short_build());
        assert!(BuildKind::TarpTent.is_short_build());
        assert!(!BuildKind::Laavu.is_short_build());
        assert!(!BuildKind::Cabin.is_short_build());
    }

    #[test]
    fn build_kind_glyphs_distinct() {
        let _glyphs: Vec<char> = BuildKind::all().iter().map(|k| k.glyph()).collect();
        let home_glyph = BuildKind::Home.glyph();
        let cabin_glyph = BuildKind::Cabin.glyph();
        assert_ne!(home_glyph, cabin_glyph);
    }

    #[test]
    fn generate_npc_structures_deterministic() {
        let s1 = generate_npc_structures(42, 0, "forest", &["Metsik".into()], 2, 40, 30);
        let s2 = generate_npc_structures(42, 0, "forest", &["Metsik".into()], 2, 40, 30);
        assert_eq!(s1.len(), s2.len());
        for (a, b) in s1.iter().zip(s2.iter()) {
            assert_eq!(a.kind, b.kind);
            assert_eq!(a.x, b.x);
            assert_eq!(a.y, b.y);
        }
    }

    #[test]
    fn metsik_region_has_kota() {
        let structs = generate_npc_structures(1, 0, "forest", &["Metsik".into()], 2, 40, 30);
        assert!(structs.iter().any(|s| s.kind == BuildKind::Kota));
    }

    #[test]
    fn no_metsik_no_kota() {
        let structs = generate_npc_structures(1, 0, "grassland", &["Arkit".into()], 2, 40, 30);
        assert!(!structs.iter().any(|s| s.kind == BuildKind::Kota));
    }

    #[test]
    fn decay_tick_none_for_tarp() {
        let s = Structure {
            kind: BuildKind::Tarp,
            region_idx: 0,
            x: 0,
            y: 0,
            built_tick: 0,
            last_maintenance_tick: 0,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        };
        assert!(s.decay_tick(1000, 24).is_none());
    }

    #[test]
    fn decay_tick_increases_over_time() {
        let s = Structure {
            kind: BuildKind::TarpTent,
            region_idx: 0,
            x: 0,
            y: 0,
            built_tick: 0,
            last_maintenance_tick: 0,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        };
        let early = s.decay_tick(1000, 24).unwrap();
        let later = s.decay_tick(100000, 24).unwrap();
        assert!(later > early);
    }

    #[test]
    fn build_site_progress_zero_at_start() {
        let site = BuildSite {
            kind: BuildKind::Cabin,
            region_idx: 0,
            x: 5,
            y: 5,
            hours_done: 0,
            started_tick: 0,
            dedication: None,
        };
        assert!(!site.progress_fraction(BuildKind::Cabin).is_nan());
        assert_eq!(site.progress_fraction(BuildKind::Cabin), 0.0);
    }

    #[test]
    fn build_site_progress_halfway() {
        let site = BuildSite {
            kind: BuildKind::Cabin,
            region_idx: 0,
            x: 5,
            y: 5,
            hours_done: 36,
            started_tick: 0,
            dedication: None,
        };
        let pct = site.progress_fraction(BuildKind::Cabin);
        assert!(pct > 0.4 && pct < 0.6);
    }

    #[test]
    fn tarp_completes_instantly() {
        assert_eq!(BuildKind::Tarp.build_hours(), 0);
        assert!(BuildKind::Tarp.is_short_build());
    }

    #[test]
    fn cabin_takes_long() {
        assert!(!BuildKind::Cabin.is_short_build());
        assert!(BuildKind::Cabin.build_hours() > 1);
    }

    #[test]
    fn structure_at_position_matches() {
        let s = Structure {
            kind: BuildKind::Cabin,
            region_idx: 2,
            x: 10,
            y: 20,
            built_tick: 0,
            last_maintenance_tick: 0,
            name: None,
            is_npc_built: false,
            stash: Default::default(),
        };
        assert!(s.at_position(2, 10, 20));
        assert!(!s.at_position(2, 11, 20));
        assert!(!s.at_position(1, 10, 20));
    }

    #[test]
    fn rest_quality_labels() {
        assert_eq!(BuildKind::Tarp.rest_quality(), "campfire");
        assert_eq!(BuildKind::Cabin.rest_quality(), "cabin");
        assert_eq!(BuildKind::Home.rest_quality(), "home");
    }

    #[test]
    fn beacon_stands_on_open_sight_lines_only() {
        use crate::model::Terrain as T;
        for t in [T::Mountain, T::Grass, T::Tundra, T::Coast] {
            assert!(BuildKind::Beacon.stands_on(t), "{t:?}");
        }
        for t in [
            T::Forest,
            T::Swamp,
            T::Water,
            T::Road,
            T::Settlement,
            T::House,
        ] {
            assert!(!BuildKind::Beacon.stands_on(t), "{t:?}");
        }
    }

    #[test]
    fn beacon_is_real_construction_that_rots_fast() {
        assert!(BuildKind::Beacon.needs_tool());
        assert!(BuildKind::Beacon.needs_labor());
        assert_eq!(BuildKind::Beacon.build_hours(), 24);
        // Infrastructure decays from day one; a fire-stack goes fastest of
        // the watch-works.
        let beacon = BuildKind::Beacon.decay_years().unwrap();
        assert!(beacon < BuildKind::Waymarker.decay_years().unwrap());
        assert_eq!(BuildKind::from_name("beacon"), Some(BuildKind::Beacon));
        assert_eq!(BuildKind::from_name("signalfire"), Some(BuildKind::Beacon));
    }
}
