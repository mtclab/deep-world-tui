// Crime & justice (#8): getting caught stealing no longer stays a local matter
// — word runs the roads ahead of you and your welcome cools across the region.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{ItemType, PlayerPos};
use deep_world_tui::ui::app::App;

fn app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.clock.hour = 8;
    a
}

#[test]
fn a_caught_theft_cools_welcome_across_the_region() {
    let mut found = false;
    for seed in 0..80u64 {
        let mut a = app(seed);
        // A region with at least two settlements, so "word travels" has somewhere to go.
        let pick = {
            let sim = a.sim.as_ref().unwrap();
            sim.world.regions.iter().enumerate().find_map(|(ri, r)| {
                if r.settlements.len() >= 2 {
                    let others: Vec<String> =
                        r.settlements.iter().skip(1).map(|s| s.id.clone()).collect();
                    let (mx, my) = (
                        r.settlements[0].map_x as usize,
                        r.settlements[0].map_y as usize,
                    );
                    Some((ri, mx, my, others))
                } else {
                    None
                }
            })
        };
        let Some((ri, mx, my, others)) = pick else {
            continue;
        };
        a.player_pos = Some(PlayerPos {
            region_idx: ri,
            px: mx,
            py: my,
        });
        let pid = a.player_start.as_ref().unwrap().person.id.clone();

        a.steal_item(ItemType::Iron);

        // Only the "caught" outcome spreads the word.
        if a.status_msg
            .clone()
            .unwrap_or_default()
            .contains("travel the region")
        {
            let sim = a.sim.as_ref().unwrap();
            for oid in &others {
                assert!(
                    sim.reputation.get(&pid, oid) < 0.5,
                    "word of the theft cooled the welcome at {oid}"
                );
            }
            found = true;
            break;
        }
    }
    assert!(
        found,
        "expected a caught theft across 80 seeds (test setup)"
    );
}
