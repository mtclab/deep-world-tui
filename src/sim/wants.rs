use crate::model::{NpcActivity, NpcSchedule};
use crate::rng::SeedRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WantKind {
    FindFood,
    FindWater,
    TalkToFriend,
    GoToTemple,
    RepairRoof,
    FleeFrost,
    MournSomeone,
    FindApprentice,
    PayDebt,
    HideFromBandit,
}

impl WantKind {
    pub fn label(self) -> &'static str {
        match self {
            WantKind::FindFood => "foraging",
            WantKind::FindWater => "drawing water",
            WantKind::TalkToFriend => "visiting",
            WantKind::GoToTemple => "praying",
            WantKind::RepairRoof => "repairing",
            WantKind::FleeFrost => "fleeing frost",
            WantKind::MournSomeone => "mourning",
            WantKind::FindApprentice => "seeking apprentice",
            WantKind::PayDebt => "settling debt",
            WantKind::HideFromBandit => "hiding",
        }
    }

    pub fn activity(self) -> NpcActivity {
        match self {
            WantKind::FindFood | WantKind::FindWater => NpcActivity::Travel,
            WantKind::TalkToFriend => NpcActivity::Socialize,
            WantKind::GoToTemple => NpcActivity::Worship,
            WantKind::RepairRoof | WantKind::FindApprentice => NpcActivity::Craft,
            WantKind::FleeFrost | WantKind::HideFromBandit => NpcActivity::Idle,
            WantKind::MournSomeone => NpcActivity::Worship,
            WantKind::PayDebt => NpcActivity::Travel,
        }
    }

    fn people_weights(people: &str) -> &'static [(WantKind, u32)] {
        use crate::model::PeopleKind as P;
        // Normalize through PeopleKind::from_name — chart people strings are
        // lowercase ("metsik", "vayla", ...) but this used to match display
        // names with diacritics ("Sepät", "Väylä"), so NO people ever matched
        // and every NPC fell through to the generic default wants.
        match P::from_name(people) {
            P::Metsik | P::Metsareunat | P::Koskimetsa | P::Hal => {
                &[(WantKind::FindFood, 4), (WantKind::TalkToFriend, 3)]
            }
            P::Sepat | P::Ahjo | P::Takovaki => {
                &[(WantKind::RepairRoof, 4), (WantKind::FindApprentice, 3)]
            }
            P::Arkit | P::Muistikansa | P::Taulukansa | P::Kirjakansa => {
                &[(WantKind::GoToTemple, 4), (WantKind::PayDebt, 3)]
            }
            P::Vayla | P::Rantavaki | P::Saarivaki | P::Hiekkakavelijat => {
                &[(WantKind::FindFood, 3), (WantKind::FindWater, 4)]
            }
            P::Laakso | P::Haramaki | P::Jamavaki | P::Pohjavaki => {
                &[(WantKind::MournSomeone, 4), (WantKind::GoToTemple, 3)]
            }
            P::Shear => &[(WantKind::TalkToFriend, 3), (WantKind::HideFromBandit, 4)],
            P::Khor | P::Porokansa => &[(WantKind::FleeFrost, 4), (WantKind::FindFood, 3)],
            P::Merak => &[(WantKind::FindWater, 4), (WantKind::PayDebt, 3)],
            P::Tzakhar => &[(WantKind::MournSomeone, 4), (WantKind::RepairRoof, 3)],
            _ => &[(WantKind::FindFood, 2), (WantKind::TalkToFriend, 2)],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WantStatus {
    Pending,
    InProgress,
    Satisfied,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpcWant {
    pub kind: WantKind,
    pub status: WantStatus,
    pub start_tick: u64,
    pub fail_after_days: u32,
}

pub fn generate_npc_wants(seed: u64, person_id: &str, people: &str) -> Vec<NpcWant> {
    let mut rng = SeedRng::new(seed).fork_for(&format!("want-gen-{person_id}"));
    let weights = WantKind::people_weights(people);
    let total: u32 = weights.iter().map(|(_, w)| *w).sum();
    let mut wants = Vec::new();
    let count = if rng.gen_range(4) < 1 { 1 } else { 2 };
    for _ in 0..count {
        let roll = rng.gen_range(total) as u32;
        let mut acc = 0u32;
        for &(kind, w) in weights {
            acc += w;
            if roll < acc {
                wants.push(NpcWant {
                    kind,
                    status: WantStatus::Pending,
                    start_tick: 0,
                    fail_after_days: 6,
                });
                break;
            }
        }
    }
    wants
}

pub fn tick_wants(
    wants: &mut [NpcWant],
    current_tick: u64,
    ticks_per_day: u64,
    is_frost_season: bool,
) {
    for want in wants.iter_mut() {
        match want.status {
            WantStatus::Pending => {
                want.status = WantStatus::InProgress;
                want.start_tick = current_tick;
            }
            WantStatus::InProgress => {
                let elapsed_days = (current_tick - want.start_tick) / ticks_per_day.max(1);
                if elapsed_days >= want.fail_after_days as u64 {
                    want.status = WantStatus::Failed;
                }
                if want.kind == WantKind::FleeFrost && !is_frost_season {
                    want.status = WantStatus::Satisfied;
                }
            }
            WantStatus::Satisfied | WantStatus::Failed => {}
        }
    }
}

pub fn recompute_schedule(
    wants: &[NpcWant],
    base_schedule: &NpcSchedule,
    hour: u32,
) -> NpcSchedule {
    let active = wants.iter().find(|w| w.status == WantStatus::InProgress);
    if let Some(want) = active {
        let activity = want.kind.activity();
        let mut blocks = base_schedule.blocks;
        let block_idx = (hour / 4) as usize;
        if block_idx < blocks.len() && !matches!(blocks[block_idx], NpcActivity::Sleep) {
            blocks[block_idx] = activity;
        }
        NpcSchedule { blocks }
    } else {
        base_schedule.clone()
    }
}

pub fn wants_label(wants: &[NpcWant]) -> Option<&'static str> {
    wants
        .iter()
        .find(|w| w.status == WantStatus::InProgress)
        .map(|w| w.kind.label())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_wants_deterministic() {
        let w1 = generate_npc_wants(42, "npc-1", "Metsik");
        let w2 = generate_npc_wants(42, "npc-1", "Metsik");
        assert_eq!(w1.len(), w2.len());
        for (a, b) in w1.iter().zip(w2.iter()) {
            assert_eq!(a.kind, b.kind);
        }
    }

    #[test]
    fn generate_wants_per_people() {
        let metsik = generate_npc_wants(1, "a", "Metsik");
        assert!(metsik
            .iter()
            .all(|w| matches!(w.kind, WantKind::FindFood | WantKind::TalkToFriend)));
        let sepat = generate_npc_wants(1, "b", "Sepät");
        assert!(sepat
            .iter()
            .all(|w| matches!(w.kind, WantKind::RepairRoof | WantKind::FindApprentice)));
    }

    #[test]
    fn tick_wants_pending_to_in_progress() {
        let mut wants = vec![NpcWant {
            kind: WantKind::FindFood,
            status: WantStatus::Pending,
            start_tick: 0,
            fail_after_days: 6,
        }];
        tick_wants(&mut wants, 10, 24, false);
        assert_eq!(wants[0].status, WantStatus::InProgress);
        assert_eq!(wants[0].start_tick, 10);
    }

    #[test]
    fn tick_wants_fail_after_days() {
        let mut wants = vec![NpcWant {
            kind: WantKind::FindFood,
            status: WantStatus::InProgress,
            start_tick: 0,
            fail_after_days: 6,
        }];
        tick_wants(&mut wants, 24 * 6, 24, false);
        assert_eq!(wants[0].status, WantStatus::Failed);
    }

    #[test]
    fn tick_wants_flee_frost_satisfied_in_summer() {
        let mut wants = vec![NpcWant {
            kind: WantKind::FleeFrost,
            status: WantStatus::InProgress,
            start_tick: 0,
            fail_after_days: 6,
        }];
        tick_wants(&mut wants, 100, 24, false);
        assert_eq!(wants[0].status, WantStatus::Satisfied);
    }

    #[test]
    fn recompute_schedule_active_want_overrides() {
        let wants = vec![NpcWant {
            kind: WantKind::GoToTemple,
            status: WantStatus::InProgress,
            start_tick: 0,
            fail_after_days: 6,
        }];
        let base = NpcSchedule::default();
        let sched = recompute_schedule(&wants, &base, 8);
        assert_eq!(sched.blocks[2], NpcActivity::Worship);
    }

    #[test]
    fn recompute_schedule_no_active_want_returns_base() {
        let wants = vec![NpcWant {
            kind: WantKind::FindFood,
            status: WantStatus::Satisfied,
            start_tick: 0,
            fail_after_days: 6,
        }];
        let base = NpcSchedule::default();
        let sched = recompute_schedule(&wants, &base, 8);
        assert_eq!(sched, base);
    }
}
