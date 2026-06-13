// Fleeing is a gamble, not an exit (#397). Caution is not immunity: a creature
// that stands its ground or hunts gets a say when you turn your back, and you
// never know your luck. A guardian shortens the odds — never to zero. The
// worst roll (run-down) empties you and hands you to the collapse funnel.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::companion::{Animal, Companion};
use deep_world_tui::model::wildlife::WildSpecies;
use deep_world_tui::model::{Encounter, EncounterAction, EncounterKind, PlayerPos, Terrain};
use deep_world_tui::ui::app::App;

fn app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    if let Some(ps) = a.player_start.as_mut() {
        ps.companions.clear();
    }
    a
}

fn set_encounter(a: &mut App, species: WildSpecies) {
    a.encounter = Some(Encounter {
        kind: EncounterKind::Wildlife,
        terrain: Terrain::Forest,
        species: Some(species),
    });
}

#[derive(Default)]
struct Tally {
    clean: u32,
    wounded: u32,
    run_down: u32,
}

// Sweep many turns against `species`, classifying each flee. Reuses one app
// (the roll depends only on seed/tick/danger/guard), resetting state per turn.
fn sweep(seed: u64, species: WildSpecies, guarded: bool, n: u64) -> Tally {
    let mut a = app(seed);
    if guarded {
        if let Some(ps) = a.player_start.as_mut() {
            ps.companions
                .push(Companion::new(Animal::Dog, "Guard".into(), 0));
        }
    }
    let mut t = Tally::default();
    for tick in 0..n {
        a.sim.as_mut().unwrap().world.tick = tick;
        a.vitals.energy = 1.0;
        a.vitals.hunger = 1.0;
        a.vitals.thirst = 1.0;
        a.collapse = None;
        let before = a.collapses_had;
        set_encounter(&mut a, species);
        a.resolve_encounter(EncounterAction::Flee);
        if a.collapses_had > before {
            t.run_down += 1; // emptied → collapse funnel
        } else if a.vitals.energy < 0.8 {
            t.wounded += 1; // a gash (clean flee costs only 0.15)
        } else {
            t.clean += 1;
        }
    }
    t
}

#[test]
fn fleeing_a_creature_that_flees_on_sight_is_always_safe() {
    assert_eq!(WildSpecies::Hare.danger(), 0);
    let t = sweep(99, WildSpecies::Hare, false, 1500);
    assert_eq!(t.wounded, 0, "a hare never wounds you");
    assert_eq!(t.run_down, 0, "and never runs you down");
}

#[test]
fn fleeing_a_predator_sometimes_costs_blood_or_worse() {
    assert_eq!(WildSpecies::Wolf.danger(), 2);
    let t = sweep(7, WildSpecies::Wolf, false, 2000);
    assert!(
        t.wounded > 0,
        "the wolf draws blood sometimes ({})",
        t.wounded
    );
    assert!(
        t.run_down > 0,
        "and sometimes runs you down ({})",
        t.run_down
    );
    // …but most flights still succeed: a gamble, not a death sentence.
    assert!(
        t.clean > (t.wounded + t.run_down),
        "most flights are still clean ({} vs {}+{})",
        t.clean,
        t.wounded,
        t.run_down
    );
}

#[test]
fn the_same_turn_always_resolves_the_same_way() {
    // Two single-turn sweeps at matched ticks must agree exactly.
    let a = sweep(7, WildSpecies::Wolf, false, 200);
    let b = sweep(7, WildSpecies::Wolf, false, 200);
    assert_eq!(a.wounded, b.wounded);
    assert_eq!(a.run_down, b.run_down);
}

#[test]
fn a_guardian_shortens_the_odds_but_never_to_zero() {
    let bare = sweep(7, WildSpecies::Wolf, false, 2000);
    let guarded = sweep(7, WildSpecies::Wolf, true, 2000);
    let bare_hits = bare.wounded + bare.run_down;
    let guarded_hits = guarded.wounded + guarded.run_down;
    assert!(
        guarded_hits < bare_hits,
        "a guardian shortens the odds ({guarded_hits} vs {bare_hits})"
    );
    assert!(
        guarded_hits > 0,
        "but a guard is not a wall ({guarded_hits})"
    );
}

#[test]
fn run_down_feeds_the_collapse_funnel() {
    // The cautious traveler who fled the rumored road meets a beast on the
    // "safer" one — and being run down hands them to the same funnel that
    // hunger and cold do. Over a sweep, that funnel sometimes does not let go.
    let mut a = app(13);
    let mut deaths = 0;
    let mut run_downs = 0;
    for tick in 0..4000u64 {
        if a.death_cause.is_some() {
            // continue_as_npc already swapped the body; reset for measurement.
            a = app(13 + tick);
        }
        a.sim.as_mut().unwrap().world.tick = tick;
        a.vitals.energy = 0.2; // worn thin, far from any bed
        a.vitals.hunger = 0.3;
        let before = a.collapses_had;
        set_encounter(&mut a, WildSpecies::BrownBear);
        a.resolve_encounter(EncounterAction::Flee);
        if a.collapses_had > before {
            run_downs += 1;
            if a.death_cause.is_some() {
                deaths += 1;
            }
        }
    }
    assert!(run_downs > 0, "bears run down the worn ({run_downs})");
    assert!(
        deaths > 0,
        "you never know your luck — the wilds claim some ({deaths} of {run_downs})"
    );
}
