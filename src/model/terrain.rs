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
    #[serde(alias = "DeepSteppe")]
    Steppe,
    /// A roofed building inside a settlement: someone's house, the tavern,
    /// the temple. Solid to walk through; entered by stepping into the door
    /// (the walk-in interaction layer).
    House,
    /// A building wall — impassable. The border of a real structure (#458).
    Wall,
    /// A building's interior floor — walkable. You walk into and through a
    /// building's rooms, on the one world map (#458).
    Floor,
    /// A doorway in a wall — the passable entry into a building (#458).
    Door,
    /// The hearth at the heart of a building — walkable, and the warmest
    /// place to rest in all the world: a roof, walls, and a fire (#458).
    Hearth,
}

impl Terrain {
    pub fn glyph(self) -> char {
        match self {
            Terrain::Grass => ',',
            Terrain::Forest => '▓',
            Terrain::Water => '≈',
            Terrain::Mountain => '▲',
            Terrain::Road => '·',
            Terrain::Settlement => '·',
            Terrain::House => '⌂',
            Terrain::Farmland => '▒',
            Terrain::Sand => '·',
            Terrain::Swamp => '~',
            Terrain::Coast => '≋',
            Terrain::Cave => '◉',
            Terrain::Tundra => '▒',
            Terrain::Steppe => '"',
            Terrain::Wall => '▒',
            Terrain::Floor => '·',
            Terrain::Door => '+',
            Terrain::Hearth => '*',
        }
    }

    pub fn passable(self) -> bool {
        // Houses and walls are solid: you enter a building by its door (the
        // walk-in layer), you don't walk through walls. Floors and doors are
        // walkable — you move through a building's rooms (#458).
        !matches!(
            self,
            Terrain::Water | Terrain::Mountain | Terrain::House | Terrain::Wall
        )
    }

    pub fn travel_hours(self) -> u32 {
        // Tiles are half the ground they were (80x40 sectors, #372): costs
        // halve so crossing a region takes the same days it always did.
        match self {
            Terrain::Road | Terrain::Settlement => 1,
            Terrain::Grass | Terrain::Farmland | Terrain::Sand | Terrain::Coast => 1,
            Terrain::Forest | Terrain::Swamp | Terrain::Cave | Terrain::Tundra => 2,
            Terrain::Steppe => 2,
            Terrain::Water | Terrain::Mountain | Terrain::House => 1,
            Terrain::Wall | Terrain::Floor | Terrain::Door | Terrain::Hearth => 1,
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
            (PeopleKind::Shear, Terrain::Sand | Terrain::Steppe) => 1,
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
            Terrain::Sand | Terrain::Steppe => None,
            // The hearth-keeper holds the home and all its rooms.
            Terrain::House | Terrain::Wall | Terrain::Floor | Terrain::Door | Terrain::Hearth => {
                Some(GodName::Oltzed)
            }
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

    /// Upscale to a finer grid (each tile becomes an f x f block) — used
    /// when old saves meet the larger sector maps.
    pub fn upscale(&self, f: usize) -> ExploredMap {
        let mut out = ExploredMap::new(self.width * f, self.height * f);
        for y in 0..self.height {
            for x in 0..self.width {
                if self.tiles[y * self.width + x] {
                    for dy in 0..f {
                        for dx in 0..f {
                            out.tiles[(y * f + dy) * out.width + (x * f + dx)] = true;
                        }
                    }
                }
            }
        }
        out
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

        assert!(Terrain::Steppe.passable(), "deep steppe must be passable");
    }

    #[test]

    fn terrain_travel_hours() {
        assert_eq!(Terrain::Road.travel_hours(), 1);

        assert_eq!(Terrain::Settlement.travel_hours(), 1);

        assert_eq!(Terrain::Grass.travel_hours(), 1);

        assert_eq!(Terrain::Forest.travel_hours(), 2);

        assert_eq!(Terrain::Swamp.travel_hours(), 2);
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
