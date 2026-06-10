// Crime (#321) and the inter-people escalation ladder (#322): theft is a
// choice with witnesses and weight; hostility closes markets and bars gates;
// trade, festivals, and penance mend what tension breaks.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, SettlementService};
use deep_world_tui::ui::app::App;

fn town_app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 10;
    a.enter_settlement(0, 0);
    a
}

#[test]
fn theft_has_outcomes_and_consequences() {
    // Across seeds, stealing must sometimes succeed and sometimes get caught,
    // and getting caught must cost standing.
    let mut took = 0;
    let mut caught = 0;
    for seed in 0..30u64 {
        let mut a = town_app(seed);
        let pid = a.player_start.as_ref().unwrap().person.id.clone();
        let sid = a.current_settlement().unwrap().id.clone();
        let rep_before = a.sim.as_ref().unwrap().reputation.get(&pid, &sid);
        let before = a
            .player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Herb);
        a.steal_item(ItemType::Herb);
        let after = a
            .player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Herb);
        let rep_after = a.sim.as_ref().unwrap().reputation.get(&pid, &sid);
        if after > before {
            took += 1;
        } else {
            caught += 1;
            assert!(
                rep_after < rep_before,
                "getting caught must cost standing ({rep_before} -> {rep_after})"
            );
        }
    }
    assert!(took > 0, "theft should sometimes succeed");
    assert!(caught > 0, "theft should sometimes fail");
}

#[test]
fn hostility_closes_the_market_and_bars_the_gate() {
    let mut a = town_app(42);
    let people = a.current_settlement_people().expect("people");
    a.inter_people_bias.mod_toward(people, -2.0); // floor it
    a.steal_item(ItemType::Herb); // (crime independent of bias)
    a.buy_item(ItemType::Food);
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        msg.contains("closed to your kind"),
        "deep hostility should close the market, got: {msg}"
    );
    a.exit_settlement();
    a.enter_settlement(0, 0);
    let msg = a.status_msg.clone().unwrap_or_default();
    assert!(
        msg.contains("block your path") || msg.contains("not welcome"),
        "open hostility should bar the gate, got: {msg}"
    );
}

#[test]
fn trade_mends_bias() {
    let mut a = town_app(42);
    let people = a.current_settlement_people().expect("people");
    a.inter_people_bias.mod_toward(people, -0.2);
    let before = a.inter_people_bias.effective_bias(people);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 30);
    a.buy_item(ItemType::Food);
    a.buy_item(ItemType::Herb);
    let after = a.inter_people_bias.effective_bias(people);
    assert!(
        after > before,
        "honest trade should mend bias ({before} -> {after})"
    );
}

#[test]
fn penance_at_the_temple_restores_standing() {
    let mut a = town_app(42);
    let pid = a.player_start.as_ref().unwrap().person.id.clone();
    let sid = a.current_settlement().unwrap().id.clone();
    a.sim
        .as_mut()
        .unwrap()
        .reputation
        .adjust_settlement(&pid, &sid, -0.3);
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 20);
    let before = a.sim.as_ref().unwrap().reputation.get(&pid, &sid);
    a.use_service(SettlementService::Temple);
    let after = a.sim.as_ref().unwrap().reputation.get(&pid, &sid);
    assert!(
        after > before,
        "penance should restore a little standing ({before} -> {after})"
    );
}
