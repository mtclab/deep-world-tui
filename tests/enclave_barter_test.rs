// The enclave trade floor (#454): the Five take no coin. At one of their
// settlements, laying down a good barters it for theirs at a fixed rate; trying
// to buy with coin is refused.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{economy::enclave_barter, ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

/// Find a world with an enclave and stand the player in it. Returns the
/// enclave's people and a good they will take.
fn stand_in_an_enclave(a: &mut App) -> Option<(deep_world_tui::model::PeopleKind, ItemType)> {
    for ri in 0..a.sim.as_ref().unwrap().world.regions.len() {
        let found = {
            let region = &a.sim.as_ref().unwrap().world.regions[ri];
            region.settlements.iter().find_map(|s| {
                s.enclave_people()
                    .map(|pk| (pk, s.map_x as usize, s.map_y as usize))
            })
        };
        if let Some((pk, mx, my)) = found {
            a.player_pos = Some(PlayerPos {
                region_idx: ri,
                px: mx,
                py: my,
            });
            // A good this people will take.
            let offered = [
                ItemType::Tool,
                ItemType::Cloth,
                ItemType::Food,
                ItemType::Herb,
                ItemType::Iron,
            ]
            .into_iter()
            .find(|&it| enclave_barter(pk, it).is_some())?;
            return Some((pk, offered));
        }
    }
    None
}

#[test]
fn the_floor_barters_in_kind_and_refuses_coin() {
    // Search a few seeds for a world that has an enclave.
    let mut a = app(42);
    let mut deal = stand_in_an_enclave(&mut a);
    for seed in [7u64, 100, 2024, 555, 1, 9] {
        if deal.is_some() {
            break;
        }
        a = app(seed);
        deal = stand_in_an_enclave(&mut a);
    }
    let Some((pk, offered)) = deal else {
        panic!("no enclave found across seeds — the Five should hold one somewhere");
    };
    let (cost, gives) = enclave_barter(pk, offered).unwrap();

    // Stock the offered good and some coin.
    {
        let inv = &mut a.player_start.as_mut().unwrap().inventory;
        inv.add(offered, cost + 2);
        inv.add(ItemType::Coin, 50);
    }
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    let offered_before = a.player_start.as_ref().unwrap().inventory.get(offered);
    let gain_before: Vec<u32> = gives
        .iter()
        .map(|(it, _)| a.player_start.as_ref().unwrap().inventory.get(*it))
        .collect();

    // Buying with coin is refused — no coin moves.
    a.buy_item(ItemType::Bandage);
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Coin),
        coin_before,
        "the Five take no coin"
    );

    // Laying down the good barters it in kind.
    a.sell_item(offered);
    let inv = &a.player_start.as_ref().unwrap().inventory;
    assert_eq!(
        inv.get(ItemType::Coin),
        coin_before,
        "no coin changes hands at an enclave"
    );
    assert_eq!(
        inv.get(offered),
        offered_before - cost,
        "the offered good is given up"
    );
    for (i, (it, qty)) in gives.iter().enumerate() {
        // (offered may equal a received good in theory; these tables don't, so
        // a plain delta check holds.)
        assert_eq!(
            inv.get(*it),
            gain_before[i] + qty,
            "received {qty} {it:?} in fair measure"
        );
    }
}
