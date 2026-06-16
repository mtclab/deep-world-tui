// Consult a settlement's healer for your own sickness (#527 tradespeople): the
// temple's cure, out where there is no temple. A healer cures; a herbalist eases.
use deep_world_tui::model::economy::ActiveDisease;
use deep_world_tui::model::{Disease, ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app(seed: u64) -> App {
    let charts = deep_world_tui::charts::load::load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a
}

/// Find a settlement with a healer or herbalist; stand the player there.
fn find_a_healer(a: &mut App) -> Option<(usize, usize, usize, bool)> {
    let regions = a.sim.as_ref().unwrap().world.regions.len();
    for ri in 0..regions {
        let found = {
            let region = &a.sim.as_ref().unwrap().world.regions[ri];
            region.settlements.iter().enumerate().find_map(|(si, s)| {
                s.people
                    .iter()
                    .position(|p| p.profession == "healer" || p.profession == "herbalist")
                    .map(|pi| {
                        let is_healer = s.people[pi].profession == "healer";
                        (si, pi, s.map_x as usize, s.map_y as usize, is_healer)
                    })
            })
        };
        if let Some((si, pi, mx, my, is_healer)) = found {
            a.player_pos = Some(PlayerPos {
                region_idx: ri,
                px: mx,
                py: my,
            });
            return Some((ri, si, pi, is_healer));
        }
    }
    None
}

fn afflict(a: &mut App) {
    let tick = a.sim.as_ref().unwrap().world.tick;
    a.player_start
        .as_mut()
        .unwrap()
        .person
        .illnesses
        .push(ActiveDisease::new(Disease::Fever, tick));
}

#[test]
fn a_healer_cures_what_ails_you_for_a_fee() {
    let mut a = app(42);
    let mut deal = find_a_healer(&mut a);
    for seed in [7u64, 100, 555, 2024, 1, 9, 13, 21] {
        if deal.is_some() {
            break;
        }
        a = app(seed);
        deal = find_a_healer(&mut a);
    }
    let Some((ri, si, pi, is_healer)) = deal else {
        panic!("no healer/herbalist found across seeds");
    };
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 30);
    afflict(&mut a);
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);

    a.consult_healer(ri, si, pi);

    let ps = a.player_start.as_ref().unwrap();
    assert!(
        ps.inventory.get(ItemType::Coin) < coin_before,
        "the fee is paid"
    );
    if is_healer {
        assert!(
            ps.person.illnesses.is_empty(),
            "a healer cures the sickness outright"
        );
    } else {
        // A herbalist eases (severity floored), not necessarily clears.
        assert!(
            !ps.person.illnesses.is_empty(),
            "a herbalist eases but does not cure"
        );
    }
}

#[test]
fn the_hale_need_no_healer() {
    let mut a = app(42);
    let Some((ri, si, pi, _)) = find_a_healer(&mut a).or_else(|| {
        for seed in [7u64, 100, 555, 2024, 1, 9, 13, 21] {
            a = app(seed);
            if let Some(d) = find_a_healer(&mut a) {
                return Some(d);
            }
        }
        None
    }) else {
        panic!("no healer");
    };
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 30);
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    a.consult_healer(ri, si, pi);
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Coin),
        coin_before,
        "no fee taken from the hale"
    );
}
