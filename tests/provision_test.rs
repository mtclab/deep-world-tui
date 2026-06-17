// Provision a settlement (#526 livelihoods / #540): carry in a good the town
// lacks; they pay a premium, their stores rise, and they remember you.
use deep_world_tui::model::{ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app() -> App {
    let charts = deep_world_tui::charts::load::load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let (mx, my) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[0];
        (s.map_x as usize, s.map_y as usize)
    };
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: mx,
        py: my,
    });
    a
}

#[test]
fn provisioning_a_short_good_pays_and_stocks_the_town() {
    let mut a = app();
    let (_ri, si) = a.player_on_settlement().expect("town");
    // Drain every tracked good so the town is short of all of them; the
    // shortfall is then well-defined and the player can fill it.
    for it in [
        ItemType::Iron,
        ItemType::Tool,
        ItemType::Cloth,
        ItemType::Wood,
    ] {
        a.sim.as_mut().unwrap().world.regions[0].settlements[si]
            .goods_stock
            .insert(it, 0.0);
    }
    let want = a.settlement_shortfall().expect("a shortfall");
    a.player_start.as_mut().unwrap().inventory.add(want, 5);
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    let town_before = a.sim.as_ref().unwrap().world.regions[0].settlements[si].good(want);

    a.provision_settlement();

    let ps = a.player_start.as_ref().unwrap();
    assert!(
        ps.inventory.get(ItemType::Coin) > coin_before,
        "paid for the goods"
    );
    assert!(ps.inventory.get(want) < 5, "the goods were handed over");
    let town_after = a.sim.as_ref().unwrap().world.regions[0].settlements[si].good(want);
    assert!(town_after > town_before, "the town's stores actually rose");
}

#[test]
fn keeping_a_town_supplied_lifts_its_traders() {
    // A trade-runner who provisions a town again and again shifts who holds its
    // council toward the Traders (#565 the player's mark).
    let mut a = app();
    let (_ri, si) = a.player_on_settlement().expect("town");
    let traders_before = a.sim.as_ref().unwrap().world.regions[0].settlements[si]
        .politics
        .standing(deep_world_tui::model::economy::Faction::Traders);
    for _ in 0..20 {
        // Keep the town short of everything and the pack full, so each call
        // delivers.
        for it in [
            ItemType::Iron,
            ItemType::Tool,
            ItemType::Cloth,
            ItemType::Wood,
        ] {
            a.sim.as_mut().unwrap().world.regions[0].settlements[si]
                .goods_stock
                .insert(it, 0.0);
        }
        if let Some(want) = a.settlement_shortfall() {
            a.player_start.as_mut().unwrap().inventory.add(want, 5);
            a.provision_settlement();
        }
    }
    let traders_after = a.sim.as_ref().unwrap().world.regions[0].settlements[si]
        .politics
        .standing(deep_world_tui::model::economy::Faction::Traders);
    assert!(
        traders_after > traders_before,
        "repeated provisioning lifted the Traders ({traders_before} -> {traders_after})"
    );
}

#[test]
fn carrying_goods_between_rivals_thaws_the_feud() {
    // #565 slice 3: provisioning a town that is a rival of the last town you
    // supplied eases their bad blood — the player is the cart that crosses where
    // no town's own cart will.
    let mut a = app();
    let (_ri, si) = a.player_on_settlement().expect("town");
    let here = a.sim.as_ref().unwrap().world.regions[0].settlements[si]
        .name
        .clone();
    let rival = "Oldgrudge".to_string();
    a.sim
        .as_mut()
        .unwrap()
        .province_ties
        .nudge(&here, &rival, -0.5); // sworn rivals
    let before = a.sim.as_ref().unwrap().province_ties.bond(&here, &rival);
    // The player just came from the rival town.
    a.sim.as_mut().unwrap().last_provisioned_town = Some(rival.clone());
    for it in [
        ItemType::Iron,
        ItemType::Tool,
        ItemType::Cloth,
        ItemType::Wood,
    ] {
        a.sim.as_mut().unwrap().world.regions[0].settlements[si]
            .goods_stock
            .insert(it, 0.0);
    }
    let want = a.settlement_shortfall().expect("a shortfall");
    a.player_start.as_mut().unwrap().inventory.add(want, 5);
    a.provision_settlement();
    let after = a.sim.as_ref().unwrap().province_ties.bond(&here, &rival);
    assert!(
        after > before,
        "the rivalry eased toward neutral ({before} -> {after})"
    );
}

#[test]
fn nothing_to_provision_if_you_carry_none() {
    let mut a = app();
    let (_ri, si) = a.player_on_settlement().expect("town");
    for it in [
        ItemType::Iron,
        ItemType::Tool,
        ItemType::Cloth,
        ItemType::Wood,
    ] {
        a.sim.as_mut().unwrap().world.regions[0].settlements[si]
            .goods_stock
            .insert(it, 0.0);
    }
    // Player carries none of the tracked goods.
    for it in [
        ItemType::Iron,
        ItemType::Tool,
        ItemType::Cloth,
        ItemType::Wood,
    ] {
        let n = a.player_start.as_ref().unwrap().inventory.get(it);
        a.player_start.as_mut().unwrap().inventory.remove(it, n);
    }
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    a.provision_settlement();
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Coin),
        coin_before,
        "no pay when you bring nothing"
    );
}
