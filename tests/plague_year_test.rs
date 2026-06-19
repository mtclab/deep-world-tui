// Plague-year play (#457): in a plague year the crowds are the danger and the
// healer pays a price. Two mechanics — crowd contagion (a settlement, the safe
// haven any ordinary season, becomes the likeliest place to catch it) and the
// healer's hazard (tending the sick through a plague is how it finds the healer).
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::{ActiveDisease, Disease};
use deep_world_tui::model::{ItemType, PlayerPos, WorldEvent};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(7, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    a
}

/// Find a calendar day on which this world is in a plague year (None if the
/// seed never has one within the scan).
fn find_plague_day(a: &mut App, plague: bool) -> Option<u32> {
    for day in (1..2000).step_by(3) {
        a.clock.day = day;
        let is_plague = a.current_world_event() == Some(WorldEvent::PlagueYear);
        if is_plague == plague {
            return Some(day);
        }
    }
    None
}

/// Count how often tending a plague-sick villager infects the healer, over many
/// trials (tick varied so the fortune-leaned hazard roll varies).
fn healer_infections(seed_day: u32, a: &mut App) -> u32 {
    a.clock.day = seed_day;
    let mut infected = 0;
    for k in 0..60u64 {
        // vary the hazard roll
        // Heal whichever settlement still holds people: heal_npc steps the
        // clock, and the sim's migration may move souls off a given town's
        // roster while they walk the road (#641), so a fixed (0,0,0) town can
        // empty mid-run. Find a populated one each trial.
        let Some((ri, si)) = (if let Some(ref sim) = a.sim {
            sim.world.regions.iter().enumerate().find_map(|(ri, r)| {
                r.settlements
                    .iter()
                    .position(|s| !s.people.is_empty())
                    .map(|si| (ri, si))
            })
        } else {
            None
        }) else {
            continue;
        };
        if let Some(ref mut sim) = a.sim {
            sim.world.tick = 1000 + k * 7;
            sim.world.regions[ri].settlements[si].people[0]
                .illnesses
                .clear();
            sim.world.regions[ri].settlements[si].people[0]
                .illnesses
                .push(ActiveDisease::new(Disease::Plague, sim.world.tick));
        }
        if let Some(ref mut ps) = a.player_start {
            ps.person.illnesses.clear();
            ps.inventory.add(ItemType::Herb, 1);
        }
        a.heal_npc(ri, si, 0);
        if a.player_start
            .as_ref()
            .is_some_and(|ps| !ps.person.illnesses.is_empty())
        {
            infected += 1;
        }
    }
    infected
}

#[test]
fn the_healer_pays_in_a_plague_year_but_not_otherwise() {
    let mut a = app();
    let plague_day = find_plague_day(&mut a, true).expect("seed 7 should see a plague year");
    let calm_day = find_plague_day(&mut a, false).expect("and an ordinary year");

    let in_plague = healer_infections(plague_day, &mut a);
    let in_calm = healer_infections(calm_day, &mut a);

    assert!(
        in_plague > 0,
        "tending the sick through a plague should sometimes infect the healer (got {in_plague}/60)"
    );
    assert_eq!(
        in_calm, 0,
        "ordinary tending must be safe (got {in_calm}/60 infections in a calm year)"
    );
}

#[test]
fn a_plague_year_is_announced_and_real() {
    let mut a = app();
    // The world this seed lives in does have plague years, and the event speaks.
    let plague_day = find_plague_day(&mut a, true);
    assert!(plague_day.is_some(), "the seed should see a plague year");
    a.clock.day = plague_day.unwrap();
    assert_eq!(a.current_world_event(), Some(WorldEvent::PlagueYear));
    assert!(!WorldEvent::PlagueYear.rumor().is_empty());
}
