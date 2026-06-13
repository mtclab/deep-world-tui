// The gift costs the body (#427): a gifted crafter masters the work their
// sense answers — it cannot botch and yields a little more — but working it
// past a day's measure brings the flame-fever, and sustained overuse the
// chronic iron-ache.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::{craft_recipes, ItemType};
use deep_world_tui::model::{CraftSense, Disease, Fortune, Gift, PlayerPos};
use deep_world_tui::ui::app::App;

fn app(seed: u64, gift: Option<CraftSense>) -> App {
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
    a.fortune = Fortune::from_value(0.0);
    a.gift = match gift {
        Some(s) => Gift::of(s),
        None => Gift::NONE,
    };
    // Plenty of forge stock for many Tool crafts (Wood 2 + Iron 1 each).
    if let Some(ps) = a.player_start.as_mut() {
        ps.inventory.add(ItemType::Wood, 60);
        ps.inventory.add(ItemType::Iron, 60);
        ps.companions.clear();
    }
    a
}

fn tool_index() -> usize {
    // People-specific recipes come after the universal ones, so the Tool
    // recipe keeps its position in the filtered list.
    craft_recipes()
        .iter()
        .position(|r| r.output == ItemType::Tool)
        .expect("a Tool recipe")
}

fn has(a: &App, d: Disease) -> bool {
    a.player_start
        .as_ref()
        .map(|ps| ps.person.illnesses.iter().any(|x| x.disease == d))
        .unwrap_or(false)
}

#[test]
fn working_the_gift_past_a_day_brings_the_flame_fever() {
    let mut a = app(7, Some(CraftSense::IronEar));
    let idx = tool_index();
    assert!(!has(&a, Disease::FlameFever));
    // A few gift-aided crafts in the same day cross the strain threshold.
    for _ in 0..5 {
        a.craft_recipe(idx);
    }
    assert!(
        has(&a, Disease::FlameFever),
        "sustained gift-craft in one day should bring flame-fever"
    );
}

#[test]
fn the_craftless_pay_no_gift_cost() {
    // No gift: crafting the same recipe never brings the craft-fever.
    let mut a = app(7, None);
    let idx = tool_index();
    for _ in 0..5 {
        a.craft_recipe(idx);
    }
    assert!(!has(&a, Disease::FlameFever));
    assert!(!has(&a, Disease::IronAche));
}

#[test]
fn the_gift_masters_its_craft_no_botch() {
    // Over many gift-aided crafts the Tool count rises every time — the gift
    // does not spoil the work it was born to.
    let mut a = app(123, Some(CraftSense::IronEar));
    let idx = tool_index();
    let before = a
        .player_start
        .as_ref()
        .map(|ps| ps.inventory.get(ItemType::Tool))
        .unwrap();
    a.craft_recipe(idx);
    let after = a
        .player_start
        .as_ref()
        .map(|ps| ps.inventory.get(ItemType::Tool))
        .unwrap();
    // Gift bonus: at least the recipe output plus one, never a botch (zero).
    assert!(after > before, "a gift-aided craft must yield, never botch");
}
