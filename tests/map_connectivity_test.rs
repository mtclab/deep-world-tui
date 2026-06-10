use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::Terrain;
use deep_world_tui::sim::SimState;
use std::collections::VecDeque;

// Flood-fill passable tiles from the first settlement; assert every settlement
// tile is reachable (no region split by an uncrossable river/coast).
#[test]
fn every_settlement_reachable_from_spawn() {
    let charts = load_charts().expect("charts");
    for seed in [1u64, 7, 42, 99, 123] {
        let sim = SimState::new(seed, charts.clone());
        for region in &sim.world.regions {
            let w = region.terrain.width;
            let h = region.terrain.height;
            let t = &region.terrain.tiles;
            let settlements: Vec<usize> = (0..t.len())
                .filter(|&i| t[i] == Terrain::Settlement)
                .collect();
            if settlements.is_empty() {
                continue;
            }
            // BFS from first settlement over passable tiles
            let mut seen = vec![false; t.len()];
            let mut q = VecDeque::new();
            q.push_back(settlements[0]);
            seen[settlements[0]] = true;
            while let Some(i) = q.pop_front() {
                let (x, y) = (i % w, i / w);
                let mut nbrs = vec![];
                if x > 0 {
                    nbrs.push(i - 1);
                }
                if x + 1 < w {
                    nbrs.push(i + 1);
                }
                if y > 0 {
                    nbrs.push(i - w);
                }
                if y + 1 < h {
                    nbrs.push(i + w);
                }
                for n in nbrs {
                    if !seen[n] && t[n].passable() {
                        seen[n] = true;
                        q.push_back(n);
                    }
                }
            }
            for &s in &settlements {
                assert!(
                    seen[s],
                    "seed {seed}: a settlement is walled off (unreachable)"
                );
            }
        }
    }
}
