use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollapseOutcome {
    BeastNest,
    BeastGuarded,
    HostileBeast,
    StrangerHut,
    SettlementBed,
    WaysideShrine,
    Riverbank,
    FestivalBench,
    Ditch,
    GodCampsite,
}

impl CollapseOutcome {
    pub fn description(self) -> &'static str {
        match self {
            CollapseOutcome::BeastNest => "You wake tangled in foul straw, a beast's breath still warm beside you. It fled at your stirring.",
            CollapseOutcome::BeastGuarded => "You wake warm. A great beast lies beside you — it watched over you through the night. It rises, snorts, and vanishes into the trees. The forest remembers kindness.",
            CollapseOutcome::HostileBeast => "Pain. Something was chewing on you. You thrash and it bolts, but the wounds are real. The wild has no love for you.",
            CollapseOutcome::StrangerHut => "You wake on a rough pallet. A stranger props broth by your head. They shake their head at you — wordless, disappointed — and wave you out.",
            CollapseOutcome::SettlementBed => "A proper bed. Clean sheets. Someone carried you here — the townsfolk whisper outside the door. Your name means something here.",
            CollapseOutcome::WaysideShrine => "Cold stone under your back. A shrine's shadow. Someone left an offering of bread — you took it. The shrine feels... watchful.",
            CollapseOutcome::Riverbank => "Water on your face. You're lying on smooth stones by a river, soaked but alive. The current carried you somewhere gentler. A traveler's cairn marks the spot.",
            CollapseOutcome::FestivalBench => "Laughter. Firelight. You're slumped on a bench at some festival — someone draped a blanket over you. A child pokes your arm offering bread. The hearth-fires burn bright here.",
            CollapseOutcome::Ditch => "Mud. Cold mud. You're in a ditch. Something crawled over you in the night. Your coin pouch feels lighter.",
            CollapseOutcome::GodCampsite => "You wake at a campsite no one admits to keeping. The fire is banked just so; food and herbs sit wrapped by your hand; the ground beside you holds the warmth of company. There are no tracks. You will tell no one, and they would not believe you, and you are not sure you believe it yourself.",
        }
    }

    pub fn glyph(self) -> char {
        match self {
            CollapseOutcome::BeastNest => '🐺',
            CollapseOutcome::BeastGuarded => '🦌',
            CollapseOutcome::HostileBeast => '🩸',
            CollapseOutcome::StrangerHut => '🏠',
            CollapseOutcome::SettlementBed => '🛏',
            CollapseOutcome::WaysideShrine => '⛩',
            CollapseOutcome::Riverbank => '🌊',
            CollapseOutcome::FestivalBench => '🎉',
            CollapseOutcome::Ditch => '🕳',
            CollapseOutcome::GodCampsite => '✦',
        }
    }

    pub fn is_divine(self) -> bool {
        matches!(self, CollapseOutcome::GodCampsite)
    }

    pub fn is_beast_aided(self) -> bool {
        matches!(self, CollapseOutcome::BeastGuarded)
    }

    pub fn is_hostile(self) -> bool {
        matches!(self, CollapseOutcome::HostileBeast | CollapseOutcome::Ditch)
    }

    pub fn is_safe(self) -> bool {
        matches!(
            self,
            CollapseOutcome::SettlementBed
                | CollapseOutcome::FestivalBench
                | CollapseOutcome::BeastGuarded
                | CollapseOutcome::GodCampsite
        )
    }

    pub fn coin_loss(self) -> u32 {
        match self {
            CollapseOutcome::Ditch => 3,
            CollapseOutcome::HostileBeast => 4,
            CollapseOutcome::BeastNest => 1,
            CollapseOutcome::Riverbank => 1,
            _ => 0,
        }
    }

    pub fn item_loss(self) -> Option<ItemType> {
        match self {
            CollapseOutcome::HostileBeast => Some(ItemType::Food),
            CollapseOutcome::Ditch => Some(ItemType::Coin),
            _ => None,
        }
    }

    pub fn hunger_restore(self) -> f64 {
        match self {
            CollapseOutcome::SettlementBed => 0.6,
            CollapseOutcome::FestivalBench => 0.7,
            CollapseOutcome::StrangerHut => 0.5,
            CollapseOutcome::WaysideShrine => 0.3,
            CollapseOutcome::GodCampsite => 0.8,
            CollapseOutcome::BeastGuarded => 0.4,
            CollapseOutcome::Riverbank => 0.3,
            CollapseOutcome::HostileBeast => 0.15,
            CollapseOutcome::BeastNest => 0.2,
            CollapseOutcome::Ditch => 0.1,
        }
    }

    pub fn energy_restore(self) -> f64 {
        match self {
            CollapseOutcome::SettlementBed => 0.6,
            CollapseOutcome::FestivalBench => 0.5,
            CollapseOutcome::StrangerHut => 0.4,
            CollapseOutcome::WaysideShrine => 0.3,
            CollapseOutcome::GodCampsite => 1.0,
            CollapseOutcome::BeastGuarded => 0.5,
            CollapseOutcome::Riverbank => 0.3,
            CollapseOutcome::HostileBeast => 0.1,
            CollapseOutcome::BeastNest => 0.3,
            CollapseOutcome::Ditch => 0.2,
        }
    }

    pub fn hours_passed(self) -> u32 {
        match self {
            CollapseOutcome::SettlementBed => 14,
            CollapseOutcome::FestivalBench => 10,
            CollapseOutcome::GodCampsite => 16,
            CollapseOutcome::BeastGuarded => 12,
            CollapseOutcome::StrangerHut => 12,
            CollapseOutcome::WaysideShrine => 8,
            CollapseOutcome::Riverbank => 8,
            CollapseOutcome::HostileBeast => 6,
            CollapseOutcome::BeastNest => 10,
            CollapseOutcome::Ditch => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathCause {
    Starvation,
    Exposure,
    Exhaustion,
    Wounds,
    /// Sickness — a fever, a wound gone bad, a plague year. In a world without
    /// medicine the great leveller of ordinary lives.
    Sickness,
    OldAge,
    Unknown,
}

impl DeathCause {
    pub fn from_collapse_and_vitals(outcome: CollapseOutcome, vitals: PlayerVitals) -> Self {
        if vitals.hunger <= 0.0 {
            DeathCause::Starvation
        } else if vitals.thirst <= 0.0 {
            DeathCause::Exposure
        } else if vitals.energy <= 0.05 {
            DeathCause::Exhaustion
        } else if outcome.is_hostile() {
            DeathCause::Wounds
        } else {
            DeathCause::Unknown
        }
    }

    pub fn flavor(self) -> &'static str {
        match self {
            DeathCause::Starvation => "Hunger gnawed at your resolve until there was nothing left. The land has claimed another wanderer.",
            DeathCause::Exposure => "The elements exacted their toll. Your body could endure no more.",
            DeathCause::Exhaustion => "Your strength gave way. Even the will to rise has its limit.",
            DeathCause::Wounds => "Your wounds proved too great. The wilds are not forgiving.",
            DeathCause::Sickness => "The sickness ran its course in you, and you could not outlast it. So the world thins its people, one fever at a time.",
            DeathCause::OldAge => "Your years came full circle. You met the end as the old do — quietly, and at home in the world.",
            DeathCause::Unknown => "The end came quietly. The world holds its breath.",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DeathCause::Starvation => "starvation",
            DeathCause::Exposure => "exposure",
            DeathCause::Exhaustion => "exhaustion",
            DeathCause::Wounds => "wounds",
            DeathCause::Sickness => "sickness",
            DeathCause::OldAge => "old age",
            DeathCause::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collapse {
    pub outcome: CollapseOutcome,
    pub died: bool,
    pub rescued_by: Option<GodName>,
}

impl Collapse {
    pub fn roll(seed: u64, affinity: &GodAffinity, local_rep: f64) -> Self {
        let hash = seed.wrapping_mul(2654435761) ^ 0xDEAD;
        let val = (hash % 1000) as u32;

        let died = val < 12;

        let mut weights: [(CollapseOutcome, u32); 10] = [
            (CollapseOutcome::GodCampsite, 3),
            (CollapseOutcome::BeastGuarded, 5),
            (CollapseOutcome::HostileBeast, 30),
            (CollapseOutcome::BeastNest, 120),
            (CollapseOutcome::StrangerHut, 100),
            (CollapseOutcome::SettlementBed, 10),
            (CollapseOutcome::WaysideShrine, 50),
            (CollapseOutcome::Riverbank, 30),
            (CollapseOutcome::FestivalBench, 5),
            (CollapseOutcome::Ditch, 150),
        ];

        if let Some(ally) = affinity.strongest_ally() {
            let bonus = (affinity.get(ally) * 80.0) as u32;
            match ally {
                GodName::Oltzed => {
                    weights[0].1 += bonus / 3;
                    weights[4].1 += bonus / 2;
                    weights[5].1 += bonus;
                    weights[8].1 += bonus / 2;
                    weights[9].1 = weights[9].1.saturating_sub(bonus / 2);
                }
                GodName::Keuru => {
                    weights[0].1 += bonus / 3;
                    weights[1].1 += bonus;
                    weights[2].1 = weights[2].1.saturating_sub(bonus);
                    weights[3].1 = weights[3].1.saturating_sub(bonus / 2);
                    weights[6].1 += bonus / 4;
                }
                GodName::Sampsa => {
                    weights[0].1 += bonus / 3;
                    weights[4].1 += bonus / 2;
                    weights[6].1 += bonus;
                    weights[2].1 = weights[2].1.saturating_sub(bonus / 3);
                }
                GodName::Masa => {
                    weights[0].1 += bonus / 3;
                    weights[7].1 += bonus;
                    weights[5].1 += bonus / 4;
                    weights[4].1 += bonus / 4;
                }
                GodName::Kukri => {
                    weights[0].1 += bonus / 3;
                    weights[6].1 += bonus / 2;
                    weights[7].1 += bonus;
                    weights[9].1 = weights[9].1.saturating_sub(bonus);
                }
            }
        }

        if let Some(grudge) = affinity.strongest_grudge() {
            let penalty = (affinity.get(grudge).abs() * 60.0) as u32;
            match grudge {
                GodName::Oltzed => {
                    weights[9].1 += penalty;
                    weights[5].1 = weights[5].1.saturating_sub(penalty / 2);
                    weights[8].1 = weights[8].1.saturating_sub(penalty / 3);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
                GodName::Keuru => {
                    weights[2].1 += penalty;
                    weights[3].1 += penalty / 2;
                    weights[9].1 += penalty / 3;
                    weights[1].1 = weights[1].1.saturating_sub(penalty);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
                GodName::Sampsa => {
                    weights[9].1 += penalty / 2;
                    weights[4].1 = weights[4].1.saturating_sub(penalty / 2);
                    weights[6].1 = weights[6].1.saturating_sub(penalty / 3);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
                GodName::Masa => {
                    weights[9].1 += penalty / 2;
                    weights[7].1 = weights[7].1.saturating_sub(penalty / 2);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
                GodName::Kukri => {
                    weights[2].1 += penalty;
                    weights[9].1 += penalty / 2;
                    weights[6].1 = weights[6].1.saturating_sub(penalty / 2);
                    weights[0].1 = weights[0].1.saturating_sub(2);
                }
            }
        }

        if local_rep >= 0.7 {
            weights[5].1 += 60;
            weights[8].1 += 30;
            weights[9].1 = weights[9].1.saturating_sub(40);
        } else if local_rep >= 0.4 {
            weights[4].1 += 30;
            weights[5].1 += 20;
        } else if local_rep <= 0.15 {
            weights[9].1 += 40;
            weights[4].1 = weights[4].1.saturating_sub(20);
        }

        if affinity.get(GodName::Keuru) > 0.5 {
            weights[1].1 += 60;
            weights[2].1 = weights[2].1.saturating_sub(30);
        }

        if affinity.get(GodName::Masa) > 0.4 {
            weights[7].1 += 40;
        }

        let total: u32 = weights.iter().map(|(_, w)| *w).sum();
        let pick = if total > 0 { val % total } else { val % 1000 };
        let mut acc = 0u32;
        let mut chosen = CollapseOutcome::Ditch;
        for (outcome, weight) in &weights {
            acc += weight;
            if pick < acc {
                chosen = *outcome;
                break;
            }
        }

        let rescued_by = if chosen == CollapseOutcome::GodCampsite {
            affinity.strongest_ally()
        } else if chosen == CollapseOutcome::BeastGuarded {
            Some(GodName::Keuru)
        } else if chosen == CollapseOutcome::Riverbank {
            Some(GodName::Masa)
        } else if chosen == CollapseOutcome::FestivalBench
            || chosen == CollapseOutcome::SettlementBed
        {
            Some(GodName::Oltzed)
        } else if chosen == CollapseOutcome::WaysideShrine {
            Some(GodName::Kukri)
        } else {
            None
        };

        Collapse {
            outcome: chosen,
            died,
            rescued_by,
        }
    }

    pub fn roll_biased(
        seed: u64,
        affinity: &GodAffinity,
        local_rep: f64,
        effective_bias: f64,
    ) -> Self {
        let mut result = Self::roll(seed, affinity, local_rep);
        if effective_bias < -0.15 {
            result.outcome = match result.outcome {
                CollapseOutcome::StrangerHut => CollapseOutcome::Ditch,
                CollapseOutcome::SettlementBed => CollapseOutcome::Ditch,
                CollapseOutcome::FestivalBench => CollapseOutcome::Ditch,
                other => other,
            };
        } else if effective_bias > 0.05 {
            result.outcome = match result.outcome {
                CollapseOutcome::Ditch => CollapseOutcome::StrangerHut,
                other => other,
            };
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterKind {
    Wildlife,
    Bandit,
    Traveler,
    Storm,
    GodShrine,
    AncientRuin,
    HermitCamp,
    TravelingBard,
    SpringBloom,
    HarvestMarket,
    WinterSurvivor,
    MerchantCaravan,
    RiverFlood,
    Mirage,
    CaveIn,
    FuneralProcession,
    LostChild,
    EscapedLivestock,
    PlagueWagon,
    PilgrimBand,
    BeastMigration,
    DistantFire,
    BorderWatch,
    AuroraVeil,
    /// A Khör rendezvous — the cold-endurance steppe folk, the first non-human
    /// people in the world. They barter härkä goods for metal, take no coin,
    /// and do not haggle (#443).
    KhorTrader,
    /// A Mëräk exchange at the tideline — the Thalassian deep-sea people. They
    /// barter deep-water goods (fish, deep-glass) for surface make (cloth,
    /// tools); no coin, fixed seasonal measure (#445).
    MerakTrader,
    /// A Tzäkhar wayhold at a cave-mouth — the deep-stone people, master
    /// smiths of the Vaskiluuri. They give worked metal (iron, tools) for
    /// surface food; no coin, no haggle (#447).
    TzakharTrader,
    /// A Häl canopy-trade lowered to the forest floor — the shadow-people of
    /// the high green. They give canopy medicine and fruit for surface make
    /// (cloth, tools); no coin (#447).
    HalTrader,
    /// A She'ar meet at the desert's edge — the people who endure the heat's
    /// silence. They give desert game and succulent-physic for cloth and tools
    /// that ward the sun; no coin, the rate is the rate (#447).
    ShearTrader,
}

impl EncounterKind {
    pub fn description(self) -> &'static str {
        match self {
            EncounterKind::Wildlife => "A wild creature blocks your path!",
            EncounterKind::Bandit => "A bandit demands your coin!",
            EncounterKind::Traveler => "A friendly traveler shares news.",
            EncounterKind::Storm => "A sudden storm forces you to take shelter!",
            EncounterKind::GodShrine => "A weathered shrine stands alone. The five carved figures watch silently.",
            EncounterKind::AncientRuin => "Crumbling walls and faded murals — remnants of a forgotten age.",
            EncounterKind::HermitCamp => "A lone figure tends a small fire, watching the road with knowing eyes.",
            EncounterKind::TravelingBard => "Music drifts from the road. A bard with a cracked lute plays for coin and company.",
            EncounterKind::SpringBloom => "The thaw has come. Flowers push through the last frost, and the air tastes of renewal.",
            EncounterKind::HarvestMarket => "A market sprawls across the square — traders, food, noise, color, life.",
            EncounterKind::WinterSurvivor => "A huddled figure in the snow. They hold out a hand, half-frozen, half-hoping.",
            EncounterKind::MerchantCaravan => "A merchant caravan moves slowly along the road, pack animals laden with goods.",
            EncounterKind::RiverFlood => "The ford is gone — brown water moves fast over where the path was.",
            EncounterKind::Mirage => "Heat bends the horizon. Something shimmers there: water, walls, welcome. Maybe.",
            EncounterKind::CaveIn => "Dust drifts from the ceiling. The stone overhead groans once and goes quiet.",
            EncounterKind::FuneralProcession => "A slow line of mourners crosses the way, bearing someone home for the last time.",
            EncounterKind::LostChild => "A child stands alone at the path's edge, too tired to cry anymore.",
            EncounterKind::EscapedLivestock => "A panicked animal crashes about, trailing a broken rope — someone's livelihood running loose.",
            EncounterKind::PlagueWagon => "A covered wagon with a white-daubed rail. The driver waves you back, not in greeting.",
            EncounterKind::PilgrimBand => "Pilgrims in road-grey walk in step, singing low. They nod as they pass.",
            EncounterKind::BeastMigration => "The herd is moving — hundreds of beasts in a river of hide and breath, crossing your road.",
            EncounterKind::DistantFire => "A point of firelight in the dark distance. Someone's hearth, or someone's trouble.",
            EncounterKind::BorderWatch => "Spears at the waymark. The watch wants to know your name and your business.",
            EncounterKind::AuroraVeil => "The night sky splits into slow green fire. Even the wind stops to watch.",
            EncounterKind::KhorTrader => "Khör traders wait by a cairn hung with härkä-leather and steppe-butter — broad, cold-skinned, unhurried. They deal in goods, not coin, and they do not haggle.",
            EncounterKind::MerakTrader => "Mëräk surface at the tideline, deep-folk laying out cold deep-fish and water-smoothed deep-glass on the wet stones. They take cloth and tools, never coin, and the rate is the rate.",
            EncounterKind::TzakharTrader => "Tzäkhar stand at a cave-mouth, short and broad and patient, worked iron and fine tools set out on a slab of stone. They have no use for coin — they want surface food, and they do not hurry.",
            EncounterKind::HalTrader => "A Häl trade-party has come down from the canopy, shadow-skinned and quick-eyed, laying out salve, fruit, and bundled herbs. They want cloth and tools from the surface; coin means nothing in the high green.",
            EncounterKind::ShearTrader => "She'ar wait in the long shade at the desert's edge, still and unhurried, desert game and succulent-physic spread on a cloth. They want weave and tools to ward the sun. No coin. The rate is the rate.",
        }
    }

    pub fn is_hostile(self) -> bool {
        matches!(self, EncounterKind::Wildlife | EncounterKind::Bandit)
    }

    pub fn is_rare(self) -> bool {
        matches!(
            self,
            EncounterKind::GodShrine
                | EncounterKind::AncientRuin
                | EncounterKind::HermitCamp
                | EncounterKind::TravelingBard
        )
    }

    pub fn is_seasonal(self) -> bool {
        matches!(
            self,
            EncounterKind::SpringBloom
                | EncounterKind::HarvestMarket
                | EncounterKind::WinterSurvivor
                | EncounterKind::AuroraVeil
        )
    }

    pub fn can_have_outside_help(self) -> bool {
        matches!(
            self,
            EncounterKind::Wildlife | EncounterKind::Bandit | EncounterKind::BorderWatch
        )
    }

    pub fn available_actions(self) -> Vec<EncounterAction> {
        match self {
            EncounterKind::Wildlife => vec![EncounterAction::Flee, EncounterAction::Calm],
            EncounterKind::Bandit => vec![
                EncounterAction::Flee,
                EncounterAction::Bribe,
                EncounterAction::Intimidate,
            ],
            EncounterKind::Traveler => vec![EncounterAction::Talk, EncounterAction::Trade],
            EncounterKind::Storm => vec![EncounterAction::Shelter, EncounterAction::PushThrough],
            EncounterKind::GodShrine => vec![EncounterAction::Talk, EncounterAction::Calm],
            EncounterKind::AncientRuin => vec![EncounterAction::Calm, EncounterAction::PushThrough],
            EncounterKind::HermitCamp => vec![EncounterAction::Talk, EncounterAction::Trade],
            EncounterKind::TravelingBard => vec![EncounterAction::Talk, EncounterAction::Trade],
            EncounterKind::SpringBloom => vec![EncounterAction::Talk, EncounterAction::Calm],
            EncounterKind::HarvestMarket => vec![EncounterAction::Trade, EncounterAction::Talk],
            EncounterKind::WinterSurvivor => vec![EncounterAction::Calm, EncounterAction::Trade],
            EncounterKind::MerchantCaravan => vec![EncounterAction::Trade, EncounterAction::Calm],
            EncounterKind::RiverFlood => {
                vec![EncounterAction::Shelter, EncounterAction::PushThrough]
            }
            EncounterKind::Mirage => vec![EncounterAction::Calm, EncounterAction::PushThrough],
            EncounterKind::CaveIn => vec![EncounterAction::Flee, EncounterAction::PushThrough],
            EncounterKind::FuneralProcession => {
                vec![EncounterAction::Calm, EncounterAction::Talk]
            }
            EncounterKind::LostChild => vec![EncounterAction::Talk, EncounterAction::Calm],
            EncounterKind::EscapedLivestock => {
                vec![EncounterAction::Calm, EncounterAction::Trade]
            }
            EncounterKind::PlagueWagon => vec![EncounterAction::Flee, EncounterAction::Talk],
            EncounterKind::PilgrimBand => vec![EncounterAction::Talk, EncounterAction::Trade],
            EncounterKind::BeastMigration => vec![
                EncounterAction::Shelter,
                EncounterAction::Calm,
                EncounterAction::PushThrough,
            ],
            EncounterKind::DistantFire => vec![EncounterAction::Talk, EncounterAction::Shelter],
            EncounterKind::BorderWatch => vec![
                EncounterAction::Talk,
                EncounterAction::Bribe,
                EncounterAction::Intimidate,
            ],
            EncounterKind::AuroraVeil => vec![EncounterAction::Calm, EncounterAction::Talk],
            // The Khör barter (Trade) or you leave (Flee) — no talk, no bribe.
            EncounterKind::KhorTrader => vec![EncounterAction::Trade, EncounterAction::Flee],
            EncounterKind::MerakTrader => vec![EncounterAction::Trade, EncounterAction::Flee],
            EncounterKind::TzakharTrader => vec![EncounterAction::Trade, EncounterAction::Flee],
            EncounterKind::HalTrader => vec![EncounterAction::Trade, EncounterAction::Flee],
            EncounterKind::ShearTrader => vec![EncounterAction::Trade, EncounterAction::Flee],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterAction {
    Flee,
    Bribe,
    Calm,
    Intimidate,
    Talk,
    Trade,
    Shelter,
    PushThrough,
    /// Take a tame-enough creature for hide and meat. Only offered for danger
    /// 0/1 wildlife — a bear is a fight, not a harvest.
    Hunt,
}

impl EncounterAction {
    pub fn label(self) -> &'static str {
        match self {
            EncounterAction::Flee => "Flee",
            EncounterAction::Bribe => "Bribe (2 coins)",
            EncounterAction::Calm => "Calm the beast",
            EncounterAction::Intimidate => "Intimidate",
            EncounterAction::Talk => "Talk",
            EncounterAction::Trade => "Trade info",
            EncounterAction::Shelter => "Take shelter (1h)",
            EncounterAction::PushThrough => "Push through",
            EncounterAction::Hunt => "Hunt it (hide + meat)",
        }
    }

    pub fn key(self) -> char {
        match self {
            EncounterAction::Flee => 'f',
            EncounterAction::Bribe => 'b',
            EncounterAction::Calm => 'c',
            EncounterAction::Intimidate => 'i',
            EncounterAction::Talk => 't',
            EncounterAction::Trade => 'r',
            EncounterAction::Shelter => 's',
            EncounterAction::PushThrough => 'p',
            EncounterAction::Hunt => 'h',
        }
    }

    pub fn coin_cost(self) -> u32 {
        match self {
            EncounterAction::Bribe => 2,
            _ => 0,
        }
    }

    pub fn energy_cost(self) -> f64 {
        match self {
            EncounterAction::Flee => 0.15,
            EncounterAction::PushThrough => 0.2,
            EncounterAction::Hunt => 0.2,
            EncounterAction::Intimidate => 0.1,
            _ => 0.0,
        }
    }

    pub fn hunger_cost(self) -> f64 {
        match self {
            EncounterAction::PushThrough => 0.1,
            EncounterAction::Hunt => 0.1,
            EncounterAction::Flee => 0.05,
            _ => 0.0,
        }
    }

    pub fn hours(self) -> u32 {
        match self {
            EncounterAction::Shelter => 1,
            EncounterAction::Talk => 1,
            EncounterAction::Trade => 1,
            EncounterAction::Calm => 1,
            EncounterAction::Hunt => 2,
            _ => 0,
        }
    }

    pub fn god_affinity_effect(self) -> Option<(GodName, f64)> {
        match self {
            EncounterAction::Calm => Some((GodName::Keuru, 0.05)),
            EncounterAction::Intimidate => Some((GodName::Oltzed, -0.02)),
            EncounterAction::Talk => Some((GodName::Masa, 0.03)),
            EncounterAction::Trade => Some((GodName::Masa, 0.04)),
            EncounterAction::Bribe => Some((GodName::Masa, -0.01)),
            // Taking a creature leans away from the forest-keeper's favour.
            EncounterAction::Hunt => Some((GodName::Keuru, -0.02)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encounter {
    pub kind: EncounterKind,
    pub terrain: Terrain,
    /// For Wildlife: which animal it actually is (terrain- and season-true).
    #[serde(default)]
    pub species: Option<crate::model::wildlife::WildSpecies>,
}

impl Encounter {
    /// The actions on offer, including any that depend on the encounter's
    /// particulars (not just its kind). A tame-enough wild creature can be
    /// hunted; a dangerous one cannot — it is a fight, not a harvest.
    pub fn available_actions(&self) -> Vec<EncounterAction> {
        let mut actions = self.kind.available_actions();
        if self.kind == EncounterKind::Wildlife {
            if let Some(sp) = self.species {
                if sp.danger() <= 1 {
                    actions.push(EncounterAction::Hunt);
                }
            }
        }
        actions
    }

    pub fn roll(terrain: Terrain, hour: u32, day: u32, seed: u64) -> Option<Self> {
        Self::roll_biased(terrain, hour, day, seed, None)
    }

    pub fn roll_biased(
        terrain: Terrain,
        hour: u32,
        day: u32,
        seed: u64,
        player_people: Option<PeopleKind>,
    ) -> Option<Self> {
        Self::roll_biased_weather(terrain, hour, day, seed, player_people, 1.0)
    }

    /// Like `roll_biased`, but the baseline encounter chance is scaled by a
    /// weather multiplier (a storm stirs the land; clear skies quiet it).
    /// Weather was generated but never fed into encounter spawning.
    pub fn roll_biased_weather(
        terrain: Terrain,
        hour: u32,
        day: u32,
        seed: u64,
        player_people: Option<PeopleKind>,
        weather_mult: f64,
    ) -> Option<Self> {
        let season = Season::from_day(day);
        let hash = seed.wrapping_mul(2654435761)
            ^ (terrain as u64).wrapping_mul(40503)
            ^ (hour as u64).wrapping_mul(92000);
        let val = hash % 100;

        // Settlements have no random encounters
        if matches!(terrain, Terrain::Settlement) {
            return None;
        }

        // Rare encounters: 3% base chance, terrain-gated
        let rare_hash = hash.wrapping_mul(7919);
        let rare_val = rare_hash % 100;
        if rare_val < 3 {
            let alt = (rare_hash / 100).is_multiple_of(2);
            let rare_kind = match terrain {
                Terrain::Forest => {
                    // Three-way: ruin, distant fire, or a Häl canopy-trade down
                    // on the forest floor (#447).
                    match (rare_hash / 100) % 3 {
                        0 => EncounterKind::AncientRuin,
                        1 => EncounterKind::HalTrader,
                        _ => EncounterKind::DistantFire,
                    }
                }
                Terrain::Mountain => {
                    if alt {
                        EncounterKind::KhorTrader
                    } else {
                        EncounterKind::CaveIn
                    }
                }
                Terrain::Tundra => {
                    if alt {
                        EncounterKind::KhorTrader
                    } else {
                        EncounterKind::WinterSurvivor
                    }
                }
                Terrain::Cave => {
                    if alt {
                        EncounterKind::TzakharTrader
                    } else {
                        EncounterKind::CaveIn
                    }
                }
                Terrain::Swamp => {
                    if alt {
                        EncounterKind::HermitCamp
                    } else {
                        EncounterKind::DistantFire
                    }
                }
                Terrain::Coast => {
                    if alt {
                        EncounterKind::MerakTrader
                    } else {
                        EncounterKind::PilgrimBand
                    }
                }
                Terrain::Road => {
                    if alt {
                        EncounterKind::TravelingBard
                    } else {
                        EncounterKind::PilgrimBand
                    }
                }
                Terrain::Sand | Terrain::DeepDesert => {
                    if alt {
                        EncounterKind::ShearTrader
                    } else {
                        EncounterKind::GodShrine
                    }
                }
                _ => {
                    if alt {
                        EncounterKind::GodShrine
                    } else {
                        EncounterKind::FuneralProcession
                    }
                }
            };
            return Some(Encounter {
                kind: rare_kind,
                terrain,
                species: None,
            });
        }

        // Seasonal encounters
        let season_val = (hash.wrapping_mul(104729)) % 100;
        match season {
            Season::Thaw if season_val < 5 => {
                return Some(Encounter {
                    kind: EncounterKind::SpringBloom,
                    terrain,
                    species: None,
                });
            }
            // Meltwater: thaw floods the low ground.
            Season::Thaw
                if (5..8).contains(&season_val)
                    && matches!(terrain, Terrain::Grass | Terrain::Farmland) =>
            {
                return Some(Encounter {
                    kind: EncounterKind::RiverFlood,
                    terrain,
                    species: None,
                });
            }
            Season::Frost if season_val < 6 && matches!(terrain, Terrain::Tundra) => {
                return Some(Encounter {
                    kind: EncounterKind::AuroraVeil,
                    terrain,
                    species: None,
                });
            }
            Season::Green
                if season_val < 8
                    && matches!(terrain, Terrain::Road | Terrain::Grass | Terrain::Farmland) =>
            {
                return Some(Encounter {
                    kind: EncounterKind::HarvestMarket,
                    terrain,
                    species: None,
                });
            }
            Season::Frost if season_val < 4 => {
                return Some(Encounter {
                    kind: EncounterKind::WinterSurvivor,
                    terrain,
                    species: None,
                });
            }
            _ => {}
        }

        let (mut threshold, mut kind): (u32, EncounterKind) = match terrain {
            Terrain::Forest => (
                25,
                if val.is_multiple_of(2) {
                    EncounterKind::Wildlife
                } else {
                    EncounterKind::Bandit
                },
            ),
            Terrain::Mountain => (15, EncounterKind::Storm),
            Terrain::Swamp => (20, EncounterKind::Wildlife),
            Terrain::Sand => (
                10,
                if val.is_multiple_of(3) {
                    EncounterKind::Mirage
                } else {
                    EncounterKind::Storm
                },
            ),
            Terrain::DeepDesert => (
                12,
                if val.is_multiple_of(3) {
                    EncounterKind::Mirage
                } else {
                    EncounterKind::Storm
                },
            ),
            // Roads carry trade: every third road encounter is a merchant
            // caravan (the kind existed but was never spawned anywhere).
            Terrain::Road => (
                5,
                match val % 6 {
                    0 | 1 => EncounterKind::MerchantCaravan,
                    2 => EncounterKind::BorderWatch,
                    3 => EncounterKind::PlagueWagon,
                    _ => EncounterKind::Traveler,
                },
            ),
            Terrain::Settlement => (0, EncounterKind::Traveler),
            Terrain::Cave => (18, EncounterKind::Wildlife),
            Terrain::Tundra => (
                15,
                if val.is_multiple_of(3) {
                    EncounterKind::BeastMigration
                } else {
                    EncounterKind::Storm
                },
            ),
            Terrain::Coast => (8, EncounterKind::Traveler),
            // Open country: livestock loose, children lost, beasts about.
            Terrain::Grass | Terrain::Farmland => (
                8,
                match val % 4 {
                    0 => EncounterKind::EscapedLivestock,
                    1 => EncounterKind::LostChild,
                    2 => EncounterKind::Traveler,
                    _ => EncounterKind::Wildlife,
                },
            ),
            _ => (8, EncounterKind::Wildlife),
        };
        if let Some(pp) = player_people {
            match (pp, terrain) {
                (PeopleKind::Metsik, Terrain::Forest) => {
                    threshold = threshold.saturating_sub(5);
                    kind = EncounterKind::Wildlife;
                }
                (PeopleKind::Vayla, Terrain::Road) => {
                    threshold = threshold.saturating_add(5);
                    kind = EncounterKind::Traveler;
                }
                (PeopleKind::Sepat, Terrain::Mountain) => {
                    threshold = threshold.saturating_sub(5);
                }
                (PeopleKind::Ahjo, Terrain::Grass | Terrain::Farmland) => {
                    threshold = threshold.saturating_sub(3);
                    kind = EncounterKind::Traveler;
                }
                (PeopleKind::Laakso, Terrain::Swamp) => {
                    threshold = threshold.saturating_sub(4);
                    kind = EncounterKind::Wildlife;
                }
                (PeopleKind::Tzakhar, Terrain::Cave) => {
                    threshold = threshold.saturating_sub(6);
                    kind = EncounterKind::Wildlife;
                }
                (PeopleKind::Merak, Terrain::Coast) => {
                    threshold = threshold.saturating_sub(4);
                    kind = EncounterKind::Traveler;
                }
                (PeopleKind::Khor, Terrain::Tundra) => {
                    threshold = threshold.saturating_sub(5);
                }
                (PeopleKind::Shear, Terrain::Sand | Terrain::DeepDesert) => {
                    threshold = threshold.saturating_sub(4);
                }
                (PeopleKind::Hal, Terrain::Forest) => {
                    threshold = threshold.saturating_sub(4);
                    kind = EncounterKind::Wildlife;
                }
                _ => {}
            }
        }
        let threshold = (threshold as f64 * weather_mult.clamp(0.0, 3.0)).round() as u64;
        if (val % 100) < threshold {
            Some(Encounter {
                kind,
                terrain,
                species: None,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterLogEntry {
    pub day: u32,
    pub hour: u32,
    pub kind: EncounterKind,
    pub terrain: Terrain,
    pub action: EncounterAction,
    pub hostile: bool,
}

const ENCOUNTER_LOG_CAP: usize = 20;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EncounterLog {
    entries: Vec<EncounterLogEntry>,
}

impl EncounterLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push(&mut self, entry: EncounterLogEntry) {
        self.entries.push(entry);
        if self.entries.len() > ENCOUNTER_LOG_CAP {
            let drop = self.entries.len() - ENCOUNTER_LOG_CAP;
            self.entries.drain(0..drop);
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, EncounterLogEntry> {
        self.entries.iter()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::wildlife::WildSpecies;

    #[test]
    fn hunt_offered_only_for_tame_wildlife() {
        let tame = Encounter {
            kind: EncounterKind::Wildlife,
            terrain: Terrain::Forest,
            species: Some(WildSpecies::Hare),
        };
        assert!(
            tame.available_actions().contains(&EncounterAction::Hunt),
            "a hare should be huntable"
        );
        let fierce = Encounter {
            kind: EncounterKind::Wildlife,
            terrain: Terrain::Forest,
            species: Some(WildSpecies::BrownBear),
        };
        assert!(
            !fierce.available_actions().contains(&EncounterAction::Hunt),
            "a bear is a fight, not a hunt"
        );
        // A non-wildlife encounter never offers Hunt.
        let traveler = Encounter {
            kind: EncounterKind::Traveler,
            terrain: Terrain::Road,
            species: None,
        };
        assert!(!traveler
            .available_actions()
            .contains(&EncounterAction::Hunt));
    }

    #[test]

    fn encounter_forest_higher_chance() {
        let mut encounters = 0;

        for seed in 0..100u64 {
            if Encounter::roll(Terrain::Forest, 10, 1, seed).is_some() {
                encounters += 1;
            }
        }

        assert!(
            encounters > 10,
            "forest should have ~25% encounter rate, got {}/100",
            encounters
        );
    }

    #[test]

    fn encounter_settlement_none() {
        let mut encounters = 0;

        for seed in 0..100u64 {
            if Encounter::roll(Terrain::Settlement, 10, 1, seed).is_some() {
                encounters += 1;
            }
        }

        assert_eq!(encounters, 0, "settlements should never have encounters");
    }

    #[test]

    fn encounter_deterministic() {
        for seed in 0..50u64 {
            let a = Encounter::roll(Terrain::Forest, 10, 1, seed);

            let b = Encounter::roll(Terrain::Forest, 10, 1, seed);

            assert_eq!(a, b, "encounter roll must be deterministic");
        }
    }

    #[test]

    fn encounter_hostile_kinds() {
        assert!(EncounterKind::Wildlife.is_hostile());

        assert!(EncounterKind::Bandit.is_hostile());

        assert!(!EncounterKind::Traveler.is_hostile());

        assert!(!EncounterKind::Storm.is_hostile());
    }

    #[test]

    fn collapse_roll_deterministic() {
        let ga = GodAffinity::new();

        for seed in 0..50u64 {
            let a = Collapse::roll(seed, &ga, 0.0);

            let b = Collapse::roll(seed, &ga, 0.0);

            assert_eq!(a.outcome, b.outcome);

            assert_eq!(a.died, b.died);
        }
    }

    #[test]

    fn collapse_high_rep_prefer_settlement() {
        let ga = GodAffinity::new();

        let mut settlement_count = 0u32;

        let mut ditch_count = 0u32;

        for seed in 0..200u64 {
            let c = Collapse::roll(seed, &ga, 0.8);

            if matches!(
                c.outcome,
                CollapseOutcome::SettlementBed | CollapseOutcome::FestivalBench
            ) {
                settlement_count += 1;
            }

            if matches!(c.outcome, CollapseOutcome::Ditch) {
                ditch_count += 1;
            }
        }

        assert!(settlement_count > 5, "high rep should get more safe beds");

        assert!(ditch_count < 50, "high rep should avoid ditches");
    }

    #[test]

    fn collapse_metsik_ally_beast_guarded() {
        let mut ga = GodAffinity::new();

        ga.adjust(GodName::Keuru, 0.8);

        let mut guarded = 0u32;

        let mut hostile = 0u32;

        for seed in 0..200u64 {
            let c = Collapse::roll(seed, &ga, 0.0);

            if matches!(c.outcome, CollapseOutcome::BeastGuarded) {
                guarded += 1;
            }

            if matches!(c.outcome, CollapseOutcome::HostileBeast) {
                hostile += 1;
            }
        }

        assert!(guarded > 5, "Keuru ally should get beast-guarded more");

        assert!(hostile < 30, "Keuru ally should avoid hostile beasts");
    }

    #[test]

    fn collapse_grudge_more_hostile() {
        let mut ga = GodAffinity::new();

        ga.adjust(GodName::Keuru, -0.8);

        let mut hostile = 0u32;

        for seed in 0..200u64 {
            let c = Collapse::roll(seed, &ga, 0.0);

            if matches!(
                c.outcome,
                CollapseOutcome::HostileBeast | CollapseOutcome::Ditch
            ) {
                hostile += 1;
            }
        }

        assert!(
            hostile > 40,
            "Keuru grudge should cause more hostile outcomes"
        );
    }

    #[test]

    fn collapse_biased_hostile_downgrades_safe() {
        let ga = GodAffinity::new();

        let mut ditch_from_safe = 0u32;

        for seed in 0..200u64 {
            let c = Collapse::roll_biased(
                seed,
                &ga,
                0.5,
                PeopleKind::Metsik.bias_toward(PeopleKind::Sepat),
            );

            if matches!(c.outcome, CollapseOutcome::Ditch) {
                ditch_from_safe += 1;
            }
        }

        let mut ditch_neutral = 0u32;

        for seed in 0..200u64 {
            let c = Collapse::roll_biased(
                seed,
                &ga,
                0.5,
                PeopleKind::Arkit.bias_toward(PeopleKind::Arkit),
            );

            if matches!(c.outcome, CollapseOutcome::Ditch) {
                ditch_neutral += 1;
            }
        }

        assert!(
            ditch_from_safe >= ditch_neutral,
            "hostile bias should produce more ditches: {ditch_from_safe} vs {ditch_neutral}"
        );
    }

    #[test]

    fn collapse_biased_ally_upgrades_ditch() {
        let ga = GodAffinity::new();

        let mut hut_from_ditch = 0u32;

        for seed in 0..200u64 {
            let c = Collapse::roll_biased(
                seed,
                &ga,
                0.3,
                PeopleKind::Metsik.bias_toward(PeopleKind::Metsik),
            );

            if matches!(c.outcome, CollapseOutcome::StrangerHut) {
                hut_from_ditch += 1;
            }
        }

        assert!(
            hut_from_ditch > 0,
            "ally bias should upgrade some ditches to StrangerHut"
        );
    }

    #[test]

    fn collapse_outcome_properties() {
        assert!(CollapseOutcome::BeastGuarded.is_beast_aided());

        assert!(!CollapseOutcome::BeastGuarded.is_hostile());

        assert!(CollapseOutcome::HostileBeast.is_hostile());

        assert!(CollapseOutcome::SettlementBed.is_safe());

        assert!(CollapseOutcome::GodCampsite.is_divine());

        assert_eq!(CollapseOutcome::GodCampsite.glyph(), '✦');
    }

    #[test]

    fn encounter_actions_per_kind() {
        let wildlife = EncounterKind::Wildlife.available_actions();

        assert!(wildlife.contains(&EncounterAction::Flee));

        assert!(wildlife.contains(&EncounterAction::Calm));

        let bandit = EncounterKind::Bandit.available_actions();

        assert!(bandit.contains(&EncounterAction::Bribe));

        assert!(bandit.contains(&EncounterAction::Intimidate));

        let traveler = EncounterKind::Traveler.available_actions();

        assert!(traveler.contains(&EncounterAction::Talk));

        assert!(traveler.contains(&EncounterAction::Trade));

        let storm = EncounterKind::Storm.available_actions();

        assert!(storm.contains(&EncounterAction::Shelter));

        assert!(storm.contains(&EncounterAction::PushThrough));
    }

    #[test]

    fn encounter_action_costs() {
        assert_eq!(EncounterAction::Bribe.coin_cost(), 2);

        assert_eq!(EncounterAction::Flee.coin_cost(), 0);

        assert!((EncounterAction::Flee.energy_cost() - 0.15).abs() < f64::EPSILON);

        assert!((EncounterAction::PushThrough.energy_cost() - 0.2).abs() < f64::EPSILON);
    }

    #[test]

    fn encounter_action_god_effects() {
        assert_eq!(
            EncounterAction::Calm.god_affinity_effect(),
            Some((GodName::Keuru, 0.05))
        );

        assert_eq!(
            EncounterAction::Talk.god_affinity_effect(),
            Some((GodName::Masa, 0.03))
        );

        assert_eq!(EncounterAction::Flee.god_affinity_effect(), None);
    }

    #[test]

    fn encounter_log_starts_empty() {
        let log = EncounterLog::new();

        assert_eq!(log.len(), 0);

        assert!(log.is_empty());
    }

    #[test]

    fn encounter_log_pushes_in_order() {
        let mut log = EncounterLog::new();

        for i in 0..5 {
            log.push(EncounterLogEntry {
                day: i,

                hour: 0,

                kind: EncounterKind::Wildlife,

                terrain: Terrain::Forest,

                action: EncounterAction::Flee,

                hostile: true,
            });
        }

        assert_eq!(log.len(), 5);

        let days: Vec<u32> = log.iter().map(|e| e.day).collect();

        assert_eq!(days, vec![0, 1, 2, 3, 4]);
    }

    #[test]

    fn encounter_log_caps_at_twenty() {
        let mut log = EncounterLog::new();

        for i in 0..35 {
            log.push(EncounterLogEntry {
                day: i,

                hour: 0,

                kind: EncounterKind::Wildlife,

                terrain: Terrain::Forest,

                action: EncounterAction::Flee,

                hostile: true,
            });
        }

        assert_eq!(log.len(), 20);

        let first = log.iter().next().map(|e| e.day);

        let last = log.iter().next_back().map(|e| e.day);

        assert_eq!(first, Some(15));

        assert_eq!(last, Some(34));
    }

    #[test]

    fn starvation_when_hunger_zero() {
        let vitals = PlayerVitals {
            hunger: 0.0,

            thirst: 0.5,

            energy: 0.8,
        };

        let cause = DeathCause::from_collapse_and_vitals(CollapseOutcome::Ditch, vitals);

        assert_eq!(cause, DeathCause::Starvation);
    }

    #[test]

    fn exhaustion_when_energy_zero() {
        let vitals = PlayerVitals {
            hunger: 0.5,

            thirst: 0.5,

            energy: 0.01,
        };

        let cause = DeathCause::from_collapse_and_vitals(CollapseOutcome::Ditch, vitals);

        assert_eq!(cause, DeathCause::Exhaustion);
    }

    #[test]

    fn wounds_from_hostile_collapse() {
        let vitals = PlayerVitals {
            hunger: 0.8,

            thirst: 0.8,

            energy: 0.8,
        };

        let cause = DeathCause::from_collapse_and_vitals(CollapseOutcome::HostileBeast, vitals);

        assert_eq!(cause, DeathCause::Wounds);
    }

    #[test]

    fn unknown_from_safe_collapse() {
        let vitals = PlayerVitals {
            hunger: 0.8,

            thirst: 0.8,

            energy: 0.8,
        };

        let cause = DeathCause::from_collapse_and_vitals(CollapseOutcome::StrangerHut, vitals);

        assert_eq!(cause, DeathCause::Unknown);
    }

    #[test]

    fn death_cause_flavors() {
        assert!(!DeathCause::Starvation.flavor().is_empty());

        assert!(!DeathCause::Exposure.flavor().is_empty());

        assert!(!DeathCause::Exhaustion.flavor().is_empty());

        assert!(!DeathCause::Wounds.flavor().is_empty());

        assert!(!DeathCause::Unknown.flavor().is_empty());
    }
}
