// NPCs carry the gift too (#441): rare gifted crafters live in settlements and
// their craft-goods are truer (cheaper) where they work.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::person::generate_person;
use deep_world_tui::model::economy::ItemType;
use deep_world_tui::model::{CraftSense, Gift, PlayerPos};
use deep_world_tui::rng::SeedRng;
use deep_world_tui::ui::app::{App, Screen};

#[test]
fn npcs_are_gifted_at_the_ordinary_rare_rate() {
    let charts = load_charts().expect("charts");
    let n = 6000;
    let mut gifted = 0;
    for s in 0..n {
        let mut rng = SeedRng::new(s as u64);
        if generate_person(&mut rng, &charts).gift.has() {
            gifted += 1;
        }
    }
    let rate = gifted as f64 / n as f64;
    assert!(
        (0.012..0.04).contains(&rate),
        "npc gift rate off: {rate} ({gifted}/{n})"
    );
}

#[test]
fn an_iron_ear_smith_in_town_makes_truer_tools() {
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
    // Resolve the town directly so the market reads its people.
    a.screen = Screen::Location {
        region_idx: 0,
        settlement_idx: 0,
        scroll: 0,
    };
    let before = a.quote_buy_price(ItemType::Tool);
    // Drop a gifted iron-ear smith into the current settlement.
    let placed = {
        if let Some(sim) = a.sim.as_mut() {
            if let Some(region) = sim.world.regions.get_mut(0) {
                if let Some(s) = region.settlements.first_mut() {
                    if let Some(p) = s.people.first_mut() {
                        p.gift = Gift::of(CraftSense::IronEar);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    };
    assert!(placed, "test needs a settlement with people");
    let after = a.quote_buy_price(ItemType::Tool);
    assert!(
        after < before,
        "an iron-ear smith should cheapen tools: {before} -> {after}"
    );
}
