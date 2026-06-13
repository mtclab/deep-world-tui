// Luck in the everyday. The hidden star is not only for wounds and death — it
// leans the small things too: the fortunate find a little more in the gathering
// and strike a slightly better bargain, the cursed a little less. Bounded, and
// consistent across a life (the star does not change hour to hour).
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{Fortune, ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app(seed: u64, fortune: f64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.fortune = Fortune::from_value(fortune);
    a
}

// Stand the player on any tile that yields on gather, at midday.
fn on_gatherable(a: &mut App) -> bool {
    let found = {
        let terr = &a.sim.as_ref().unwrap().world.regions[0].terrain;
        let mut hit = None;
        'o: for y in 0..terr.height {
            for x in 0..terr.width {
                if terr.get(x, y).and_then(ItemType::gather_from).is_some() {
                    hit = Some((x, y));
                    break 'o;
                }
            }
        }
        hit
    };
    if let Some((x, y)) = found {
        a.player_pos = Some(PlayerPos {
            region_idx: 0,
            px: x,
            py: y,
        });
        a.clock.hour = 12;
        true
    } else {
        false
    }
}

#[test]
fn the_fortunate_gather_more_over_a_life() {
    fn total_gathered(fortune: f64) -> u32 {
        let mut a = app(7, fortune);
        if !on_gatherable(&mut a) {
            return 0;
        }
        // Make the land productive: high summer, clear skies — so the base
        // haul is non-zero and the fortune lean has something to lean on.
        a.clock.day = 45; // Green season in the 90-day year
        for r in a.sim.as_mut().unwrap().world.regions.iter_mut() {
            r.weather = deep_world_tui::model::Weather::Clear;
        }
        // The item this tile yields — count exactly it.
        let pos = a.player_pos.unwrap();
        let item = a.sim.as_ref().unwrap().world.regions[0]
            .terrain
            .get(pos.px, pos.py)
            .and_then(ItemType::gather_from)
            .expect("gatherable");
        let mut total = 0u32;
        for tick in 0..2000u64 {
            // Re-pin the life: a gather-induced collapse would swap in an heir
            // with a fresh star and drift the measurement.
            a.fortune = Fortune::from_value(fortune);
            a.vitals.energy = 1.0;
            a.vitals.hunger = 1.0;
            a.vitals.thirst = 1.0;
            a.collapse = None;
            a.death_cause = None;
            a.sim.as_mut().unwrap().world.tick = tick;
            a.clock.hour = 12; // keep the light
            let before = a.player_inventory().get(item);
            a.gather();
            let after = a.player_inventory().get(item);
            total += after.saturating_sub(before);
        }
        total
    }
    let blessed = total_gathered(1.0);
    let cursed = total_gathered(-1.0);
    assert!(
        blessed > 0 && cursed > 0,
        "gathering yields under either star"
    );
    assert!(
        blessed > cursed,
        "the fortunate find more over a life ({blessed} vs {cursed})"
    );
}

#[test]
fn the_fortunate_strike_a_better_bargain() {
    // Same town, same goods — only the star differs. The blessed buy no dearer
    // (usually cheaper) and sell no cheaper than the cursed.
    let mut blessed = app(7, 1.0);
    let mut cursed = app(7, -1.0);
    // Put both at the same settlement so the market context matches.
    for a in [&mut blessed, &mut cursed] {
        let pos = PlayerPos {
            region_idx: 0,
            px: 0,
            py: 0,
        };
        a.player_pos = Some(pos);
    }
    let item = ItemType::Food;
    let b_buy = blessed.quote_buy_price(item);
    let c_buy = cursed.quote_buy_price(item);
    let b_sell = blessed.quote_sell_price(item);
    let c_sell = cursed.quote_sell_price(item);
    assert!(
        b_buy <= c_buy,
        "the blessed buy no dearer ({b_buy} vs {c_buy})"
    );
    assert!(
        b_sell >= c_sell,
        "the blessed sell no cheaper ({b_sell} vs {c_sell})"
    );
    // And the luck is a lean, not a windfall: buy still exceeds sell (no
    // infinite-coin loop).
    assert!(b_buy > b_sell, "the spread holds for the blessed too");
}
