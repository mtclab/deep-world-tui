use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Grass,
    Forest,
    Water,
    Mountain,
    Road,
    Settlement,
    Farmland,
    Sand,
    Swamp,
    Coast,
    Cave,
    Tundra,
    DeepDesert,
}

impl Terrain {
    pub fn glyph(self) -> char {
        match self {
            Terrain::Grass => ',',
            Terrain::Forest => '▓',
            Terrain::Water => '≈',
            Terrain::Mountain => '▲',
            Terrain::Road => '·',
            Terrain::Settlement => '█',
            Terrain::Farmland => '▒',
            Terrain::Sand => '·',
            Terrain::Swamp => '~',
            Terrain::Coast => '≋',
            Terrain::Cave => '◉',
            Terrain::Tundra => '▒',
            Terrain::DeepDesert => ':',
        }
    }

    pub fn passable(self) -> bool {
        !matches!(self, Terrain::Water | Terrain::Mountain)
    }

    pub fn travel_hours(self) -> u32 {
        match self {
            Terrain::Road | Terrain::Settlement => 1,
            Terrain::Grass | Terrain::Farmland | Terrain::Sand | Terrain::Coast => 2,
            Terrain::Forest | Terrain::Swamp | Terrain::Cave | Terrain::Tundra => 3,
            Terrain::DeepDesert => 4,
            Terrain::Water | Terrain::Mountain => 2,
        }
    }

    pub fn people_gather_bonus(people: PeopleKind, terrain: Terrain) -> u32 {
        match (people, terrain) {
            (PeopleKind::Metsik, Terrain::Forest) => 1,
            (PeopleKind::Sepat, Terrain::Mountain) => 1,
            (PeopleKind::Ahjo, Terrain::Grass | Terrain::Farmland) => 1,
            (PeopleKind::Hal, Terrain::Forest) => 1,
            (PeopleKind::Tzakhar, Terrain::Cave) => 1,
            (PeopleKind::Merak, Terrain::Coast) => 1,
            (PeopleKind::Khor, Terrain::Tundra) => 1,
            // Stayed peoples terrain bonuses
            (PeopleKind::Metsareunat, Terrain::Forest) => 1,
            (PeopleKind::Koskimetsa, Terrain::Forest) => 1,
            (PeopleKind::Porokansa, Terrain::Tundra) => 1,
            (PeopleKind::Rantavaki, Terrain::Coast) => 1,
            (PeopleKind::Saarivaki, Terrain::Coast) => 1,
            (PeopleKind::Hiekkakavelijat, Terrain::Coast) => 1,
            (PeopleKind::Haramaki, Terrain::Mountain) => 1,
            (PeopleKind::Pohjavaki, Terrain::Cave) => 1,
            // The Shear walk the dry places — they were the one people with no
            // gather identity (their encounter edge on sand already existed).
            (PeopleKind::Shear, Terrain::Sand | Terrain::DeepDesert) => 1,
            _ => 0,
        }
    }

    pub fn patron_god(self) -> Option<GodName> {
        match self {
            Terrain::Forest => Some(GodName::Keuru),
            Terrain::Grass | Terrain::Farmland | Terrain::Settlement => Some(GodName::Oltzed),
            Terrain::Mountain => Some(GodName::Oltzed),
            Terrain::Road | Terrain::Water => Some(GodName::Masa),
            Terrain::Swamp => Some(GodName::Kukri),
            Terrain::Coast => Some(GodName::Masa),
            Terrain::Cave => Some(GodName::Kukri),
            Terrain::Tundra => Some(GodName::Kukri),
            Terrain::Sand | Terrain::DeepDesert => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PlayerPos {
    pub region_idx: usize,
    pub px: usize,
    pub py: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TerrainMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Terrain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploredMap {
    width: usize,
    height: usize,
    tiles: Vec<bool>,
}

impl ExploredMap {
    pub fn new(width: usize, height: usize) -> Self {
        ExploredMap {
            width,
            height,
            tiles: vec![false; width * height],
        }
    }

    pub fn is_explored(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x]
        } else {
            false
        }
    }

    pub fn reveal(&mut self, cx: usize, cy: usize, radius: usize) {
        let r = radius as isize;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let x = (cx as isize + dx) as usize;
                    let y = (cy as isize + dy) as usize;
                    if x < self.width && y < self.height {
                        self.tiles[y * self.width + x] = true;
                    }
                }
            }
        }
    }

    pub fn reveal_radius_for_elder(elder: bool) -> usize {
        if elder {
            5
        } else {
            3
        }
    }
}

impl TerrainMap {
    pub fn get(&self, x: usize, y: usize) -> Option<Terrain> {
        if x < self.width && y < self.height {
            self.tiles.get(y * self.width + x).copied()
        } else {
            None
        }
    }

    pub fn set(&mut self, x: usize, y: usize, terrain: Terrain) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = terrain;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip<
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug,
    >(
        value: &T,
    ) {
        let ser = ron::ser::to_string(value).unwrap();
        let de: T = ron::from_str(&ser).unwrap();
        assert_eq!(*value, de);
    }

    #[test]

    fn terrain_passability() {
        assert!(!Terrain::Water.passable(), "water must be impassable");

        assert!(!Terrain::Mountain.passable(), "mountain must be impassable");

        assert!(Terrain::Grass.passable(), "grass must be passable");

        assert!(Terrain::Forest.passable(), "forest must be passable");

        assert!(Terrain::Road.passable(), "road must be passable");

        assert!(
            Terrain::Settlement.passable(),
            "settlement must be passable"
        );

        assert!(Terrain::Farmland.passable(), "farmland must be passable");

        assert!(Terrain::Sand.passable(), "sand must be passable");

        assert!(Terrain::Swamp.passable(), "swamp must be passable");

        assert!(Terrain::Coast.passable(), "coast must be passable");

        assert!(Terrain::Cave.passable(), "cave must be passable");

        assert!(Terrain::Tundra.passable(), "tundra must be passable");

        assert!(
            Terrain::DeepDesert.passable(),
            "deep desert must be passable"
        );
    }

    #[test]

    fn terrain_travel_hours() {
        assert_eq!(Terrain::Road.travel_hours(), 1);

        assert_eq!(Terrain::Settlement.travel_hours(), 1);

        assert_eq!(Terrain::Grass.travel_hours(), 2);

        assert_eq!(Terrain::Forest.travel_hours(), 3);

        assert_eq!(Terrain::Swamp.travel_hours(), 3);
    }

    #[test]

    fn player_pos_serialization() {
        let pos = PlayerPos {
            region_idx: 2,

            px: 15,

            py: 7,
        };

        roundtrip(&pos);
    }

    #[test]

    fn explored_map_new_all_unexplored() {
        let map = ExploredMap::new(10, 10);

        assert!(!map.is_explored(5, 5));

        assert!(!map.is_explored(0, 0));
    }

    #[test]

    fn explored_map_reveal_center() {
        let mut map = ExploredMap::new(10, 10);

        map.reveal(5, 5, 2);

        assert!(map.is_explored(5, 5));

        assert!(map.is_explored(5, 4));

        assert!(map.is_explored(5, 6));

        assert!(map.is_explored(4, 5));

        assert!(map.is_explored(6, 5));

        assert!(!map.is_explored(0, 0));
    }

    #[test]

    fn explored_map_reveal_radius_elder() {
        assert_eq!(ExploredMap::reveal_radius_for_elder(false), 3);

        assert_eq!(ExploredMap::reveal_radius_for_elder(true), 5);
    }

    #[test]

    fn explored_map_reveal_clamps_edges() {
        let mut map = ExploredMap::new(10, 10);

        map.reveal(0, 0, 2);

        assert!(map.is_explored(0, 0));

        assert!(map.is_explored(1, 0));

        assert!(map.is_explored(0, 1));
    }

    #[test]

    fn explored_map_out_of_bounds() {
        let map = ExploredMap::new(5, 5);

        assert!(!map.is_explored(10, 10));
    }

    #[test]

    fn people_gather_bonus_no_match() {
        assert_eq!(
            Terrain::people_gather_bonus(PeopleKind::Laakso, Terrain::Forest),
            0
        );

        assert_eq!(
            Terrain::people_gather_bonus(PeopleKind::Metsik, Terrain::Mountain),
            0
        );
    }

    #[test]

    fn people_gather_bonus_metsik_forest() {
        assert_eq!(
            Terrain::people_gather_bonus(PeopleKind::Metsik, Terrain::Forest),
            1
        );
    }

    #[test]

    fn people_gather_bonus_sepat_mountain() {
        assert_eq!(
            Terrain::people_gather_bonus(PeopleKind::Sepat, Terrain::Mountain),
            1
        );
    }

    #[test]

    fn people_gather_bonus_ahjo_farmland() {
        assert_eq!(
            Terrain::people_gather_bonus(PeopleKind::Ahjo, Terrain::Farmland),
            1
        );
    }
}
