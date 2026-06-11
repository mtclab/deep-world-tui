// Homestead → hamlet (#345): a fed homestead on wild ground draws settlers —
// rumor first, then wagons. Settlers are drawn OUT of real settlements
// (people are conserved), the hamlet is named by the region's naming
// tradition, and the founder gets standing, not naming rights.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::economy::{CropType, Farm, PlayerFarm};
use deep_world_tui::model::{ItemType, PlayerPos, Terrain};
use deep_world_tui::sim::structures::{BuildKind, Structure};
use deep_world_tui::ui::app::App;

/// An app with a player cabin + working field on wild grass (no settlement
/// within eight tiles) and a stash full enough to feed arrivals. None if the
/// seed offers no such tile.
fn wild_homestead() -> Option<(App, usize, usize, usize)> {
    let charts = load_charts().expect("charts");
    let mut a = App::new(42, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    let mut site = None;
    {
        let sim = a.sim.as_ref().unwrap();
        'o: for (ri, region) in sim.world.regions.iter().enumerate() {
            let terr = &region.terrain;
            for y in 0..terr.height {
                for x in 0..terr.width {
                    if terr.get(x, y) != Some(Terrain::Grass) {
                        continue;
                    }
                    let near_town = (y.saturating_sub(8)..(y + 9).min(terr.height)).any(|ty| {
                        (x.saturating_sub(8)..(x + 9).min(terr.width))
                            .any(|tx| terr.get(tx, ty) == Some(Terrain::Settlement))
                    });
                    if !near_town {
                        site = Some((ri, x, y));
                        break 'o;
                    }
                }
            }
        }
    }
    let (ri, x, y) = site?;
    a.player_pos = Some(PlayerPos {
        region_idx: ri,
        px: x,
        py: y,
    });
    let mut stash = deep_world_tui::model::Inventory::default();
    stash.add(ItemType::Food, 100);
    let tick = a.sim.as_ref().unwrap().world.tick;
    {
        let sim = a.sim.as_mut().unwrap();
        sim.world.regions[ri].structures.push(Structure {
            kind: BuildKind::Cabin,
            region_idx: ri,
            x: x as u32,
            y: y as u32,
            built_tick: 0,
            last_maintenance_tick: 0,
            name: None,
            is_npc_built: false,
            stash,
        });
        // Make sure the roads have people to send.
        for region in sim.world.regions.iter_mut() {
            for s in region.settlements.iter_mut() {
                s.population = s.population.max(60);
            }
        }
    }
    a.player_farms.push(PlayerFarm {
        farm: Farm::new(7, CropType::Grain, tick, Terrain::Grass),
        region_idx: ri,
        x: x as u32,
        y: y as u32,
    });
    Some((a, ri, x, y))
}

fn total_population(a: &App) -> u64 {
    a.sim
        .as_ref()
        .unwrap()
        .world
        .regions
        .iter()
        .flat_map(|r| r.settlements.iter())
        .map(|s| s.population as u64)
        .sum()
}

#[test]
fn a_fed_homestead_becomes_a_hamlet_and_people_are_conserved() {
    let Some((mut a, ri, x, y)) = wild_homestead() else {
        return; // no wild ground on this seed: vacuous
    };
    let pop_before = total_population(&a);
    let player_name = a.player_start.as_ref().unwrap().person.name.clone();
    // First check: the rumor precedes the wagons.
    a.tick_founding();
    assert!(a.homestead_rumored, "word goes out before anyone arrives");
    assert!(a.homestead_settlers.is_empty(), "rumor is not people");
    // Then the waves, until the world recognizes the place.
    let mut founded = false;
    for _ in 0..10 {
        a.tick_founding();
        let region = &a.sim.as_ref().unwrap().world.regions[ri];
        if let Some(s) = region.settlements.iter().find(|s| s.id.contains("-set-f")) {
            founded = true;
            assert_eq!(s.size, "hamlet");
            assert!(
                s.population >= 12,
                "a hamlet is at least a dozen souls, got {}",
                s.population
            );
            assert!(!s.name.is_empty());
            assert!(
                !s.name.contains(&player_name),
                "the region names the place, not the founder"
            );
            assert_eq!(
                s.population as usize,
                s.people.len(),
                "every settler is a real person carried over"
            );
            break;
        }
    }
    assert!(
        founded,
        "twelve souls should have gathered within the waves"
    );
    // The ground is marked.
    assert_eq!(
        a.sim.as_ref().unwrap().world.regions[ri].terrain.get(x, y),
        Some(Terrain::Settlement),
        "the homestead tile becomes the hamlet"
    );
    // People are conserved: nobody was made from nothing.
    assert_eq!(
        total_population(&a),
        pop_before,
        "settlers must be subtracted from where they came from"
    );
    // Founder status: the place holds the player high.
    let (player_id, new_id) = {
        let sim = a.sim.as_ref().unwrap();
        let id = sim.world.regions[ri]
            .settlements
            .iter()
            .find(|s| s.id.contains("-set-f"))
            .map(|s| s.id.clone())
            .unwrap();
        (a.player_start.as_ref().unwrap().person.id.clone(), id)
    };
    let rep = a.sim.as_ref().unwrap().reputation.get(&player_id, &new_id);
    assert!(rep > 0.9, "founder standing should be near the top: {rep}");
    // The tile↔settlement mapping still holds: standing on the new tile
    // resolves to the new settlement, not a shifted neighbor.
    a.player_pos = Some(PlayerPos {
        region_idx: ri,
        px: x,
        py: y,
    });
    let (mri, msi) = a.player_on_settlement().expect("on settlement ground");
    let mapped = &a.sim.as_ref().unwrap().world.regions[mri].settlements[msi];
    assert!(
        mapped.id.contains("-set-f"),
        "x-rank insertion keeps tile mapping honest, got {}",
        mapped.id
    );
}

#[test]
fn poor_land_draws_no_one() {
    let Some((mut a, ri, _, _)) = wild_homestead() else {
        return;
    };
    a.sim.as_mut().unwrap().world.regions[ri].game_richness = 0.3;
    for _ in 0..5 {
        a.tick_founding();
    }
    assert!(
        !a.homestead_rumored && a.homestead_settlers.is_empty(),
        "thin land feeds no rumor"
    );
}

#[test]
fn an_unfarmed_cabin_is_just_a_cabin() {
    let Some((mut a, _, _, _)) = wild_homestead() else {
        return;
    };
    a.player_farms.clear();
    for _ in 0..5 {
        a.tick_founding();
    }
    assert!(
        !a.homestead_rumored && a.homestead_settlers.is_empty(),
        "no working field, no founding story"
    );
}
