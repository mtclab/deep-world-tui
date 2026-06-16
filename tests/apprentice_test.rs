// Apprentice to a master crafter to learn their craft-sense (#527/#529): give
// days and board, gain a lesser taught echo of the sense. Only the giftless can,
// and only one in a life.
use deep_world_tui::model::{ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app(seed: u64) -> App {
    let charts = deep_world_tui::charts::load::load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    // Ensure the player is giftless (apprenticeship is for the ungifted).
    a.gift = Default::default();
    a
}

/// Find a settlement holding a gifted crafter; stand the player there.
fn find_a_master(a: &mut App) -> Option<(usize, usize, usize)> {
    let regions = a.sim.as_ref().unwrap().world.regions.len();
    for ri in 0..regions {
        let found = {
            let region = &a.sim.as_ref().unwrap().world.regions[ri];
            region.settlements.iter().enumerate().find_map(|(si, s)| {
                s.people
                    .iter()
                    .position(|p| p.gift.has())
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

fn find_master_across_seeds() -> (App, usize, usize, usize) {
    for seed in 0..60u64 {
        let mut a = app(seed);
        if let Some((ri, si, pi)) = find_a_master(&mut a) {
            return (a, ri, si, pi);
        }
    }
    panic!("no gifted crafter found across seeds");
}

#[test]
fn apprenticing_teaches_the_masters_sense() {
    let (mut a, ri, si, pi) = find_master_across_seeds();
    let sense = {
        let s = &a.sim.as_ref().unwrap().world.regions[ri].settlements[si];
        s.people[pi].gift.sense().unwrap()
    };
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 10);
    assert!(a.learned_sense.is_none());
    a.apprentice_to(ri, si, pi);
    assert_eq!(
        a.learned_sense,
        Some(sense),
        "the master's sense is learned"
    );
    assert!(
        a.player_start
            .as_ref()
            .unwrap()
            .inventory
            .get(ItemType::Food)
            < 10,
        "board is paid in food"
    );
}

#[test]
fn the_gifted_cannot_apprentice() {
    let (mut a, ri, si, pi) = find_master_across_seeds();
    // Give the player their own gift — they cannot take another's sense.
    let masters_gift = {
        let s = &a.sim.as_ref().unwrap().world.regions[ri].settlements[si];
        s.people[pi].gift
    };
    a.gift = masters_gift;
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 10);
    a.apprentice_to(ri, si, pi);
    assert!(
        a.learned_sense.is_none(),
        "the gifted learn no second sense"
    );
}

#[test]
fn only_one_apprenticeship_in_a_life() {
    let (mut a, ri, si, pi) = find_master_across_seeds();
    a.player_start
        .as_mut()
        .unwrap()
        .inventory
        .add(ItemType::Food, 20);
    a.apprentice_to(ri, si, pi);
    let first = a.learned_sense;
    assert!(first.is_some());
    // A second attempt is refused; the learned sense does not change.
    a.apprentice_to(ri, si, pi);
    assert_eq!(
        a.learned_sense, first,
        "a life holds only one learned craft"
    );
}
