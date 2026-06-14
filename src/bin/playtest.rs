use std::io::{self, BufRead, Write};

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{EncounterAction, ItemType, PeopleKind, SettlementService};
use deep_world_tui::save::{CompactSave, PlayerChoice};
use deep_world_tui::ui::app::App;
use deep_world_tui::voice::people_banks::PeopleBanks;
use deep_world_tui::voice::{self, Situation};

fn main() -> anyhow::Result<()> {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);
    let charts = load_charts().expect("Failed to load data/charts.ron");
    let mut app = App::new(seed, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);

    let mut recorded: Vec<PlayerChoice> = Vec::new();

    println!("=== Deep World Playtest (seed={}) ===", seed);
    println!("Commands: status, move <dir>, map, gather, forage, rest [h], tend,");
    println!("  enter <ri> <si>, exit, inventory, craft [n], use <svc>,");
    println!(
        "  buy/sell/steal <item>, build [kind], work, plant, harvest, stash/take <item> [n], quests, journal [n], region,"
    );
    println!("  encounter <action>, talk [idx], court [idx], collapse-dismiss,");
    println!("  save [slot], load [slot], record <file>, replay <file>, help, quit");
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
                if let Some(p) = app.player_pos {
                    recorded.push(PlayerChoice::TravelTo {
                        region_idx: p.region_idx,
                        px: p.px as u32,
                        py: p.py as u32,
                    });
                }
                if let Some(enc) = app.encounter {
                    match enc.species {
                        Some(sp) => println!(
                            "  *** Encounter! {:?} ({}) on {:?}",
                            enc.kind,
                            sp.name(),
                            enc.terrain
                        ),
                        None => println!("  *** Encounter! {:?} on {:?}", enc.kind, enc.terrain),
                    }
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
                recorded.push(PlayerChoice::Gather);
                app.gather();
                print_msg(&app);
            }
            "tend" | "physic" => {
                recorded.push(PlayerChoice::TendSelf);
                app.tend_illness();
                print_msg(&app);
            }
            "forage" => {
                recorded.push(PlayerChoice::ForageHerbs);
                app.forage_herbs();
                print_msg(&app);
            }
            "rest" | "r" => {
                let before = app.collapse_log.len();
                let hours: u32 = parts.get(1).and_then(|h| h.parse().ok()).unwrap_or(8);
                recorded.push(PlayerChoice::Rest);
                app.rest_hours(hours);
                if app.collapse_log.len() > before {
                    println!("  *** You collapsed during the rest!");
                }
                if let Some(d) = app.death_cause {
                    println!("  *** DEATH: {}", d.label());
                }
                print_msg(&app);
            }
            "enter" => {
                let ri: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let si: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                recorded.push(PlayerChoice::EnterSettlement {
                    region_idx: ri,
                    settlement_idx: si,
                });
                app.enter_settlement(ri, si);
                print_msg(&app);
            }
            "exit" => {
                recorded.push(PlayerChoice::ExitSettlement);
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
                    println!(
                        "  People: {}",
                        deep_world_tui::model::PeopleKind::from_name(&ps.person.people).label()
                    );
                    println!("  Profession: {}", ps.person.profession);
                }
                continue;
            }
            "craft" => {
                if parts.len() > 1 {
                    let idx: usize = parts[1].parse().unwrap_or(0);
                    recorded.push(PlayerChoice::CraftRecipe { recipe_idx: idx });
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
                recorded.push(PlayerChoice::UseService {
                    service: parts.get(1).copied().unwrap_or("").to_string(),
                });
                app.use_service(svc);
                print_msg(&app);
            }
            "court" => {
                let idx: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                app.apply_choice(&PlayerChoice::Court { person_idx: idx });
                recorded.push(PlayerChoice::Court { person_idx: idx });
                print_msg(&app);
            }
            "talk" | "t" => {
                let people_banks = match PeopleBanks::load("data/voice/people_banks.ron") {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("  Warning: Could not load people banks: {}", e);
                        continue;
                    }
                };
                if let (Some(ref sim), Some(pos)) = (&app.sim, app.player_pos) {
                    if let Some(region) = sim.world.regions.get(pos.region_idx) {
                        if let Some(settlement) = region.settlements.first() {
                            let idx: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                            if let Some(person) = settlement.people.get(idx) {
                                let npc_people = PeopleKind::from_name(&person.people);
                                let player_people = app.inter_people_bias.player_people;
                                for sit in [
                                    Situation::Greeting,
                                    Situation::Trade,
                                    Situation::NeedFine,
                                    Situation::Farewell,
                                    Situation::Gossip,
                                    Situation::NeedDire,
                                ] {
                                    let line = voice::voice_line_situation_biased(
                                        person,
                                        sit,
                                        player_people,
                                    );
                                    println!("  [{:?}] {}", sit, line);
                                }
                                let bank = people_banks.bank_for(&npc_people);
                                let bank_line = &bank.greetings[idx % bank.greetings.len()];
                                println!("  [{}] {}", person.people, bank_line);
                            } else {
                                println!(
                                    "  No NPC at index {}. Settlement has {} people.",
                                    idx,
                                    settlement.people.len()
                                );
                            }
                        }
                    }
                }
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
                recorded.push(PlayerChoice::ResolveEncounter {
                    action: parts.get(1).copied().unwrap_or("").to_string(),
                });
                app.resolve_encounter(action);
                print_msg(&app);
            }
            "buy" | "sell" | "steal" => {
                let Some(name) = parts.get(1) else {
                    println!("  Usage: {} <item>", parts[0]);
                    continue;
                };
                let Some(item) = App::item_from_name(name) else {
                    println!("  Unknown item: {}", name);
                    continue;
                };
                let choice = match parts[0] {
                    "buy" => PlayerChoice::BuyItem {
                        item: item.name().into(),
                    },
                    "sell" => PlayerChoice::SellItem {
                        item: item.name().into(),
                    },
                    _ => PlayerChoice::StealItem {
                        item: item.name().into(),
                    },
                };
                recorded.push(choice.clone());
                app.apply_choice(&choice);
                print_msg(&app);
            }
            "stash" | "take" => {
                let Some(name) = parts.get(1) else {
                    println!("  Usage: {} <item> [n]", parts[0]);
                    continue;
                };
                let Some(item) = App::item_from_name(name) else {
                    println!("  Unknown item: {}", name);
                    continue;
                };
                let n: u32 = parts.get(2).and_then(|x| x.parse().ok()).unwrap_or(1);
                let choice = if parts[0] == "stash" {
                    PlayerChoice::StashItem {
                        item: item.name().into(),
                        count: n,
                    }
                } else {
                    PlayerChoice::TakeItem {
                        item: item.name().into(),
                        count: n,
                    }
                };
                recorded.push(choice.clone());
                app.apply_choice(&choice);
                print_msg(&app);
            }
            "plant" => {
                let arg = parts.get(1).map(|s| s.to_string());
                match arg {
                    Some(name) => {
                        let crop = deep_world_tui::model::economy::CropType::from_name(&name);
                        if crop.is_none() {
                            println!("  Unknown crop: {name}");
                            continue;
                        }
                        recorded.push(PlayerChoice::PlantCrop { crop: name });
                        app.plant_crop(crop);
                    }
                    None => {
                        recorded.push(PlayerChoice::Plant);
                        app.plant();
                    }
                }
                print_msg(&app);
            }
            "harvest" => {
                recorded.push(PlayerChoice::Harvest);
                app.harvest();
                print_msg(&app);
            }
            "build" => {
                let kind_str = parts.get(1).map(|s| s.to_string());
                let choice = PlayerChoice::Build {
                    kind: kind_str.clone(),
                };
                recorded.push(choice.clone());
                app.apply_choice(&choice);
                print_msg(&app);
            }
            "work" => {
                app.work_site();
                print_msg(&app);
            }
            "quests" => {
                if let Some(ref sim) = app.sim {
                    if sim.quests.is_empty() {
                        println!("  No quests on the board.");
                    }
                    for (i, q) in sim.quests.iter().enumerate() {
                        println!(
                            "  [{}] {} ({}/{}, due day {}) — {}",
                            i,
                            q.description,
                            q.progress,
                            q.target,
                            q.deadline_day,
                            q.progress_hint()
                        );
                    }
                }
            }
            "journal" => {
                let n: usize = parts.get(1).and_then(|x| x.parse().ok()).unwrap_or(5);
                if let Some(ref sim) = app.sim {
                    for e in sim
                        .journal
                        .iter_rev()
                        .take(n)
                        .collect::<Vec<_>>()
                        .iter()
                        .rev()
                    {
                        println!("  [{}] {}", e.voice.label(), e.text);
                    }
                }
            }
            "region" => {
                if let (Some(pos), Some(ref sim)) = (app.player_pos, &app.sim) {
                    if let Some(r) = sim.world.regions.get(pos.region_idx) {
                        println!(
                            "  {} — weather {}, game {:.0}%",
                            r.name,
                            r.weather.name(),
                            r.game_richness * 100.0
                        );
                        for s in &r.settlements {
                            let fest = if s.in_festival(app.clock.day) {
                                " FESTIVAL"
                            } else {
                                ""
                            };
                            println!(
                                "    {}: pop {}, stores {:.0}, {} farms, {} buildings{}",
                                s.name,
                                s.population,
                                s.food_stock,
                                s.farms.len(),
                                s.buildings.iter().filter(|b| b.is_complete()).count(),
                                fest
                            );
                        }
                    }
                }
            }
            "record" => {
                let Some(file) = parts.get(1) else {
                    println!("  Usage: record <file>");
                    continue;
                };
                let compact = CompactSave {
                    seed,
                    player_choices: recorded.clone(),
                    tick: app.sim.as_ref().map_or(0, |s| s.world.tick),
                };
                match deep_world_tui::save::save_compact(&compact, file) {
                    Ok(()) => println!("  Recorded {} choices to {}", recorded.len(), file),
                    Err(e) => println!("  Record failed: {}", e),
                }
            }
            "replay" => {
                let Some(file) = parts.get(1) else {
                    println!("  Usage: replay <file>");
                    continue;
                };
                match deep_world_tui::save::load_compact(file) {
                    Ok(compact) => {
                        println!(
                            "  Replaying {} choices (recorded on seed {})...",
                            compact.player_choices.len(),
                            compact.seed
                        );
                        for c in &compact.player_choices {
                            app.apply_choice(c);
                        }
                        print_msg(&app);
                    }
                    Err(e) => println!("  Replay failed: {}", e),
                }
            }
            "collapse-dismiss" => {
                recorded.push(PlayerChoice::DismissCollapse);
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
            "god" => {
                println!("  God Affinity:");
                for g in [
                    deep_world_tui::model::GodName::Oltzed,
                    deep_world_tui::model::GodName::Keuru,
                    deep_world_tui::model::GodName::Sampsa,
                    deep_world_tui::model::GodName::Masa,
                    deep_world_tui::model::GodName::Kukri,
                ] {
                    let v = app.god_affinity.get(g);
                    if v.abs() > f64::EPSILON {
                        println!("    {:?}: {:.3}", g, v);
                    }
                }
                println!("  People Bias:");
                let pp = app.inter_people_bias.player_people;
                for p in [
                    deep_world_tui::model::PeopleKind::Metsik,
                    deep_world_tui::model::PeopleKind::Ahjo,
                    deep_world_tui::model::PeopleKind::Sepat,
                    deep_world_tui::model::PeopleKind::Arkit,
                    deep_world_tui::model::PeopleKind::Vayla,
                    deep_world_tui::model::PeopleKind::Laakso,
                    deep_world_tui::model::PeopleKind::Tzakhar,
                    deep_world_tui::model::PeopleKind::Merak,
                    deep_world_tui::model::PeopleKind::Shear,
                    deep_world_tui::model::PeopleKind::Hal,
                    deep_world_tui::model::PeopleKind::Khor,
                ] {
                    let raw = pp.bias_toward(p);
                    let eff = app.inter_people_bias.effective_bias(p);
                    if raw.abs() > f64::EPSILON || eff.abs() > f64::EPSILON {
                        println!("    {:?}: raw={:.3} eff={:.3}", p, raw, eff);
                    }
                }
                let title = app.god_affinity.people_title(pp);
                if !title.is_empty() {
                    println!("  Known as: {}", title);
                }
                continue;
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
            "[{}] E:{:.0}% H:{:.0}% | Day {} h{} ({} AF) | Pos {:?}",
            ps.person.name,
            app.vitals.energy * 100.0,
            app.vitals.hunger * 100.0,
            app.clock.day,
            app.clock.hour,
            app.clock.year_af(),
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
    println!("  talk [idx]    - Talk to NPC (voice + people bank)");
    println!("  collapse-dismiss - Dismiss collapse");
    println!("  god          - Show god affinity and people bias");
    println!("  save/load    - Save/load game");
    println!("  quit         - Exit");
}
