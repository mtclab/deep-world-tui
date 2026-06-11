// Tone fixes from the §8 canon-logic audit (#351): restitution is more Masa
// than absolution — mended standing costs a donation scaled to the offense —
// and a city has more eyes than a hamlet, so theft-witness odds scale with
// the size of the place you steal in.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, SettlementService};
use deep_world_tui::ui::app::App;

fn app_in_town() -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.enter_settlement(0, 0);
    a.clock.hour = 10;
    a
}

fn rep(a: &App) -> f64 {
    a.reputation_in_current_settlement()
}

fn lower_rep(a: &mut App, delta: f64) {
    let pid = a.player_start.as_ref().unwrap().person.id.clone();
    let sid = a.sim.as_ref().unwrap().world.regions[0].settlements[0]
        .id
        .clone();
    a.sim
        .as_mut()
        .unwrap()
        .reputation
        .adjust_local(&pid, &sid, delta);
}

#[test]
fn mended_standing_costs_a_donation() {
    let mut broke = app_in_town();
    let mut paying = app_in_town();
    for a in [&mut broke, &mut paying] {
        lower_rep(a, -0.2); // an offense on the books
        let ps = a.player_start.as_mut().unwrap();
        let c = ps.inventory.get(ItemType::Coin);
        ps.inventory.remove(ItemType::Coin, c);
    }
    // The visit alone (3 coins + nothing left for the poor-box) mends nothing.
    broke
        .player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 4);
    // Coin enough for visit AND restitution mends the ledger.
    paying
        .player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 40);
    let (b0, p0) = (rep(&broke), rep(&paying));
    broke.use_service(SettlementService::Temple);
    paying.use_service(SettlementService::Temple);
    // Reputation drifts a little on its own with the hours (spread); the
    // restitution is the 0.05 ON TOP of identical drift.
    let broke_delta = rep(&broke) - b0;
    let paying_delta = rep(&paying) - p0;
    assert!(
        paying_delta > broke_delta + 0.04,
        "restitution paid, standing mended beyond mere drift \
         (paid {paying_delta:.3} vs unpaid {broke_delta:.3})"
    );
    assert!(
        broke_delta < 0.04,
        "no donation, no absolution — drift only ({broke_delta:.3})"
    );
    assert!(
        paying
            .status_msg
            .clone()
            .unwrap_or_default()
            .contains("Restitution"),
        "the poor-box is named: {:?}",
        paying.status_msg
    );
}

#[test]
fn a_city_has_more_eyes_than_a_hamlet() {
    let mut hamlet = app_in_town();
    let mut city = app_in_town();
    hamlet.sim.as_mut().unwrap().world.regions[0].settlements[0].size = "hamlet".into();
    city.sim.as_mut().unwrap().world.regions[0].settlements[0].size = "city".into();
    // Same seed, same tick sequence — the only difference is the eyes.
    let caught = |a: &App| a.status_msg.clone().unwrap_or_default().contains("Caught");
    let clean = |a: &App| {
        a.status_msg
            .clone()
            .unwrap_or_default()
            .contains("No one saw")
    };
    let (mut h_caught, mut c_caught, mut h_clean, mut c_clean) = (0, 0, 0, 0);
    for _ in 0..40 {
        hamlet.steal_item(ItemType::Food);
        city.steal_item(ItemType::Food);
        if caught(&hamlet) {
            h_caught += 1;
        }
        if caught(&city) {
            c_caught += 1;
        }
        if clean(&hamlet) {
            h_clean += 1;
        }
        if clean(&city) {
            c_clean += 1;
        }
    }
    assert!(
        c_caught > h_caught,
        "a city street is never unwatched ({c_caught} caught vs {h_caught} in the hamlet)"
    );
    assert!(
        h_clean > c_clean,
        "a hamlet's lanes are empty half the day ({h_clean} clean vs {c_clean} in the city)"
    );
}
