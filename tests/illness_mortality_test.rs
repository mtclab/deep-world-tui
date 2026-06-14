// Disease as the great leveller of the post-Fall age (#449). Before, a player's
// illness only sped vitals decay and never killed; a fed, careful life reached
// old age ~96% of the time. Now an untreated fever, a plague, a wound gone bad
// can take you — gentler when fed/sheltered/healed, deadlier starving, leaned by
// the life's hidden star. Treatment is the counter, never immunity.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::{ActiveDisease, Disease};
use deep_world_tui::model::{DeathCause, PlayerPos};
use deep_world_tui::ui::app::App;

#[test]
fn the_acute_killers_outrank_the_chronic() {
    // Plague, childbirth, venom, an infected wound bite hardest; sprains and
    // exhaustion do not kill in their own right.
    assert!(Disease::Plague.daily_mortality() > Disease::Infection.daily_mortality());
    assert!(Disease::Infection.daily_mortality() > Disease::Fever.daily_mortality());
    assert!(Disease::Fever.daily_mortality() > Disease::IronAche.daily_mortality());
    assert_eq!(Disease::Sprain.daily_mortality(), 0.0);
    assert_eq!(Disease::Exhaustion.daily_mortality(), 0.0);
}

/// Run a life that already carries the plague, in the open with no healer, and
/// see whether the sickness takes them before it runs its course. `hunger` sets
/// how fed they are each day — the only thing varied — so the test can show that
/// a fed body outlasts disease more often than a starving one. Returns whether
/// the plague killed: a death that hands off to an heir clears `death_cause` but
/// leaves a lineage record (cause = "sickness"), so we read that, not the live
/// field.
fn died_of_plague(seed: u64, hunger: f64) -> bool {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.lifespan_years = 9999; // isolate disease from old age
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    // Anchor the illness to the live sim tick so it does not read as already
    // "recovered" (recovery is contracted_tick + recovery_ticks).
    let now = a.sim.as_ref().map_or(0, |s| s.world.tick);
    if let Some(ref mut ps) = a.player_start {
        ps.person.illnesses.clear();
        ps.person
            .illnesses
            .push(ActiveDisease::new(Disease::Plague, now));
    }
    // Step a handful of days. Re-top the chosen vitals each dawn and cross the
    // midnight boundary with only a couple of hours of decay (hour 23 -> +2), so
    // the body never starves out from under the test — the only thing killing is
    // the disease's own daily mortality roll.
    let sickness = DeathCause::Sickness.label();
    for _ in 0..10 {
        a.vitals.hunger = hunger;
        a.vitals.energy = 0.8;
        a.vitals.thirst = 0.8;
        let now = a.sim.as_ref().map_or(0, |s| s.world.tick);
        if let Some(ref mut ps) = a.player_start {
            if ps.person.illnesses.is_empty() {
                ps.person
                    .illnesses
                    .push(ActiveDisease::new(Disease::Plague, now));
            }
        }
        a.clock.hour = 23;
        a.advance_clock(2); // cross the day boundary -> one mortality roll
        if a.death_cause == Some(DeathCause::Sickness)
            || a.lineage.iter().any(|r| r.cause == sickness)
        {
            return true;
        }
    }
    false
}

#[test]
fn an_untreated_plague_can_kill() {
    let deaths = (0..240u64).filter(|&s| died_of_plague(s, 0.5)).count();
    assert!(
        deaths > 12,
        "an untreated plague in the open should claim a real share of lives, got {deaths}/240"
    );
}

#[test]
fn the_fed_outlast_the_sickness_more_than_the_starving() {
    let fed = (0..240u64).filter(|&s| died_of_plague(s, 0.85)).count();
    let starving = (0..240u64).filter(|&s| died_of_plague(s, 0.15)).count();
    assert!(
        starving > fed,
        "starving should die of the plague more than the fed: starving {starving} vs fed {fed}"
    );
}
