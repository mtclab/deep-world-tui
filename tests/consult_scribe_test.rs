// Consult a scribe for wider-world lore (#527 tradespeople): Sampsa's folk deal
// in knowledge — a canon fact of the continent, kept in the journal.
use deep_world_tui::model::{ItemType, PlayerPos};
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

fn find_a_scribe(a: &mut App) -> Option<(usize, usize, usize)> {
    let regions = a.sim.as_ref().unwrap().world.regions.len();
    for ri in 0..regions {
        let found = {
            let region = &a.sim.as_ref().unwrap().world.regions[ri];
            region.settlements.iter().enumerate().find_map(|(si, s)| {
                s.people
                    .iter()
                    .position(|p| p.profession == "scribe")
                    .map(|pi| (si, pi, s.map_x as usize, s.map_y as usize))
            })
        };
        if let Some((si, pi, mx, my)) = found {
            a.player_pos = Some(PlayerPos {
                region_idx: ri,
                px: mx,
                py: my,
            });
            return Some((ri, si, pi));
        }
    }
    None
}

#[test]
fn a_scribe_reads_you_lore_for_a_fee() {
    let mut a = app(42);
    let mut deal = find_a_scribe(&mut a);
    for seed in [7u64, 100, 555, 2024, 1, 9, 13, 21, 33, 77, 88, 200] {
        if deal.is_some() {
            break;
        }
        a = app(seed);
        deal = find_a_scribe(&mut a);
    }
    let Some((ri, si, pi)) = deal else {
        panic!("no scribe found across seeds");
    };
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 20);
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    let journal_before = a.sim.as_ref().unwrap().journal.entries.len();

    a.consult_scribe(ri, si, pi);

    assert!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Coin)
            < coin_before,
        "the fee is paid"
    );
    assert!(
        a.sim.as_ref().unwrap().journal.entries.len() > journal_before,
        "the lore is kept in the journal"
    );
    // Same scribe, same day → same fact.
    let first = a.status_msg.clone();
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 20);
    a.consult_scribe(ri, si, pi);
    assert_eq!(
        first, a.status_msg,
        "the scribe reads the same fact that day"
    );
}

#[test]
fn a_non_scribe_keeps_no_records() {
    let mut a = app(42);
    let (ri, si, pi) = {
        let region = &a.sim.as_ref().unwrap().world.regions[0];
        let (si, pi) = region
            .settlements
            .iter()
            .enumerate()
            .find_map(|(si, s)| {
                s.people
                    .iter()
                    .position(|p| p.profession != "scribe")
                    .map(|pi| (si, pi))
            })
            .expect("a non-scribe");
        (0usize, si, pi)
    };
    let (mx, my) = {
        let s = &a.sim.as_ref().unwrap().world.regions[0].settlements[si];
        (s.map_x as usize, s.map_y as usize)
    };
    a.player_pos = Some(PlayerPos {
        region_idx: ri,
        px: mx,
        py: my,
    });
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Coin, 20);
    let coin_before = a
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Coin);
    a.consult_scribe(ri, si, pi);
    assert_eq!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Coin),
        coin_before,
        "a non-scribe takes no fee"
    );
}
