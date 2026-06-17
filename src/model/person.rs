use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GodName {
    Oltzed,
    Keuru,
    Sampsa,
    Masa,
    Kukri,
}

impl GodName {
    pub fn label(self) -> &'static str {
        match self {
            GodName::Oltzed => "Oltzed",
            GodName::Keuru => "Keuru",
            GodName::Sampsa => "Sampsa",
            GodName::Masa => "Masa",
            GodName::Kukri => "Kukri",
        }
    }

    pub fn from_label(s: &str) -> Option<GodName> {
        match s.to_ascii_lowercase().as_str() {
            "oltzed" => Some(GodName::Oltzed),
            "keuru" => Some(GodName::Keuru),
            "sampsa" => Some(GodName::Sampsa),
            "masa" => Some(GodName::Masa),
            "kukri" => Some(GodName::Kukri),
            _ => None,
        }
    }

    pub fn domains(self) -> &'static str {
        match self {
            GodName::Oltzed => "labor, invention, engineering",
            GodName::Keuru => "forests, hospitality, celebration",
            GodName::Sampsa => "knowledge, memory, archives",
            GodName::Masa => "trade, perseverance, common people",
            GodName::Kukri => "solitude, old wisdom, nostalgia",
        }
    }

    pub fn glyph(self) -> char {
        match self {
            GodName::Oltzed => '⚒',
            GodName::Keuru => '🌲',
            GodName::Sampsa => '📖',
            GodName::Masa => '⚖',
            GodName::Kukri => '🕯',
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GodAffinity {
    #[serde(default)]
    pub oltzed: f64,
    #[serde(default)]
    pub keuru: f64,
    #[serde(default)]
    pub sampsa: f64,
    #[serde(default)]
    pub masa: f64,
    #[serde(default)]
    pub kukri: f64,
}

impl Default for GodAffinity {
    fn default() -> Self {
        GodAffinity {
            oltzed: 0.0,
            keuru: 0.0,
            sampsa: 0.0,
            masa: 0.0,
            kukri: 0.0,
        }
    }
}

impl GodAffinity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, god: GodName) -> f64 {
        match god {
            GodName::Oltzed => self.oltzed,
            GodName::Keuru => self.keuru,
            GodName::Sampsa => self.sampsa,
            GodName::Masa => self.masa,
            GodName::Kukri => self.kukri,
        }
    }

    pub fn adjust(&mut self, god: GodName, delta: f64) {
        let val = match god {
            GodName::Oltzed => &mut self.oltzed,
            GodName::Keuru => &mut self.keuru,
            GodName::Sampsa => &mut self.sampsa,
            GodName::Masa => &mut self.masa,
            GodName::Kukri => &mut self.kukri,
        };
        *val = (*val + delta).clamp(-1.0, 1.0);
    }

    pub fn strongest_ally(&self) -> Option<GodName> {
        let gods = [
            (GodName::Oltzed, self.oltzed),
            (GodName::Keuru, self.keuru),
            (GodName::Sampsa, self.sampsa),
            (GodName::Masa, self.masa),
            (GodName::Kukri, self.kukri),
        ];
        let best = gods
            .iter()
            .filter(|(_, v)| *v > 0.0)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        best.map(|(g, _)| *g)
    }

    pub fn strongest_grudge(&self) -> Option<GodName> {
        let gods = [
            (GodName::Oltzed, self.oltzed),
            (GodName::Keuru, self.keuru),
            (GodName::Sampsa, self.sampsa),
            (GodName::Masa, self.masa),
            (GodName::Kukri, self.kukri),
        ];
        let worst = gods
            .iter()
            .filter(|(_, v)| *v < 0.0)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        worst.map(|(g, _)| *g)
    }

    pub fn people_title(self, people: PeopleKind) -> &'static str {
        let affinity = self.get(people.patron_god().unwrap_or(GodName::Oltzed));
        if affinity > 0.6 {
            match people {
                PeopleKind::Metsik => "Friend of the Forest",
                PeopleKind::Ahjo => "Hearth-kin",
                PeopleKind::Sepat => "Iron-Bound",
                PeopleKind::Vayla => "River-Walker",
                PeopleKind::Arkit => "Archive-Shadow",
                PeopleKind::Laakso => "Deep-Patient",
                PeopleKind::Varhaiset => "Ancient-Known",
                PeopleKind::Metsareunat => "Edge-Friend",
                PeopleKind::Porokansa => "Herd-Brother",
                PeopleKind::Koskimetsa => "Rapids-Kin",
                PeopleKind::Muistikansa => "Song-Keeper",
                PeopleKind::Taulukansa => "Tablet-Friend",
                PeopleKind::Kirjakansa => "Book-Brother",
                PeopleKind::Takovaki => "Copper-Bound",
                PeopleKind::Rantavaki => "Shore-Kin",
                PeopleKind::Saarivaki => "Island-Brother",
                PeopleKind::Hiekkakavelijat => "Sand-Kin",
                PeopleKind::Haramaki => "Terrace-Friend",
                PeopleKind::Jamavaki => "Hidden-Known",
                PeopleKind::Pohjavaki => "Deep-Root",
                PeopleKind::Tzakhar => "Cave-Dweller",
                PeopleKind::Merak => "Wave-Rider",
                PeopleKind::Shear => "Sand-Walker",
                PeopleKind::Hal => "Canopy-Friend",
                PeopleKind::Khor => "Frost-Enduring",
            }
        } else if affinity > 0.3 {
            match people {
                PeopleKind::Metsik => "Wood-Familiar",
                PeopleKind::Ahjo => "Settlement-Guest",
                PeopleKind::Sepat => "Forge-Acquainted",
                PeopleKind::Vayla => "Trade-Known",
                PeopleKind::Arkit => "Page-Turner",
                PeopleKind::Laakso => "Steady-Presence",
                PeopleKind::Varhaiset => "Old-Acquaintance",
                PeopleKind::Metsareunat => "Margin-Known",
                PeopleKind::Porokansa => "Herd-Aware",
                PeopleKind::Koskimetsa => "River-Aware",
                PeopleKind::Muistikansa => "Song-Heard",
                PeopleKind::Taulukansa => "Tablet-Aware",
                PeopleKind::Kirjakansa => "Book-Aware",
                PeopleKind::Takovaki => "Copper-Aware",
                PeopleKind::Rantavaki => "Shore-Acquainted",
                PeopleKind::Saarivaki => "Island-Acquainted",
                PeopleKind::Hiekkakavelijat => "Sand-Aware",
                PeopleKind::Haramaki => "Terrace-Acquainted",
                PeopleKind::Jamavaki => "Valley-Aware",
                PeopleKind::Pohjavaki => "Depth-Aware",
                PeopleKind::Tzakhar => "Depth-Curious",
                PeopleKind::Merak => "Shore-Acquainted",
                PeopleKind::Shear => "Heat-Tolerant",
                PeopleKind::Hal => "Branch-Greeter",
                PeopleKind::Khor => "Cold-Respected",
            }
        } else if affinity < -0.2 {
            match people {
                PeopleKind::Metsik => "Tree-Feller",
                PeopleKind::Ahjo => "Hearth-Stranger",
                PeopleKind::Sepat => "Ore-Taker",
                PeopleKind::Vayla => "Current-Breaker",
                PeopleKind::Arkit => "Page-Burner",
                PeopleKind::Laakso => "Impatient-One",
                PeopleKind::Varhaiset => "Ancient-Shadow",
                PeopleKind::Metsareunat => "Edge-Usurper",
                PeopleKind::Porokansa => "Herd-Thief",
                PeopleKind::Koskimetsa => "Rapids-Blocker",
                PeopleKind::Muistikansa => "Song-Silencer",
                PeopleKind::Taulukansa => "Tablet-Smasher",
                PeopleKind::Kirjakansa => "Book-Burner",
                PeopleKind::Takovaki => "Copper-Thief",
                PeopleKind::Rantavaki => "Shore-Taker",
                PeopleKind::Saarivaki => "Island-Invader",
                PeopleKind::Hiekkakavelijat => "Sand-Grabber",
                PeopleKind::Haramaki => "Terrace-Seizer",
                PeopleKind::Jamavaki => "Valley-Defiler",
                PeopleKind::Pohjavaki => "Depth-Robber",
                PeopleKind::Tzakhar => "Surface-Clasher",
                PeopleKind::Merak => "Land-Anchor",
                PeopleKind::Shear => "Oasis-Hoarder",
                PeopleKind::Hal => "Canopy-Skimmer",
                PeopleKind::Khor => "Warmth-Seeker",
            }
        } else {
            ""
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PeopleKind {
    #[default]
    Metsik,
    Arkit,
    Vayla,
    Laakso,
    Sepat,
    Ahjo,
    Varhaiset,
    Metsareunat,
    Porokansa,
    Koskimetsa,
    Muistikansa,
    Taulukansa,
    Kirjakansa,
    Takovaki,
    Rantavaki,
    Saarivaki,
    Hiekkakavelijat,
    Haramaki,
    Jamavaki,
    Pohjavaki,
    Tzakhar,
    Merak,
    Shear,
    Hal,
    Khor,
}

impl PeopleKind {
    pub fn glyph(self) -> char {
        match self {
            PeopleKind::Metsik => 'δ',
            PeopleKind::Arkit => 'α',
            PeopleKind::Vayla => 'ν',
            PeopleKind::Laakso => 'λ',
            PeopleKind::Sepat => 'σ',
            PeopleKind::Ahjo => 'ω',
            PeopleKind::Varhaiset => 'ε',
            PeopleKind::Metsareunat => 'δ',
            PeopleKind::Porokansa => 'π',
            PeopleKind::Koskimetsa => 'κ',
            PeopleKind::Muistikansa => 'μ',
            PeopleKind::Taulukansa => 'τ',
            PeopleKind::Kirjakansa => 'χ',
            PeopleKind::Takovaki => 'φ',
            PeopleKind::Rantavaki => 'ρ',
            PeopleKind::Saarivaki => 'ι',
            PeopleKind::Hiekkakavelijat => 'η',
            PeopleKind::Haramaki => 'θ',
            PeopleKind::Jamavaki => 'ψ',
            PeopleKind::Pohjavaki => 'ξ',
            PeopleKind::Tzakhar => 'ζ',
            PeopleKind::Merak => 'β',
            PeopleKind::Shear => 'γ',
            PeopleKind::Hal => 'δ',
            PeopleKind::Khor => 'ξ',
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "metsik" | "čyrvä" | "keurimä" => PeopleKind::Metsik,
            "arkit" | "märät" | "sampsari" => PeopleKind::Arkit,
            "vayla" | "väylä" | "vylti" | "masari" => PeopleKind::Vayla,
            "laakso" | "kiškam" | "kukreva" => PeopleKind::Laakso,
            "sepat" | "sepät" | "wosyt" => PeopleKind::Sepat,
            "ahjo" | "njumka" | "iltkari" => PeopleKind::Ahjo,
            "varhaiset" | "körvä" | "perikansan" => PeopleKind::Varhaiset,
            "metsareunat" | "metsäreunat" | "pyršä" | "keurunreunat" => PeopleKind::Metsareunat,
            "porokansa" | "tuorva" | "keuruporo" => PeopleKind::Porokansa,
            "koskimetsa" | "koskimetsä" | "jälky" | "keurukoski" => PeopleKind::Koskimetsa,
            "muistikansa" | "särät" | "sampsamuisti" => PeopleKind::Muistikansa,
            "taulukansa" | "velmät" | "sampsataulu" => PeopleKind::Taulukansa,
            "kirjakansa" | "tärent" | "sampsakirja" => PeopleKind::Kirjakansa,
            "takovaki" | "takoväki" | "wonśyt" | "oltkartako" => PeopleKind::Takovaki,
            "rantavaki" | "rantaväki" | "vylri" | "masaranta" => PeopleKind::Rantavaki,
            "saarivaki" | "saariväki" | "kylmi" | "masasaari" => PeopleKind::Saarivaki,
            "hiekkakavelijat" | "hiekkakävelijät" | "tyrväi" | "masahiekka" => {
                PeopleKind::Hiekkakavelijat
            }
            "haramaki" | "härämäki" | "kišmäs" | "kukriharma" => PeopleKind::Haramaki,
            "jamavaki" | "jämäväki" | "hoskam" | "kukrijämä" => PeopleKind::Jamavaki,
            "pohjavaki" | "pohjaväki" | "väškam" | "kukripohja" => PeopleKind::Pohjavaki,
            "tzakhar" | "tzäkhar" | "vaskiluuri" => PeopleKind::Tzakhar,
            "merak" | "mëräk" | "iltäkälä" => PeopleKind::Merak,
            "shear" | "she'ar" | "muraskala" => PeopleKind::Shear,
            "hal" | "häl" => PeopleKind::Hal,
            "khor" | "khör" | "khmört" => PeopleKind::Khor,
            _ => PeopleKind::Metsik,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Forest-people",
            PeopleKind::Arkit => "Archive-keepers",
            PeopleKind::Vayla => "River-folk",
            PeopleKind::Laakso => "Vale-dwellers",
            PeopleKind::Sepat => "Iron-people",
            PeopleKind::Ahjo => "Sacred-forge folk",
            PeopleKind::Varhaiset => "The Earliest Ones — highland headwaters folk",
            PeopleKind::Metsareunat => "Forest-edge people",
            PeopleKind::Porokansa => "Reindeer-following people",
            PeopleKind::Koskimetsa => "River-rapids forest people",
            PeopleKind::Muistikansa => "Memory-keepers — oral tradition folk",
            PeopleKind::Taulukansa => "Tablet-people — independent scribes",
            PeopleKind::Kirjakansa => "Book-people — river-valley scribes",
            PeopleKind::Takovaki => "Surface-copper forge folk",
            PeopleKind::Rantavaki => "Shore-folk — tidepool cultivators",
            PeopleKind::Saarivaki => "Island-folk — inland sea traders",
            PeopleKind::Hiekkakavelijat => "Sand-walkers — beach-ridge lagoon folk",
            PeopleKind::Haramaki => "Steep-valley people — dry-terrace farmers",
            PeopleKind::Jamavaki => "Hidden-valley people — hermit traditions",
            PeopleKind::Pohjavaki => "Deep-bottom people — depth-silence keepers",
            PeopleKind::Tzakhar => "Deep-cave people",
            PeopleKind::Merak => "Sea-people",
            PeopleKind::Shear => "Desert people",
            PeopleKind::Hal => "Canopy people",
            PeopleKind::Khor => "Steppe people",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Metsik",
            PeopleKind::Arkit => "Arkit",
            PeopleKind::Vayla => "Väylä",
            PeopleKind::Laakso => "Laakso",
            PeopleKind::Sepat => "Sepät",
            PeopleKind::Ahjo => "Ahjo",
            PeopleKind::Varhaiset => "Varhaiset",
            PeopleKind::Metsareunat => "Metsäreunat",
            PeopleKind::Porokansa => "Porokansa",
            PeopleKind::Koskimetsa => "Koskimetsä",
            PeopleKind::Muistikansa => "Muistikansa",
            PeopleKind::Taulukansa => "Taulukansa",
            PeopleKind::Kirjakansa => "Kirjakansa",
            PeopleKind::Takovaki => "Takoväki",
            PeopleKind::Rantavaki => "Rantaväki",
            PeopleKind::Saarivaki => "Saariväki",
            PeopleKind::Hiekkakavelijat => "Hiekkakävelijät",
            PeopleKind::Haramaki => "Härämäki",
            PeopleKind::Jamavaki => "Jämäväki",
            PeopleKind::Pohjavaki => "Pohjaväki",
            PeopleKind::Tzakhar => "Tzäkhar",
            PeopleKind::Merak => "Mëräk",
            PeopleKind::Shear => "She'ar",
            PeopleKind::Hal => "Häl",
            PeopleKind::Khor => "Khör",
        }
    }

    pub fn patron_god(self) -> Option<GodName> {
        match self {
            PeopleKind::Metsik => Some(GodName::Keuru),
            PeopleKind::Ahjo | PeopleKind::Sepat => Some(GodName::Oltzed),
            PeopleKind::Vayla => Some(GodName::Masa),
            PeopleKind::Arkit => Some(GodName::Sampsa),
            PeopleKind::Laakso => Some(GodName::Kukri),
            // Keurish family: Keuru
            PeopleKind::Varhaiset => None, // All Five (travelers saw all gods pass through)
            PeopleKind::Metsareunat => Some(GodName::Keuru),
            PeopleKind::Porokansa => Some(GodName::Keuru),
            PeopleKind::Koskimetsa => Some(GodName::Keuru),
            // Sampsaran family: Sampsa
            PeopleKind::Muistikansa => Some(GodName::Sampsa),
            PeopleKind::Taulukansa => Some(GodName::Sampsa),
            PeopleKind::Kirjakansa => Some(GodName::Sampsa),
            // Oltkar family: Oltzed
            PeopleKind::Takovaki => Some(GodName::Oltzed),
            // Masaran family: Masa
            PeopleKind::Rantavaki => Some(GodName::Masa),
            PeopleKind::Saarivaki => Some(GodName::Masa),
            PeopleKind::Hiekkakavelijat => Some(GodName::Masa),
            // Kukresh family: Kukri
            PeopleKind::Haramaki => Some(GodName::Kukri),
            PeopleKind::Jamavaki => Some(GodName::Kukri),
            PeopleKind::Pohjavaki => Some(GodName::Kukri),
            // Non-human — first encounters per canon
            // (races/00_sapient_races_overview.md "Relationship with the Five
            // Gods"): Tzäkhar met Kukri underground; Mëräk met Keuru and Masa
            // at the tide-line (Masa as patron of their trade-facing life);
            // She'ar hear Kukri in the desert's silence; Häl met Keuru in the
            // canopy; Khör's primary divine relationship is Kukri,
            // Talven-Hiljaisuus, "Winter's-Silence".
            PeopleKind::Tzakhar => Some(GodName::Kukri),
            PeopleKind::Merak => Some(GodName::Masa),
            PeopleKind::Shear => Some(GodName::Kukri),
            PeopleKind::Hal => Some(GodName::Keuru),
            PeopleKind::Khor => Some(GodName::Kukri),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Metsik",
            PeopleKind::Arkit => "Arkit",
            PeopleKind::Vayla => "Väylä",
            PeopleKind::Laakso => "Laakso",
            PeopleKind::Sepat => "Sepät",
            PeopleKind::Ahjo => "Ahjo",
            PeopleKind::Varhaiset => "Körvä",
            PeopleKind::Metsareunat => "Pyršä",
            PeopleKind::Porokansa => "Tuorva",
            PeopleKind::Koskimetsa => "Jälky",
            PeopleKind::Muistikansa => "Särät",
            PeopleKind::Taulukansa => "Velmät",
            PeopleKind::Kirjakansa => "Tärent",
            PeopleKind::Takovaki => "Wonśyt",
            PeopleKind::Rantavaki => "Vylri",
            PeopleKind::Saarivaki => "Kylmi",
            PeopleKind::Hiekkakavelijat => "Tyrväi",
            PeopleKind::Haramaki => "Kišmäs",
            PeopleKind::Jamavaki => "Hoskam",
            PeopleKind::Pohjavaki => "Väškam",
            PeopleKind::Tzakhar => "Tzäkhar",
            PeopleKind::Merak => "Mëräk",
            PeopleKind::Shear => "She'ar",
            PeopleKind::Hal => "Häl",
            PeopleKind::Khor => "Khör",
        }
    }

    /// The canon Five non-human peoples — the deep-smiths, the tideline
    /// fishers, the dry-country walkers, the canopy-dwellers, the steppe
    /// herders. They keep to their own ground and barter in kind, never coin;
    /// a settlement of theirs is an enclave, not a town of the human regions.
    pub fn is_of_the_five(self) -> bool {
        matches!(
            self,
            PeopleKind::Tzakhar
                | PeopleKind::Merak
                | PeopleKind::Shear
                | PeopleKind::Hal
                | PeopleKind::Khor
        )
    }

    /// How one of the Five receives a stranger who walks into their enclave
    /// (#454) — a line true to each people's monograph. `None` for the human
    /// regions, whose towns are no enclave.
    pub fn enclave_welcome(self) -> Option<&'static str> {
        Some(match self {
            PeopleKind::Tzakhar => "You step into the deephold of the Tzäkhar — lamplit stone, the ring of the deep-forge below. They take no coin; lay down a good and they will weigh it.",
            PeopleKind::Merak => "You come to the Mëräk at the tideline — racks of drying fish, the smell of deep-water and salt. They trade in kind, the sea's measure for the land's.",
            PeopleKind::Shear => "You reach a waterhold of the She'ar in the dry country — shade-cloth, still air, the discipline of water counted to the drop. They speak little and barter true.",
            PeopleKind::Hal => "You enter the canopy-town of the Häl — floors woven in the high green, Keuru's wood about you. They keep the physic-shrine and trade their salves in kind.",
            PeopleKind::Khor => "You arrive at a cairn-circle of the Khör on the cold steppe — herder-houses, härkä-hide, the word-keepers' shrine. They take no coin, only an even trade.",
            _ => return None,
        })
    }

    /// The deeper lore a traveller learns the **first** time they enter one of
    /// the Five's enclaves (#454) — a thing about the people worth the journal,
    /// told once. `None` for the human regions.
    pub fn enclave_lore(self) -> Option<&'static str> {
        Some(match self {
            PeopleKind::Tzakhar => "The Tzäkhar do not measure a life in years but in the worth of what its hands have shaped. Their deep-forges have burned, they say, since before the Fall darkened the surface — and they kept the old iron-songs the upworld forgot.",
            PeopleKind::Merak => "The Mëräk reckon kinship by the tides they were born under, not by blood. They hold that the deep water remembers every drowned name, and on the turning of the year they speak those names to the sea so it does not forget.",
            PeopleKind::Shear => "The She'ar keep the discipline of silence and of water: to waste a word or a drop is the same failing. Their elders are those who have crossed the deep desert alone and returned — the heat-silence is their teacher and their faith.",
            PeopleKind::Hal => "The Häl never set foot to the forest floor if they can help it; their canopy-towns are woven living, and a hall grows for a generation before it is dwelt in. Keuru's green is their god and their roof both, and they physic the hurt of any who come in peace.",
            PeopleKind::Khor => "The Khör carry their dead as cairn-stones, one stone per life, and their circles are libraries of the gone. A word-keeper can recite a lineage back two hundred winters. They endure the cold the upworld cannot, and ask only an even trade for the warmth they make.",
            _ => return None,
        })
    }

    /// How one of the Five keeps its patron god of the Five (#457 item 5) —
    /// the canon first-encounter, surfaced in play the first time you enter
    /// their enclave. Canon: races/00_sapient_races_overview.md, "Relationship
    /// with the Five Gods". `None` for the human regions and the godless.
    pub fn enclave_god_relation(self) -> Option<&'static str> {
        Some(match self {
            PeopleKind::Tzakhar => "The Tzäkhar keep Kukri, whom they met underground in the long dark — the solitary god of old wisdom and the patient stone. They name him in the deep-forge's first light.",
            PeopleKind::Merak => "The Mëräk keep Masa of the even trade, met at the tide-line where land and sea bargain; it is Masa they thank for a fair measure and a kept word.",
            PeopleKind::Shear => "The She'ar hear Kukri in the desert's silence — the god of solitude speaks loudest where there is least to drown him, and the heat-silence is their prayer.",
            PeopleKind::Hal => "The Häl keep Keuru, met in the canopy's green; the god of forests and hospitality is their roof and their welcome both, and they physic any who come in peace in his name.",
            PeopleKind::Khor => "The Khör keep Kukri under the name Talven-Hiljaisuus, Winter's-Silence — the god of endurance and the patient cold, who keeps the lineages the cairn-stones remember.",
            _ => return None,
        })
    }

    pub fn bias_toward(self, other: PeopleKind) -> f64 {
        if self == other {
            return 0.15;
        }
        match (self, other) {
            // Human-human biases (preserved from original)
            (PeopleKind::Metsik, PeopleKind::Sepat) => -0.20,
            (PeopleKind::Metsik, PeopleKind::Ahjo) => -0.15,
            (PeopleKind::Sepat, PeopleKind::Metsik) => -0.15,
            (PeopleKind::Ahjo, PeopleKind::Metsik) => -0.12,
            (PeopleKind::Sepat, PeopleKind::Ahjo) => 0.10,
            (PeopleKind::Ahjo, PeopleKind::Sepat) => 0.10,
            (PeopleKind::Metsik, PeopleKind::Vayla) => -0.05,
            (PeopleKind::Vayla, PeopleKind::Metsik) => -0.05,
            (PeopleKind::Sepat, PeopleKind::Arkit) => 0.08,
            (PeopleKind::Arkit, PeopleKind::Sepat) => 0.05,
            (PeopleKind::Ahjo, PeopleKind::Vayla) => 0.05,
            (PeopleKind::Vayla, PeopleKind::Ahjo) => 0.05,
            (PeopleKind::Laakso, PeopleKind::Metsik) => 0.05,
            (PeopleKind::Metsik, PeopleKind::Laakso) => 0.05,
            (PeopleKind::Laakso, PeopleKind::Vayla) => -0.08,
            (PeopleKind::Vayla, PeopleKind::Laakso) => -0.05,
            // Non-human to human: generally neutral-wary
            (PeopleKind::Tzakhar, PeopleKind::Metsik) => 0.02,
            (PeopleKind::Tzakhar, PeopleKind::Laakso) => 0.05,
            (PeopleKind::Tzakhar, _) => -0.03,
            (PeopleKind::Merak, PeopleKind::Vayla) => 0.08,
            (PeopleKind::Merak, PeopleKind::Ahjo) => 0.03,
            (PeopleKind::Merak, _) => -0.02,
            (PeopleKind::Shear, _) => -0.05,
            (PeopleKind::Hal, PeopleKind::Metsik) => 0.08,
            (PeopleKind::Hal, _) => -0.02,
            (PeopleKind::Khor, PeopleKind::Arkit) => 0.03,
            (PeopleKind::Khor, _) => -0.04,
            // Human to non-human: generally curious but distant
            (PeopleKind::Metsik, PeopleKind::Hal) => 0.05,
            (PeopleKind::Vayla, PeopleKind::Merak) => 0.06,
            (PeopleKind::Laakso, PeopleKind::Tzakhar) => 0.04,
            (PeopleKind::Arkit, PeopleKind::Khor) => 0.02,
            (PeopleKind::Sepat, PeopleKind::Tzakhar) => -0.03,
            // Stayed peoples to SAST peoples
            (PeopleKind::Varhaiset, PeopleKind::Metsik) => 0.05,
            (PeopleKind::Varhaiset, PeopleKind::Arkit) => 0.02,
            (PeopleKind::Metsareunat, PeopleKind::Metsik) => 0.08,
            (PeopleKind::Metsareunat, PeopleKind::Arkit) => 0.02,
            (PeopleKind::Porokansa, PeopleKind::Metsik) => 0.06,
            (PeopleKind::Koskimetsa, PeopleKind::Metsik) => 0.07,
            (PeopleKind::Koskimetsa, PeopleKind::Vayla) => 0.04,
            (PeopleKind::Muistikansa, PeopleKind::Arkit) => 0.08,
            (PeopleKind::Taulukansa, PeopleKind::Arkit) => 0.06,
            (PeopleKind::Kirjakansa, PeopleKind::Arkit) => 0.05,
            (PeopleKind::Takovaki, PeopleKind::Sepat) => 0.06,
            (PeopleKind::Takovaki, PeopleKind::Ahjo) => 0.04,
            (PeopleKind::Rantavaki, PeopleKind::Vayla) => 0.07,
            (PeopleKind::Saarivaki, PeopleKind::Vayla) => 0.09,
            (PeopleKind::Saarivaki, PeopleKind::Merak) => 0.05,
            (PeopleKind::Hiekkakavelijat, PeopleKind::Vayla) => 0.06,
            (PeopleKind::Haramaki, PeopleKind::Laakso) => 0.08,
            (PeopleKind::Jamavaki, PeopleKind::Laakso) => 0.09,
            (PeopleKind::Pohjavaki, PeopleKind::Laakso) => 0.10,
            // SAST peoples to stayed peoples (reciprocal)
            (PeopleKind::Metsik, PeopleKind::Koskimetsa) => 0.06,
            (PeopleKind::Metsik, PeopleKind::Metsareunat) => 0.05,
            (PeopleKind::Arkit, PeopleKind::Muistikansa) => 0.07,
            (PeopleKind::Vayla, PeopleKind::Saarivaki) => 0.08,
            (PeopleKind::Laakso, PeopleKind::Haramaki) => 0.07,
            // Stayed to stayed (same family = positive)
            (PeopleKind::Koskimetsa, PeopleKind::Varhaiset) => 0.04,
            (PeopleKind::Koskimetsa, PeopleKind::Metsareunat) => 0.05,
            (PeopleKind::Muistikansa, PeopleKind::Taulukansa) => 0.06,
            (PeopleKind::Muistikansa, PeopleKind::Kirjakansa) => 0.06,
            (PeopleKind::Taulukansa, PeopleKind::Kirjakansa) => 0.07,
            (PeopleKind::Rantavaki, PeopleKind::Saarivaki) => 0.05,
            (PeopleKind::Haramaki, PeopleKind::Jamavaki) => 0.06,
            (PeopleKind::Haramaki, PeopleKind::Pohjavaki) => 0.07,
            (PeopleKind::Jamavaki, PeopleKind::Pohjavaki) => 0.08,
            // Stayed peoples to non-human
            (PeopleKind::Koskimetsa, PeopleKind::Hal) => 0.04,
            (PeopleKind::Rantavaki, PeopleKind::Merak) => 0.04,
            (PeopleKind::Haramaki, PeopleKind::Tzakhar) => 0.03,
            (PeopleKind::Pohjavaki, PeopleKind::Tzakhar) => 0.04,
            // Default
            (PeopleKind::Laakso, _) => -0.05,
            (PeopleKind::Arkit, _) => 0.0,
            (PeopleKind::Vayla, _) => 0.0,
            _ => 0.0,
        }
    }

    pub fn greeting_to(self, other: PeopleKind) -> &'static str {
        if self == other {
            return "You are among your own. Their eyes warm with recognition.";
        }
        match (self, other) {
            (PeopleKind::Metsik, PeopleKind::Sepat) | (PeopleKind::Metsik, PeopleKind::Ahjo) => {
                "They size you up. Forest-people are not loved here. 'Clearing-sympathizer,' someone mutters."
            }
            (PeopleKind::Sepat, PeopleKind::Metsik) => {
                "They eye your hands. 'Forest-dweller. Your kind took good iron-ore ground.'"
            }
            (PeopleKind::Ahjo, PeopleKind::Metsik) => {
                "A guarded look. 'Another one who thinks trees matter more than forge-heat.'"
            }
            (PeopleKind::Metsik, PeopleKind::Vayla) => {
                "Neutral, but watchful. 'Trader. You would sell the forest if the price was right.'"
            }
            (PeopleKind::Vayla, PeopleKind::Metsik) => {
                "Interested. 'Forest goods fetch fine prices. But we respect your... boundaries.'"
            }
            (PeopleKind::Laakso, PeopleKind::Vayla) => {
                "A long pause. 'You move too fast. Everything with you is a transaction.'"
            }
            (PeopleKind::Sepat, PeopleKind::Arkit) => {
                "A nod of respect. 'Keeper of knowledge. That is honest work.'"
            }
            (PeopleKind::Arkit, PeopleKind::Sepat) => {
                "Professional courtesy. 'Makers. You preserve things differently than we do.'"
            }
            (PeopleKind::Laakso, _) => "They watch you in silence. You must prove patience to earn their warmth.",
            (PeopleKind::Vayla, _) => "Open face, calculating eyes. 'Welcome, stranger. What do you trade?'",
            (PeopleKind::Arkit, _) => "Polite, measured. The archivist's neutral welcome.",
            (PeopleKind::Tzakhar, PeopleKind::Laakso) => "A faint nod from the shadows. 'Cave-kin. We know your ways.'",
            (PeopleKind::Tzakhar, _) => "Eyes that have seen too much dark. 'You are not of the deep. Walk carefully.'",
            (PeopleKind::Merak, PeopleKind::Vayla) => "A salty grin. 'River-folk! We share a love of water, at least.'",
            (PeopleKind::Merak, _) => "Sea-salt in the air. 'Land-walker. What brings you shoreward?'",
            (PeopleKind::Shear, _) => "Distant eyes. 'The sun is harsh. But we endure. As must you.'",
            (PeopleKind::Hal, PeopleKind::Metsik) => "A sway and a smile from above. 'Cousins of the canopy! Welcome.'",
            (PeopleKind::Hal, _) => "Bright eyes from the branches. 'Ground-walker. Interesting.'",
            (PeopleKind::Khor, PeopleKind::Arkit) => "A slow nod. 'You keep memory. We keep survival. Same work, different tool.'",
            (PeopleKind::Khor, _) => "A breath in the cold. 'The wind tests all equally. You are being tested now.'",
            // Stayed peoples greeting SAST
            (PeopleKind::Varhaiset, _) => "They look at you as if from a very long time ago. 'The source remembers.'",
            (PeopleKind::Metsareunat, PeopleKind::Metsik) => "A knowing look from the forest-edge. 'Cousin-who-went-deeper. The margin remembers.'",
            (PeopleKind::Porokansa, PeopleKind::Metsik) => "A reindeer-follower's measured gaze. 'The herd moved on. We follow still.'",
            (PeopleKind::Koskimetsa, PeopleKind::Metsik) => "Quiet eyes from the rapids. 'We fish where you hunt. The river runs between us.'",
            (PeopleKind::Koskimetsa, _) => "Quiet, appraising eyes. They offer river-fish without a word.",
            (PeopleKind::Muistikansa, PeopleKind::Arkit) => "An ancient stare. 'We sang before you wrote. The song is older than the tablet.'",
            (PeopleKind::Taulukansa, PeopleKind::Arkit) => "A wry smile. 'We had tablets too. We just chose not to stack them into an Archive.'",
            (PeopleKind::Kirjakansa, PeopleKind::Arkit) => "Careful courtesy. 'The river carried the books. The books carried us. We both serve memory, differently.'",
            (PeopleKind::Takovaki, PeopleKind::Sepat) => "A forge-worker's nod, but cautious. 'We worked copper before you found iron. The trailing sound, they called our god.'",
            (PeopleKind::Rantavaki, PeopleKind::Vayla) => "A shore-salt greeting. 'We cultivated the tide-pools while you sailed the deep. Both are water's work.'",
            (PeopleKind::Saarivaki, PeopleKind::Vayla) => "An islander's open welcome. 'Island-kin! The waves bring us news of you.'",
            (PeopleKind::Saarivaki, _) => "An islander's measured curiosity. 'Mainland-walker. The tides bring all kinds here.'",
            (PeopleKind::Hiekkakavelijat, PeopleKind::Vayla) => "A sand-walker's grin. 'We walk the beach-ridges. You sail past them. Same water, different legs.'",
            (PeopleKind::Haramaki, PeopleKind::Laakso) => "A rare, almost warm look. 'Härma valley recognizes Kyrö valley. The slopes remember.'",
            (PeopleKind::Jamavaki, PeopleKind::Laakso) => "Silence, then a slight nod. 'The hidden valley keeps its own counsel. As do we.'",
            (PeopleKind::Pohjavaki, PeopleKind::Laakso) => "Deep silence, then three words: 'The bottom speaks.'",
            (PeopleKind::Pohjavaki, _) => "Bottom-eyes. They watch you from the deepest place. 'You stand high. We stand in the root.'",
            _ => "Cautious eyes. A stranger from another people.",
        }
    }

    pub fn trade_modifier(self, seller: PeopleKind) -> f64 {
        1.0 - seller.bias_toward(self) * 0.3
    }

    pub fn true_endonym(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Čyrvä",
            PeopleKind::Arkit => "Märät",
            PeopleKind::Sepat => "Wosyt",
            PeopleKind::Ahjo => "Njumka",
            PeopleKind::Vayla => "Vylti",
            PeopleKind::Laakso => "Kiškam",
            // Stayed peoples: opaque roots per language family branch
            PeopleKind::Varhaiset => "Körvä",
            PeopleKind::Metsareunat => "Pyršä",
            PeopleKind::Porokansa => "Tuorva",
            PeopleKind::Koskimetsa => "Jälky",
            PeopleKind::Muistikansa => "Särät",
            PeopleKind::Taulukansa => "Velmät",
            PeopleKind::Kirjakansa => "Tärent",
            PeopleKind::Takovaki => "Wonśyt",
            PeopleKind::Rantavaki => "Vylri",
            PeopleKind::Saarivaki => "Kylmi",
            PeopleKind::Hiekkakavelijat => "Tyrväi",
            PeopleKind::Haramaki => "Kišmäs",
            PeopleKind::Jamavaki => "Hoskam",
            PeopleKind::Pohjavaki => "Väškam",
            // Non-humans: opaque native endonyms
            PeopleKind::Tzakhar => "Tzäkhar",
            PeopleKind::Merak => "Mëräk",
            PeopleKind::Shear => "She'ar",
            PeopleKind::Hal => "Häl",
            PeopleKind::Khor => "Khör",
        }
    }

    pub fn arkit_name(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Metsik",
            PeopleKind::Arkit => "Arkit",
            PeopleKind::Sepat => "Sepät",
            PeopleKind::Ahjo => "Ahjo",
            PeopleKind::Vayla => "Väylä",
            PeopleKind::Laakso => "Laakso",
            PeopleKind::Varhaiset => "Varhaiset",
            PeopleKind::Metsareunat => "Metsäreunat",
            PeopleKind::Porokansa => "Porokansa",
            PeopleKind::Koskimetsa => "Koskimetsä",
            PeopleKind::Muistikansa => "Muistikansa",
            PeopleKind::Taulukansa => "Taulukansa",
            PeopleKind::Kirjakansa => "Kirjakansa",
            PeopleKind::Takovaki => "Takoväki",
            PeopleKind::Rantavaki => "Rantaväki",
            PeopleKind::Saarivaki => "Saariväki",
            PeopleKind::Hiekkakavelijat => "Hiekkakävelijät",
            PeopleKind::Haramaki => "Härämäki",
            PeopleKind::Jamavaki => "Jämäväki",
            PeopleKind::Pohjavaki => "Pohjaväki",
            PeopleKind::Tzakhar => "Vaskiluuri",
            PeopleKind::Merak => "Iltäkälä",
            PeopleKind::Shear => "Muraskala",
            PeopleKind::Hal => "Khör",
            PeopleKind::Khor => "Khmört",
        }
    }

    pub fn pilgrimage_exonym(self) -> &'static str {
        match self {
            PeopleKind::Metsik => "Keurimä",
            PeopleKind::Arkit => "Sampsari",
            PeopleKind::Sepat => "Olkari",
            PeopleKind::Ahjo => "Iltkari",
            PeopleKind::Vayla => "Masari",
            PeopleKind::Laakso => "Kukreva",
            PeopleKind::Varhaiset => "Perikansan",
            PeopleKind::Metsareunat => "Keurunreunat",
            PeopleKind::Porokansa => "Keuruporo",
            PeopleKind::Koskimetsa => "Keurukoski",
            PeopleKind::Muistikansa => "Sampsamuisti",
            PeopleKind::Taulukansa => "Sampsataulu",
            PeopleKind::Kirjakansa => "Sampsakirja",
            PeopleKind::Takovaki => "Oltkartako",
            PeopleKind::Rantavaki => "Masaranta",
            PeopleKind::Saarivaki => "Masasaari",
            PeopleKind::Hiekkakavelijat => "Masahiekka",
            PeopleKind::Haramaki => "Kukriharma",
            PeopleKind::Jamavaki => "Kukrijämä",
            PeopleKind::Pohjavaki => "Kukripohja",
            PeopleKind::Tzakhar => "Vaskiluuri",
            PeopleKind::Merak => "Iltäkälä",
            PeopleKind::Shear => "Muraskala",
            PeopleKind::Hal => "Khör",
            PeopleKind::Khor => "Khmört",
        }
    }

    pub fn language_family(self) -> &'static str {
        match self {
            PeopleKind::Metsik
            | PeopleKind::Varhaiset
            | PeopleKind::Metsareunat
            | PeopleKind::Porokansa
            | PeopleKind::Koskimetsa => "Keurish",
            PeopleKind::Arkit
            | PeopleKind::Muistikansa
            | PeopleKind::Taulukansa
            | PeopleKind::Kirjakansa => "Sampsaran",
            PeopleKind::Sepat | PeopleKind::Ahjo | PeopleKind::Takovaki => "Oltkar",
            PeopleKind::Vayla
            | PeopleKind::Rantavaki
            | PeopleKind::Saarivaki
            | PeopleKind::Hiekkakavelijat => "Masaran",
            PeopleKind::Laakso
            | PeopleKind::Haramaki
            | PeopleKind::Jamavaki
            | PeopleKind::Pohjavaki => "Kukresh",
            PeopleKind::Tzakhar => "Deep-Isolate",
            PeopleKind::Merak => "Coastal-Isolate",
            PeopleKind::Shear => "Desert-Isolate",
            PeopleKind::Hal => "Canopy-Isolate",
            PeopleKind::Khor => "Steppe-Isolate",
        }
    }
}

/// A child of the player's household: a name and a birthday, grown against
/// the same compressed aging calendar as the player. Becomes a full Person
/// only if they inherit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HouseholdChild {
    pub name: String,
    pub born_day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InterPeopleBias {
    pub player_people: PeopleKind,
    #[serde(default)]
    pub bias_modifiers: HashMap<String, f64>,
}

impl InterPeopleBias {
    pub fn new(player_people: PeopleKind) -> Self {
        InterPeopleBias {
            player_people,
            bias_modifiers: HashMap::new(),
        }
    }

    pub fn mod_toward(&mut self, other: PeopleKind, delta: f64) {
        let key = format!("{:?}", other);
        let entry = self.bias_modifiers.entry(key).or_insert(0.0);
        // Wide enough that sustained grudge or goodwill genuinely matters:
        // the escalation ladder (serve-refusal -0.15, market -0.25, gate
        // -0.34) is reachable through play, and mendable the same way.
        *entry = (*entry + delta).clamp(-0.35, 0.35);
    }

    pub fn effective_bias(&self, other: PeopleKind) -> f64 {
        let base = self.player_people.bias_toward(other);
        let key = format!("{:?}", other);
        let modifier = self.bias_modifiers.get(&key).copied().unwrap_or(0.0);
        base + modifier
    }

    pub fn trust_baseline(&self, npc_people: PeopleKind) -> f64 {
        let bias = self.effective_bias(npc_people);
        (0.5 + bias).clamp(0.1, 0.9)
    }

    pub fn npc_trust_baseline(&self, npc_people: PeopleKind) -> f64 {
        let bias = npc_people.bias_toward(self.player_people);
        (0.5 + bias).clamp(0.1, 0.9)
    }

    pub fn strength_modifier(&self, npc_people: PeopleKind) -> f64 {
        self.effective_bias(npc_people) * 0.5
    }

    pub fn price_modifier(&self, seller_people: PeopleKind) -> f64 {
        self.player_people.trade_modifier(seller_people)
    }

    pub fn personality_mod(personality: &[String]) -> f64 {
        let mut mod_val = 0.0;
        for trait_val in personality {
            match trait_val.as_str() {
                "hospitable" | "warm" | "generous" => mod_val += 0.08,
                "xenophobic" | "suspicious" | "insular" => mod_val -= 0.10,
                "cautious" | "guarded" => mod_val -= 0.04,
                "open" | "curious" => mod_val += 0.05,
                // Previously chart-only flavor with no effect:
                "cheerful" | "earnest" => mod_val += 0.05,
                "melancholic" | "world-weary" => mod_val -= 0.04,
                _ => {}
            }
        }
        mod_val
    }

    pub fn trade_price_modifier(personality: &[String]) -> f64 {
        let mut mod_val = 0.0;
        for trait_val in personality {
            match trait_val.as_str() {
                "hospitable" | "generous" => mod_val -= 0.05,
                "mercenary" | "greedy" => mod_val += 0.08,
                "miserly" => mod_val += 0.10,
                "cautious" | "guarded" => mod_val += 0.03,
                "open" | "curious" => mod_val -= 0.02,
                "bitter" | "shrewd" => mod_val += 0.04,
                "reckless" => mod_val -= 0.03,
                // Previously chart-only flavor with no effect:
                "devious" => mod_val += 0.05,
                "proud" => mod_val += 0.03,
                "earnest" => mod_val -= 0.02,
                _ => {}
            }
        }
        mod_val
    }

    pub fn encounter_modifier(personality: &[String]) -> EncounterMod {
        let mut mod_val = EncounterMod::default();
        for trait_val in personality {
            match trait_val.as_str() {
                "hospitable" | "warm" | "generous" => mod_val.talk += 0.05,
                "xenophobic" | "suspicious" | "insular" => mod_val.flee += 0.08,
                "cautious" | "guarded" => mod_val.flee += 0.03,
                "open" | "curious" => mod_val.talk += 0.03,
                "mercenary" => mod_val.bribe_cost -= 0.10,
                "devout" => mod_val.calm += 0.05,
                "reckless" => mod_val.push_through += 0.05,
                "loyal" => mod_val.calm += 0.03,
                "bitter" => mod_val.intimidate += 0.04,
                "stoic" => mod_val.calm += 0.04,
                "withdrawn" => mod_val.talk -= 0.03,
                "shrewd" => mod_val.trade += 0.04,
                // Previously chart-only flavor with no effect:
                "proud" => mod_val.intimidate += 0.03,
                "cheerful" | "boisterous" => mod_val.talk += 0.03,
                "melancholic" => mod_val.talk -= 0.02,
                "devious" => mod_val.bribe_cost -= 0.05,
                "earnest" => mod_val.talk += 0.04,
                "world-weary" => mod_val.calm += 0.03,
                _ => {}
            }
        }
        mod_val
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncounterMod {
    pub talk: f64,
    pub flee: f64,
    pub calm: f64,
    pub push_through: f64,
    pub bribe_cost: f64,
    pub intimidate: f64,
    pub trade: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Need {
    Food,
    Money,
    Care,
    Presence,
    Safety,
}

impl fmt::Display for Need {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Need::Food => write!(f, "Food"),
            Need::Money => write!(f, "Money"),
            Need::Care => write!(f, "Care"),
            Need::Presence => write!(f, "Presence"),
            Need::Safety => write!(f, "Safety"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Needs {
    pub values: HashMap<Need, f64>,
}

impl Default for Needs {
    fn default() -> Self {
        let mut values = HashMap::new();
        values.insert(Need::Food, 0.8);
        values.insert(Need::Money, 0.8);
        values.insert(Need::Care, 0.8);
        values.insert(Need::Presence, 0.8);
        values.insert(Need::Safety, 0.8);
        Needs { values }
    }
}

impl Needs {
    pub fn get(&self, need: Need) -> f64 {
        self.values.get(&need).copied().unwrap_or(0.0)
    }

    pub fn satisfy(&mut self, need: Need, amount: f64) {
        let current = self.get(need);
        self.values.insert(need, (current + amount).clamp(0.0, 1.0));
    }

    pub fn decay(&mut self, need: Need, rate: f64) {
        let current = self.get(need);
        self.values.insert(need, (current - rate).clamp(0.0, 1.0));
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CraftAffinity {
    #[default]
    None,
    Word,
    Current,
    Still,
    Forge,
    Root,
}

impl CraftAffinity {
    pub fn from_chart_key(key: &str) -> Option<Self> {
        match key {
            "none" => Some(Self::None),
            "word" => Some(Self::Word),
            "current" => Some(Self::Current),
            "still" => Some(Self::Still),
            "forge" => Some(Self::Forge),
            "root" => Some(Self::Root),
            _ => None,
        }
    }

    pub fn to_chart_key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Word => "word",
            Self::Current => "current",
            Self::Still => "still",
            Self::Forge => "forge",
            Self::Root => "root",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NpcActivity {
    Sleep,
    Work,
    Socialize,
    Travel,
    Worship,
    Craft,
    Idle,
}

impl NpcActivity {
    pub fn name(self) -> &'static str {
        match self {
            NpcActivity::Sleep => "sleeping",
            NpcActivity::Work => "working",
            NpcActivity::Socialize => "socializing",
            NpcActivity::Travel => "traveling",
            NpcActivity::Worship => "worshipping",
            NpcActivity::Craft => "crafting",
            NpcActivity::Idle => "idle",
        }
    }

    pub fn is_available(self) -> bool {
        !matches!(self, NpcActivity::Sleep | NpcActivity::Travel)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcSchedule {
    pub blocks: [NpcActivity; 6],
}

impl Default for NpcSchedule {
    fn default() -> Self {
        NpcSchedule {
            blocks: [
                NpcActivity::Sleep,
                NpcActivity::Work,
                NpcActivity::Work,
                NpcActivity::Socialize,
                NpcActivity::Idle,
                NpcActivity::Sleep,
            ],
        }
    }
}

impl NpcSchedule {
    pub fn activity_at_hour(&self, hour: u32) -> NpcActivity {
        let block_idx = (hour / 4) as usize;
        self.blocks[block_idx.min(5)]
    }

    pub fn is_available_at_hour(&self, hour: u32) -> bool {
        self.activity_at_hour(hour).is_available()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CombatAction {
    Attack,
    Parry,
    Feint,
    Yield,
}

impl CombatAction {
    pub fn name(self) -> &'static str {
        match self {
            CombatAction::Attack => "attack",
            CombatAction::Parry => "parry",
            CombatAction::Feint => "feint",
            CombatAction::Yield => "yield",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombatOutcome {
    pub player_injury: f64,
    pub npc_injury: f64,
    pub reputation_delta: f64,
    pub player_died: bool,
    pub npc_yielded: bool,
    pub flavor: String,
}

impl CombatOutcome {
    pub fn resolve(
        player_action: CombatAction,
        npc_action: CombatAction,
        player_trust: f64,
        npc_aggression: f64,
        seed: u64,
    ) -> Self {
        let mut rng = crate::rng::SeedRng::new(seed);
        let roll = rng.gen_range(1000) as f64 / 1000.0;

        // Death is rare (~1.2%)
        let player_died = roll < 0.012;

        let (player_injury, npc_injury, reputation_delta, npc_yielded, flavor) =
            match (player_action, npc_action) {
                (CombatAction::Yield, _) => (
                    0.1,
                    0.0,
                    -0.05,
                    false,
                    "You yield gracefully. They let you go with a warning.",
                ),
                (_, CombatAction::Yield) => (
                    0.0,
                    0.05,
                    0.02,
                    true,
                    "They yield. The duel ends with respect.",
                ),
                (CombatAction::Attack, CombatAction::Attack) => {
                    if player_trust > 0.5 {
                        (
                            0.15,
                            0.25,
                            0.01,
                            false,
                            "Your strike lands true. They stagger back, wounded.",
                        )
                    } else {
                        (
                            0.25,
                            0.15,
                            -0.01,
                            false,
                            "They meet your blow. You both bleed, but they seem stronger.",
                        )
                    }
                }
                (CombatAction::Attack, CombatAction::Parry) => (
                    0.2,
                    0.05,
                    -0.02,
                    false,
                    "They parry your attack. You overextend and take a hit.",
                ),
                (CombatAction::Attack, CombatAction::Feint) => (
                    0.05,
                    0.3,
                    0.03,
                    false,
                    "You see through their feint. Your attack catches them off-guard.",
                ),
                (CombatAction::Parry, CombatAction::Attack) => (
                    0.05,
                    0.2,
                    0.02,
                    false,
                    "You parry their attack. They stumble, exposed.",
                ),
                (CombatAction::Parry, CombatAction::Parry) => (
                    0.0,
                    0.0,
                    0.0,
                    false,
                    "You circle each other, blades raised. Neither commits.",
                ),
                (CombatAction::Parry, CombatAction::Feint) => (
                    0.1,
                    0.0,
                    -0.01,
                    false,
                    "Their feint draws your parry. They strike, but you deflect most of it.",
                ),
                (CombatAction::Feint, CombatAction::Attack) => (
                    0.3,
                    0.05,
                    -0.03,
                    false,
                    "Your feint fails. They punish your opening.",
                ),
                (CombatAction::Feint, CombatAction::Parry) => (
                    0.0,
                    0.1,
                    0.01,
                    false,
                    "Your feint draws their parry. You find an opening.",
                ),
                (CombatAction::Feint, CombatAction::Feint) => (
                    0.0,
                    0.0,
                    0.0,
                    false,
                    "Both feint. Neither commits. The dance continues.",
                ),
            };

        // Apply aggression modifier to injuries
        let player_injury = (player_injury * (1.0 + npc_aggression * 0.3)).min(0.5);
        let npc_injury = (npc_injury * (1.0 - npc_aggression * 0.2)).min(0.5);

        CombatOutcome {
            player_injury,
            npc_injury,
            reputation_delta,
            player_died,
            npc_yielded,
            flavor: flavor.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub people: String,
    pub sex: String,
    pub age_band: String,
    pub profession: String,
    pub social_class: String,
    pub craft_affinity: String,
    pub personality: Vec<String>,
    pub bias: String,
    pub needs: Needs,
    pub region: String,
    pub settlement: String,
    pub has_spouse: bool,
    pub children_count: u32,
    pub has_debt: bool,
    /// Age in life-years (0 = not yet derived from age_band; see lifecycle).
    #[serde(default)]
    pub age_years: u32,
    #[serde(default)]
    pub schedule: NpcSchedule,
    #[serde(default)]
    pub illnesses: Vec<ActiveDisease>,
    #[serde(default)]
    pub relations: Vec<relation::InterNpcRelation>,
    #[serde(default)]
    pub wants: Vec<crate::sim::wants::NpcWant>,
    /// The rare innate craft-gift — almost always none, as for any life (#441).
    #[serde(default)]
    pub gift: crate::model::Gift,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub people: String,
    pub sex: String,
    pub age_band: String,
    pub profession: String,
    pub social_class: String,
    pub craft_affinity: CraftAffinity,
    pub personality: Vec<String>,
    pub region: String,
    pub settlement: String,
    pub perks: Vec<Perk>,
    pub household_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Perk {
    pub name: String,
    pub description: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerStart {
    pub person: Person,
    pub reroll_count: u32,
    pub point_buy_adjustments: Vec<Adjustment>,
    pub accepted: bool,
    #[serde(default)]
    pub inventory: Inventory,
    #[serde(default)]
    pub companions: Vec<Companion>,
}

impl PlayerStart {
    pub fn adopt_companion(&mut self, animal: Animal, name: String, tick: u64) {
        let c = Companion::new(animal, name, tick);
        self.companions.push(c);
    }

    pub fn has_companion_kind(&self, kind: Animal) -> bool {
        self.companions.iter().any(|c| c.animal == kind)
    }

    pub fn total_carry_bonus(&self) -> u32 {
        self.companions
            .iter()
            .map(|c| c.animal.carry_capacity_bonus())
            .sum()
    }

    pub fn has_hound(&self) -> bool {
        self.companions
            .iter()
            .any(|c| matches!(c.animal, Animal::Dog | Animal::Hound))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Adjustment {
    SwapProfession(String),
    SetCraft(CraftAffinity),
    AddPerk(Perk),
    CutHouseholdTie,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Debt {
    pub creditor_id: String,
    pub amount: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Household {
    pub id: String,
    pub head_id: String,
    pub spouse_id: Option<String>,
    pub children_ids: Vec<String>,
    pub location_settlement_id: String,
    pub has_debt: bool,
    pub debts: Vec<Debt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipKind {
    Spouse,
    Parent,
    Child,
    Sibling,
    Kin,
    Friend,
    Rival,
    Patron,
    Apprentice,
    Guildmate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipEvent {
    pub tick: u64,
    pub description: String,
}

fn default_trust_baseline() -> f64 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relationship {
    pub from_id: String,
    pub to_id: String,
    pub kind: RelationshipKind,
    pub strength: f64,
    pub trust: f64,
    #[serde(default = "default_trust_baseline")]
    pub trust_baseline: f64,
    pub history: Vec<RelationshipEvent>,
}

// QuestType moved to quest.rs — re-exported via pub use
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
    fn the_five_keep_a_god_and_name_it_truly() {
        // Each of the Five surfaces a god-relation; the humans keep none here.
        for pk in [
            PeopleKind::Tzakhar,
            PeopleKind::Merak,
            PeopleKind::Shear,
            PeopleKind::Hal,
            PeopleKind::Khor,
        ] {
            let rel = pk
                .enclave_god_relation()
                .unwrap_or_else(|| panic!("{:?} keeps a god", pk));
            let god = pk.patron_god().expect("the Five have a patron");
            assert!(
                rel.contains(god.label())
                    || (pk == PeopleKind::Khor && rel.contains("Talven-Hiljaisuus")),
                "{:?}'s relation names its god",
                pk
            );
        }
        assert!(PeopleKind::Metsik.enclave_god_relation().is_none());
    }

    #[test]

    fn roundtrip_person() {
        let person = Person {
            id: "abc-123".into(),

            name: "Testi".into(),

            people: "metsik".into(),

            sex: "f".into(),

            age_band: "adult".into(),

            profession: "farmer".into(),

            social_class: "low".into(),

            craft_affinity: "none".into(),

            personality: vec!["stoic".into(), "curious".into()],

            bias: "metsik".into(),

            needs: {
                let mut v = HashMap::new();

                v.insert(Need::Food, 0.8);

                v.insert(Need::Safety, 0.5);

                v.insert(Need::Care, 0.6);

                v.insert(Need::Money, 0.3);

                v.insert(Need::Presence, 0.7);

                Needs { values: v }
            },

            region: "river_valley".into(),

            settlement: "hamlet-1".into(),

            has_spouse: true,

            children_count: 2,

            has_debt: false,

            schedule: NpcSchedule::default(),

            illnesses: Vec::new(),

            relations: vec![],

            wants: vec![],
            gift: Default::default(),
            age_years: 0,
        };

        roundtrip(&person);
    }

    #[test]

    fn person_default_no_panic() {
        let p = Person::default();

        assert!(p.name.is_empty());

        assert!(p.personality.is_empty());
    }

    #[test]

    fn world_holds_many_persons() {
        let mut world = World::default();

        let region = Region {
            id: "r1".into(),

            name: "Test".into(),

            region_type: "river_valley".into(),

            region_subtype: "flood_plain".into(),

            description: "desc".into(),

            terrain: TerrainMap::default(),

            neighbors: RegionNeighbors::default(),

            structures: vec![],

            settlements: vec![Settlement {
                id: "s1".into(),

                name: "V".into(),

                size: "village".into(),

                region: "river_valley".into(),

                population: 10_000,

                description: "desc".into(),

                people: (0..10_000)
                    .map(|i| Person {
                        id: format!("p{}", i),

                        ..Default::default()
                    })
                    .collect(),

                services: vec![],

                politics: SettlementPolitics::new(),
                food_stock: 0.0,
                goods_stock: Default::default(),
                farms: Vec::new(),
                buildings: Vec::new(),
                festival_until_day: 0,
                famine_days: 0,
                map_x: 0,
                map_y: 0,
                district: 0,
                remembered_deed: None,
            }],
            weather: crate::model::Weather::Clear,
            game_richness: 1.0,
        };

        world.regions.push(region);

        let total: usize = world
            .regions
            .iter()
            .flat_map(|r| r.settlements.iter())
            .map(|s| s.people.len())
            .sum();

        assert_eq!(total, 10_000);
    }

    #[test]

    fn roundtrip_household() {
        let h = Household {
            id: "hh-1".into(),

            head_id: "p1".into(),

            spouse_id: Some("p2".into()),

            children_ids: vec!["p3".into(), "p4".into()],

            location_settlement_id: "set-1".into(),

            has_debt: true,

            debts: vec![Debt {
                creditor_id: "p5".into(),

                amount: 50.0,

                description: "seed loan".into(),
            }],
        };

        roundtrip(&h);
    }

    #[test]

    fn roundtrip_relationship() {
        let r = Relationship {
            from_id: "p1".into(),

            to_id: "p2".into(),

            kind: RelationshipKind::Friend,

            strength: 0.7,

            trust: 0.5,

            trust_baseline: 0.5,

            history: vec![RelationshipEvent {
                tick: 10,

                description: "shared a meal".into(),
            }],
        };

        roundtrip(&r);
    }

    #[test]

    fn craft_affinity_roundtrip_chart_keys() {
        for key in &["none", "word", "current", "still", "forge", "root"] {
            let affinity = CraftAffinity::from_chart_key(key).unwrap();

            assert_eq!(affinity.to_chart_key(), *key);
        }

        roundtrip(&CraftAffinity::Forge);
    }

    #[test]

    fn relationship_kind_all_variants() {
        let variants = [
            RelationshipKind::Spouse,
            RelationshipKind::Parent,
            RelationshipKind::Child,
            RelationshipKind::Sibling,
            RelationshipKind::Kin,
            RelationshipKind::Friend,
            RelationshipKind::Rival,
            RelationshipKind::Patron,
            RelationshipKind::Apprentice,
            RelationshipKind::Guildmate,
        ];

        for v in &variants {
            roundtrip(v);
        }
    }

    #[test]

    fn need_enum_roundtrip() {
        roundtrip(&Need::Food);

        roundtrip(&Need::Safety);
    }

    #[test]

    fn roundtrip_player() {
        let p = Player {
            id: "player-1".into(),

            name: "Hero".into(),

            people: "metsik".into(),

            sex: "m".into(),

            age_band: "youth".into(),

            profession: "forester".into(),

            social_class: "low".into(),

            craft_affinity: CraftAffinity::Root,

            personality: vec!["curious".into()],

            region: "forest".into(),

            settlement: "set-1".into(),

            perks: vec![Perk {
                name: "Keen Eye".into(),

                description: "Spot details others miss".into(),

                source: "personality_traits".into(),
            }],

            household_id: Some("hh-1".into()),
        };

        roundtrip(&p);
    }

    #[test]

    fn player_default_valid() {
        let p = Player::default();

        assert!(p.name.is_empty());

        assert!(p.perks.is_empty());

        assert!(p.household_id.is_none());
    }

    #[test]

    fn roundtrip_player_start() {
        let ps = PlayerStart {
            person: Person::default(),

            reroll_count: 2,

            point_buy_adjustments: vec![
                Adjustment::SwapProfession("trader".into()),
                Adjustment::SetCraft(CraftAffinity::Current),
                Adjustment::AddPerk(Perk {
                    name: "Silver Tongue".into(),

                    description: "Better trade deals".into(),

                    source: "profession".into(),
                }),
                Adjustment::CutHouseholdTie,
            ],

            accepted: false,

            inventory: Inventory::default(),

            companions: vec![],
        };

        roundtrip(&ps);
    }

    #[test]

    fn adjustment_all_variants_roundtrip() {
        roundtrip(&Adjustment::SwapProfession("smith".into()));

        roundtrip(&Adjustment::SetCraft(CraftAffinity::Forge));

        roundtrip(&Adjustment::AddPerk(Perk {
            name: "test".into(),

            description: "test perk".into(),

            source: "test".into(),
        }));

        roundtrip(&Adjustment::CutHouseholdTie);
    }

    #[test]

    fn needs_default_satisfied_baseline() {
        let needs = Needs::default();

        for need in &[
            Need::Food,
            Need::Money,
            Need::Care,
            Need::Presence,
            Need::Safety,
        ] {
            assert!(
                (needs.get(*need) - 0.8).abs() < f64::EPSILON,
                "default {} should be 0.8, got {}",
                need,
                needs.get(*need)
            );
        }
    }

    #[test]

    fn needs_decay_reduces_value() {
        let mut needs = Needs::default();

        needs.decay(Need::Food, 0.1);

        assert!(
            (needs.get(Need::Food) - 0.7).abs() < f64::EPSILON,
            "food after decay should be 0.7, got {}",
            needs.get(Need::Food)
        );
    }

    #[test]

    fn needs_satisfy_increases_value() {
        let mut needs = Needs::default();

        needs.decay(Need::Food, 0.5);

        needs.satisfy(Need::Food, 0.3);

        assert!(
            (needs.get(Need::Food) - 0.6).abs() < f64::EPSILON,
            "food after satisfy should be 0.6, got {}",
            needs.get(Need::Food)
        );
    }

    #[test]

    fn needs_satisfy_clamped_at_one() {
        let mut needs = Needs::default();

        needs.satisfy(Need::Food, 0.3);

        assert!(
            (needs.get(Need::Food) - 1.0).abs() < f64::EPSILON,
            "food should clamp at 1.0, got {}",
            needs.get(Need::Food)
        );
    }

    #[test]

    fn needs_decay_clamped_at_zero() {
        let mut needs = Needs::default();

        needs.decay(Need::Food, 1.0);

        assert!(
            (needs.get(Need::Food)).abs() < f64::EPSILON,
            "food should clamp at 0.0, got {}",
            needs.get(Need::Food)
        );

        needs.decay(Need::Food, 0.5);

        assert!(
            (needs.get(Need::Food)).abs() < f64::EPSILON,
            "food should still be 0.0 after extra decay, got {}",
            needs.get(Need::Food)
        );
    }

    #[test]

    fn need_display() {
        assert_eq!(format!("{}", Need::Food), "Food");

        assert_eq!(format!("{}", Need::Money), "Money");

        assert_eq!(format!("{}", Need::Care), "Care");

        assert_eq!(format!("{}", Need::Presence), "Presence");

        assert_eq!(format!("{}", Need::Safety), "Safety");
    }

    #[test]

    fn needs_roundtrip() {
        roundtrip(&Needs::default());
    }

    #[test]

    fn god_affinity_adjust_clamps() {
        let mut ga = GodAffinity::new();

        ga.adjust(GodName::Keuru, 2.0);

        assert!((ga.get(GodName::Keuru) - 1.0).abs() < f64::EPSILON);

        ga.adjust(GodName::Keuru, -3.0);

        assert!((ga.get(GodName::Keuru) + 1.0).abs() < f64::EPSILON);
    }

    #[test]

    fn god_affinity_ally_and_grudge() {
        let mut ga = GodAffinity::new();

        assert_eq!(ga.strongest_ally(), None);

        assert_eq!(ga.strongest_grudge(), None);

        ga.adjust(GodName::Keuru, 0.5);

        ga.adjust(GodName::Oltzed, -0.3);

        assert_eq!(ga.strongest_ally(), Some(GodName::Keuru));

        assert_eq!(ga.strongest_grudge(), Some(GodName::Oltzed));
    }

    #[test]

    fn people_kind_from_str() {
        assert_eq!(PeopleKind::from_name("metsik"), PeopleKind::Metsik);

        assert_eq!(PeopleKind::from_name("Sepät"), PeopleKind::Sepat);

        assert_eq!(PeopleKind::from_name("vayla"), PeopleKind::Vayla);
    }

    #[test]

    fn people_bias_same_people() {
        assert!((PeopleKind::Metsik.bias_toward(PeopleKind::Metsik) - 0.15).abs() < f64::EPSILON);
    }

    #[test]

    fn people_bias_metsik_sepat_hostile() {
        assert!(PeopleKind::Metsik.bias_toward(PeopleKind::Sepat) < -0.15);

        assert!(PeopleKind::Sepat.bias_toward(PeopleKind::Metsik) < -0.10);
    }

    #[test]

    fn people_bias_sepat_ahjo_allied() {
        assert!(PeopleKind::Sepat.bias_toward(PeopleKind::Ahjo) > 0.0);

        assert!(PeopleKind::Ahjo.bias_toward(PeopleKind::Sepat) > 0.0);
    }

    #[test]

    fn season_bias_modifier() {
        assert_eq!(Season::Green.bias_modifier(), 0.05);

        assert_eq!(Season::Frost.bias_modifier(), -0.05);

        assert_eq!(Season::Thaw.bias_modifier(), 0.0);
    }

    #[test]

    fn trade_price_modifier_hospitable() {
        let r = InterPeopleBias::trade_price_modifier(&["hospitable".into()]);

        assert!(r < 0.0, "hospitable should reduce price: {r}");
    }

    #[test]

    fn trade_price_modifier_mercenary() {
        let r = InterPeopleBias::trade_price_modifier(&["mercenary".into()]);

        assert!(r > 0.0, "mercenary should increase price: {r}");
    }

    #[test]

    fn encounter_modifier_devout() {
        let r = InterPeopleBias::encounter_modifier(&["devout".into()]);

        assert!(r.calm > 0.0, "devout should boost calm: {}", r.calm);
    }

    #[test]

    fn encounter_modifier_xenophobic() {
        let r = InterPeopleBias::encounter_modifier(&["xenophobic".into()]);

        assert!(r.flee > 0.0, "xenophobic should boost flee: {}", r.flee);
    }

    #[test]

    fn people_bias_laakso_xenophobic() {
        assert!(PeopleKind::Laakso.bias_toward(PeopleKind::Vayla) < 0.0);

        assert!(PeopleKind::Laakso.bias_toward(PeopleKind::Ahjo) < 0.0);
    }

    #[test]
    fn the_five_are_the_enclave_peoples() {
        for k in ["khör", "tzäkhar", "mëräk", "she'ar", "häl"] {
            assert!(
                PeopleKind::from_name(k).is_of_the_five(),
                "{k} should be one of the Five"
            );
        }
        for k in ["metsik", "sepät", "väylä", "arkit"] {
            assert!(
                !PeopleKind::from_name(k).is_of_the_five(),
                "{k} is a human people, not of the Five"
            );
        }
    }

    #[test]

    fn inter_people_bias_price_modifier() {
        let ib = InterPeopleBias::new(PeopleKind::Metsik);

        assert!(
            ib.price_modifier(PeopleKind::Sepat) > 1.0,
            "Metsik buying from Sepat (hostile seller) should pay more"
        );

        assert!(
            ib.price_modifier(PeopleKind::Metsik) < 1.0,
            "Metsik buying from fellow Metsik (friendly seller) should pay less"
        );
    }

    #[test]

    fn personality_mod_hospitable() {
        let mod_val = InterPeopleBias::personality_mod(&["hospitable".into()]);

        assert!(mod_val > 0.0, "hospitable should increase trust");
    }

    #[test]

    fn personality_mod_xenophobic() {
        let mod_val = InterPeopleBias::personality_mod(&["xenophobic".into()]);

        assert!(mod_val < 0.0, "xenophobic should decrease trust");
    }

    #[test]

    fn greeting_cross_people() {
        let greeting = PeopleKind::Metsik.greeting_to(PeopleKind::Sepat);

        assert!(!greeting.is_empty());

        assert!(greeting.contains("Forest-people") || greeting.contains("iron-ore"));
    }

    #[test]

    fn npc_schedule_default_blocks() {
        let schedule = NpcSchedule::default();

        assert_eq!(schedule.blocks.len(), 6);

        assert_eq!(schedule.activity_at_hour(0), NpcActivity::Sleep);

        assert_eq!(schedule.activity_at_hour(4), NpcActivity::Work);

        assert_eq!(schedule.activity_at_hour(8), NpcActivity::Work);

        assert_eq!(schedule.activity_at_hour(12), NpcActivity::Socialize);

        assert_eq!(schedule.activity_at_hour(16), NpcActivity::Idle);

        assert_eq!(schedule.activity_at_hour(20), NpcActivity::Sleep);
    }

    #[test]

    fn npc_schedule_availability() {
        let schedule = NpcSchedule::default();

        assert!(!schedule.is_available_at_hour(0)); // Sleep

        assert!(schedule.is_available_at_hour(4)); // Work

        assert!(schedule.is_available_at_hour(8)); // Work

        assert!(schedule.is_available_at_hour(12)); // Socialize

        assert!(schedule.is_available_at_hour(16)); // Idle

        assert!(!schedule.is_available_at_hour(20)); // Sleep
    }

    #[test]

    fn npc_activity_availability() {
        assert!(!NpcActivity::Sleep.is_available());

        assert!(NpcActivity::Work.is_available());

        assert!(NpcActivity::Socialize.is_available());

        assert!(!NpcActivity::Travel.is_available());

        assert!(NpcActivity::Worship.is_available());

        assert!(NpcActivity::Craft.is_available());

        assert!(NpcActivity::Idle.is_available());
    }

    #[test]

    fn combat_action_names() {
        assert_eq!(CombatAction::Attack.name(), "attack");

        assert_eq!(CombatAction::Parry.name(), "parry");

        assert_eq!(CombatAction::Feint.name(), "feint");

        assert_eq!(CombatAction::Yield.name(), "yield");
    }

    #[test]

    fn combat_outcome_deterministic() {
        let outcome1 =
            CombatOutcome::resolve(CombatAction::Attack, CombatAction::Attack, 0.5, 0.5, 42);

        let outcome2 =
            CombatOutcome::resolve(CombatAction::Attack, CombatAction::Attack, 0.5, 0.5, 42);

        assert_eq!(outcome1.player_injury, outcome2.player_injury);

        assert_eq!(outcome1.npc_injury, outcome2.npc_injury);

        assert_eq!(outcome1.reputation_delta, outcome2.reputation_delta);

        assert_eq!(outcome1.player_died, outcome2.player_died);
    }

    #[test]

    fn combat_yield_no_death() {
        let outcome =
            CombatOutcome::resolve(CombatAction::Yield, CombatAction::Attack, 0.0, 0.5, 12345);

        assert!(!outcome.player_died);

        assert!(outcome.player_injury > 0.0);
    }

    #[test]

    fn npc_combat_action_varies_by_trust() {
        let aggressive = npc_combat_action(0.0, 0.8, 42);

        let defensive = npc_combat_action(0.9, 0.2, 42);

        // Different trust levels should produce different actions (with same seed)

        assert_ne!(aggressive, defensive);
    }

    #[test]
    fn the_five_welcome_strangers_humans_do_not() {
        for p in [
            PeopleKind::Tzakhar,
            PeopleKind::Merak,
            PeopleKind::Shear,
            PeopleKind::Hal,
            PeopleKind::Khor,
        ] {
            assert!(p.is_of_the_five());
            assert!(
                p.enclave_welcome().is_some(),
                "{p:?} greets you in its own character"
            );
        }
        // A human people keeps a plain town, no enclave welcome.
        assert!(!PeopleKind::Metsik.is_of_the_five());
        assert!(PeopleKind::Metsik.enclave_welcome().is_none());
    }

    #[test]
    fn the_five_each_have_first_visit_lore() {
        for p in [
            PeopleKind::Tzakhar,
            PeopleKind::Merak,
            PeopleKind::Shear,
            PeopleKind::Hal,
            PeopleKind::Khor,
        ] {
            assert!(p.enclave_lore().is_some(), "{p:?} has lore to reveal");
        }
        assert!(PeopleKind::Metsik.enclave_lore().is_none());
    }

    #[test]

    fn combat_outcome_ranges_valid() {
        for seed in 0..100 {
            let outcome =
                CombatOutcome::resolve(CombatAction::Attack, CombatAction::Attack, 0.5, 0.5, seed);

            assert!(outcome.player_injury >= 0.0 && outcome.player_injury <= 0.5);

            assert!(outcome.npc_injury >= 0.0 && outcome.npc_injury <= 0.5);

            assert!(outcome.reputation_delta >= -0.1 && outcome.reputation_delta <= 0.1);
        }
    }
}
