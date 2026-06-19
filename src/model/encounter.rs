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

#[cfg(test)]
mod tests {
    use super::*;

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
