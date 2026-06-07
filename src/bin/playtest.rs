use std::io::{self, BufRead, Write};

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{EncounterAction, ItemType, SettlementService};
use deep_world_tui::ui::app::App;

fn main() -> anyhow::Result<()> {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    let charts = load_charts("data/charts.ron").expect("Failed to load data/charts.ron");
    let mut app = App::new(seed, charts);
    app.generate_player();
    app.accept_player();

    println!("=== Deep World Playtest (seed={}) ===", seed);
    println!("Commands: status, move <dir>, map, gather, rest,");
    println!("  enter <ri> <si>, exit, inventory, craft [n], use <svc>,");
    println!("  encounter <action>, collapse-dismiss, save, load, help, quit");
    println!();

    let stdin = io::stdin();
    loop {
        print_status(&app);
        print!("> ");
        io::stdout().flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "quit" | "q" => break,
            "help" | "h" => print_help(),
            "status" | "st" => {}
            "move" | "m" => {
                let (dx, dy) = match parts.get(1).copied().unwrap_or("") {
                    "n" => (0, -1),
                    "s" => (0, 1),
                    "e" => (1, 0),
                    "w" => (-1, 0),
                    "nn" => (0, -2),
                    "ne" => (1, -1),
                    "nw" => (-1, -1),
                    "se" => (1, 1),
                    "sw" => (-1, 1),
                    _ => {
                        println!("  Directions: n/s/e/w/ne/nw/se/sw/nn");
                        continue;
                    }
                };
                app.move_player(dx, dy);
                if let Some(enc) = app.encounter {
                    println!("  *** Encounter! {:?} on {:?}", enc.kind, enc.terrain);
                }
                if app.collapse.is_some() {
                    println!("  *** You collapsed!");
                }
            }
            "map" => {
                if let Some(ref sim) = app.sim {
                    if let Some(pos) = app.player_pos {
                        let region = &sim.world.regions[pos.region_idx];
                        println!("  Region: {} (day {})", region.name, app.clock.day);
                        if let Some((ri, si)) = app.player_on_settlement() {
                            println!("  At: {}", sim.world.regions[ri].settlements[si].name);
                        }
                        println!("  Settlements:");
                        for (si, s) in region.settlements.iter().enumerate() {
                            println!("    [{} {}] {}", pos.region_idx, si, s.name);
                        }
                    }
                }
                continue;
            }
            "gather" | "g" => {
                app.gather();
                print_msg(&app);
            }
            "rest" | "r" => {
                app.rest();
                print_msg(&app);
            }
            "enter" => {
                let ri: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let si: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                app.enter_settlement(ri, si);
                print_msg(&app);
            }
            "exit" => {
                app.exit_settlement();
                print_msg(&app);
            }
            "inventory" | "i" => {
                if let Some(ref ps) = app.player_start {
                    println!("  Inventory:");
                    for item in [
                        ItemType::Food,
                        ItemType::Coin,
                        ItemType::Iron,
                        ItemType::Herb,
                        ItemType::Wood,
                        ItemType::Stone,
                        ItemType::Cloth,
                    ] {
                        let count = ps.inventory.get(item);
                        if count > 0 {
                            println!("    {:?}: {}", item, count);
                        }
                    }
                    println!("  People: {}", ps.person.people);
                    println!("  Profession: {}", ps.person.profession);
                }
                continue;
            }
            "craft" => {
                if parts.len() > 1 {
                    let idx: usize = parts[1].parse().unwrap_or(0);
                    app.enter_craft();
                    app.craft_recipe(idx);
                    app.exit_craft();
                    print_msg(&app);
                } else {
                    println!("  Use 'craft <n>' to craft recipe #n");
                }
            }
            "use" => {
                let svc = match parts.get(1).copied().unwrap_or("") {
                    "tavern" => SettlementService::Tavern,
                    "temple" => SettlementService::Temple,
                    "forge" => SettlementService::Forge,
                    "hearth" => SettlementService::Hearth,
                    "trap" => SettlementService::TrapWorkshop,
                    _ => {
                        println!("  Services: tavern, temple, forge, hearth, trap");
                        continue;
                    }
                };
                app.use_service(svc);
                print_msg(&app);
            }
            "encounter" | "enc" => {
                let action = match parts.get(1).copied().unwrap_or("") {
                    "flee" => EncounterAction::Flee,
                    "bribe" => EncounterAction::Bribe,
                    "talk" => EncounterAction::Talk,
                    "calm" => EncounterAction::Calm,
                    "intimidate" => EncounterAction::Intimidate,
                    "push" => EncounterAction::PushThrough,
                    "trade" => EncounterAction::Trade,
                    "shelter" => EncounterAction::Shelter,
                    _ => {
                        println!(
                            "  Actions: flee, bribe, talk, calm, intimidate, push, trade, shelter"
                        );
                        continue;
                    }
                };
                app.resolve_encounter(action);
                print_msg(&app);
            }
            "collapse-dismiss" => {
                app.dismiss_collapse();
                print_msg(&app);
            }
            "save" => {
                app.save_game();
                println!("  Game saved.");
            }
            "load" => {
                app.load_game();
                println!("  Game loaded.");
            }
            "advance" => {
                app.advance_clock(1);
                println!("  1 hour passed.");
            }
            cmd => {
                println!("  Unknown: {}. Type 'help'.", cmd);
                continue;
            }
        }
    }
    Ok(())
}

fn print_status(app: &App) {
    if let Some(ref ps) = app.player_start {
        println!(
            "[{}] E:{:.0}% H:{:.0}% | Day {} h{} | Pos {:?}",
            ps.person.name,
            app.vitals.energy * 100.0,
            app.vitals.hunger * 100.0,
            app.clock.day,
            app.clock.hour,
            app.player_pos.map(|p| (p.px, p.py)).unwrap_or((0, 0)),
        );
    }
    if let Some(msg) = &app.status_msg {
        println!("  {}", msg);
    }
}

fn print_msg(app: &App) {
    if let Some(msg) = &app.status_msg {
        println!("  {}", msg);
    }
}

fn print_help() {
    println!("  status       - Show player status");
    println!("  move <dir>   - Move (n/s/e/w/ne/nw/se/sw/nn)");
    println!("  map          - Show region info and settlements");
    println!("  gather       - Gather resources");
    println!("  rest         - Rest and recover");
    println!("  advance      - Advance clock 1 hour");
    println!("  enter <ri> <si> - Enter settlement");
    println!("  exit         - Exit settlement");
    println!("  inventory    - Show inventory");
    println!("  craft <n>    - Craft recipe #n");
    println!("  use <svc>    - Use service (tavern/temple/forge/hearth/trap)");
    println!("  encounter <a> - Resolve encounter");
    println!("  collapse-dismiss - Dismiss collapse");
    println!("  save/load    - Save/load game");
    println!("  quit         - Exit");
}
