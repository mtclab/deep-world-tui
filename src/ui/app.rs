use crate::charts::Charts;
use crate::gen::player::generate_player_start;
use crate::model::{
    craft_recipes, Collapse, Encounter, EncounterAction, GameClock, GodAffinity, GodName,
    InterPeopleBias, Inventory, ItemType, Need, PeopleKind, PlayerPos, PlayerStart, PlayerVitals,
    Settlement, SettlementService, Terrain,
};
use crate::rng::SeedRng;
use crate::save::{self, SaveData};
use crate::sim::SimState;

use super::event::AppEvent;

pub enum Screen {
    CharacterCreation,
    World,
    Map {
        region_idx: usize,
        px: usize,
        py: usize,
    },
    Overmap {
        region_idx: usize,
    },
    Inventory,
    Craft {
        scroll: u16,
    },
    WorldAlerts {
        scroll: u16,
    },
    Location {
        region_idx: usize,
        settlement_idx: usize,
        scroll: u16,
    },
    Npc {
        region_idx: usize,
        settlement_idx: usize,
        person_idx: usize,
        scroll: u16,
    },
    Journal {
        scroll: u16,
    },
    Talk {
        region_idx: usize,
        settlement_idx: usize,
        person_idx: usize,
        scroll: u16,
    },
    Market {
        region_idx: usize,
        settlement_idx: usize,
        scroll: u16,
    },
    Encounter,
    Collapse,
    GameOver,
}

pub struct App {
    pub sim: Option<SimState>,
    pub player_start: Option<PlayerStart>,
    pub running: bool,
    pub tick_interval: u64,
    pub screen: Screen,
    pub status_msg: Option<String>,
    pub player_pos: Option<PlayerPos>,
    pub clock: GameClock,
    pub vitals: PlayerVitals,
    pub encounter: Option<Encounter>,
    pub collapse: Option<Collapse>,
    pub god_affinity: GodAffinity,
    pub inter_people_bias: InterPeopleBias,
    seed: u64,
    charts: Charts,
    player_rng: Option<SeedRng>,
}

impl App {
    pub fn new(seed: u64, charts: Charts) -> Self {
        let player_rng = SeedRng::new(seed);
        App {
            sim: None,
            player_start: None,
            running: true,
            tick_interval: 100,
            screen: Screen::CharacterCreation,
            status_msg: None,
            player_pos: None,
            clock: GameClock::default(),
            vitals: PlayerVitals::default(),
            encounter: None,
            collapse: None,
            god_affinity: GodAffinity::new(),
            inter_people_bias: InterPeopleBias::default(),
            seed,
            charts,
            player_rng: Some(player_rng),
        }
    }

    pub fn generate_player(&mut self) {
        if let Some(ref mut rng) = self.player_rng {
            let ps = generate_player_start(rng, &self.charts);
            let pk = PeopleKind::from_name(&ps.person.people);
            self.inter_people_bias = InterPeopleBias::new(pk);
            self.player_start = Some(ps);
        }
    }

    pub fn reroll_player(&mut self) {
        if let Some(ref mut rng) = self.player_rng {
            if let Some(ref mut ps) = self.player_start {
                ps.reroll(rng, &self.charts);
            }
        }
    }

    pub fn accept_player(&mut self) {
        if let Some(mut ps) = self.player_start.take() {
            ps.accepted = true;
            self.player_start = Some(ps);
            let sim = SimState::new(self.seed, self.charts.clone());
            self.sim = Some(sim);
            self.screen = Screen::World;
        }
    }

    pub fn settlement_list(&self) -> Vec<(usize, usize, String)> {
        let mut out = Vec::new();
        if let Some(ref sim) = self.sim {
            for (ri, region) in sim.world.regions.iter().enumerate() {
                for (si, sett) in region.settlements.iter().enumerate() {
                    out.push((ri, si, format!("{} — {}", sett.name, region.name)));
                }
            }
        }
        out
    }

    pub fn enter_settlement(&mut self, region_idx: usize, settlement_idx: usize) {
        if let Some(npc_people) = self.current_settlement_people() {
            let bias = self.inter_people_bias.player_people.bias_toward(npc_people)
                + self.clock.season().bias_modifier();
            if bias < -0.20 {
                self.status_msg = Some(format!(
                    "Guards block your path. 'No {} allowed beyond this point.' You turn back.",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
            if bias < -0.10 {
                self.status_msg = Some(
                    "A guard eyes you suspiciously but lets you pass. 'Keep your head down.'"
                        .into(),
                );
            }
        }
        self.screen = Screen::Location {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn exit_settlement(&mut self) {
        self.screen = Screen::World;
    }

    pub fn current_settlement(&self) -> Option<&Settlement> {
        match &self.screen {
            Screen::Location {
                region_idx,
                settlement_idx,
                ..
            }
            | Screen::Npc {
                region_idx,
                settlement_idx,
                ..
            }
            | Screen::Talk {
                region_idx,
                settlement_idx,
                ..
            } => self.sim.as_ref().and_then(|sim| {
                sim.world
                    .regions
                    .get(*region_idx)
                    .and_then(|r| r.settlements.get(*settlement_idx))
            }),
            _ => None,
        }
    }

    pub fn enter_npc(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        self.screen = Screen::Npc {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn exit_npc(&mut self, region_idx: usize, settlement_idx: usize) {
        self.screen = Screen::Location {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn enter_journal(&mut self) {
        self.screen = Screen::Journal { scroll: 0 };
    }

    pub fn exit_journal(&mut self) {
        self.screen = Screen::World;
    }

    pub fn save_game(&mut self) {
        if let Some(ref sim) = self.sim {
            let data = SaveData {
                sim: sim.clone(),
                player_start: self.player_start.clone(),
                clock: self.clock,
                vitals: self.vitals,
                player_pos: self.player_pos,
                god_affinity: self.god_affinity,
                inter_people_bias: self.inter_people_bias,
            };
            match save::save_game(&data, "save.ron") {
                Ok(()) => self.status_msg = Some("Saved to save.ron".into()),
                Err(e) => self.status_msg = Some(format!("Save failed: {}", e)),
            }
        }
    }

    pub fn load_game(&mut self) {
        match save::load_game("save.ron") {
            Ok(data) => {
                self.sim = Some(data.sim);
                self.player_start = data.player_start;
                self.clock = data.clock;
                self.vitals = data.vitals;
                self.player_pos = data.player_pos;
                self.god_affinity = data.god_affinity;
                self.inter_people_bias = data.inter_people_bias;
                self.screen = Screen::World;
                self.status_msg = Some("Loaded from save.ron".into());
            }
            Err(e) => self.status_msg = Some(format!("Load failed: {}", e)),
        }
    }

    pub fn enter_talk(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                if region.region_type == "forest" && self.god_affinity.get(GodName::Metsik) > 0.2 {
                    self.god_affinity.adjust(GodName::Metsik, 0.01);
                }
                if region.region_type == "river_valley"
                    && self.god_affinity.get(GodName::Vayla) > 0.2
                {
                    self.god_affinity.adjust(GodName::Vayla, 0.01);
                }
            }
        }
        self.screen = Screen::Talk {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn exit_talk(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        self.screen = Screen::Npc {
            region_idx,
            settlement_idx,
            person_idx,
            scroll: 0,
        };
    }

    pub fn give_food(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref mut ps) = self.player_start {
            if !ps.inventory.remove(ItemType::Food, 1) {
                self.status_msg = Some("No food to give".into());
                return;
            }
        }
        let npc_people = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| PeopleKind::from_name(&p.people))
        });
        if let Some(npc_pk) = npc_people {
            let bias = self.inter_people_bias.player_people.bias_toward(npc_pk);
            if bias < -0.20 {
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Food, 1);
                }
                self.status_msg = Some(format!(
                    "'Keep your food, {}.' They push it back. 'We don't take from clearing-sympathizers.'",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
        }
        let player_id = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        let settlement_id = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .map(|s| s.id.clone())
        });
        if let Some(ref mut sim) = self.sim {
            if let Some(person) = sim
                .world
                .regions
                .get_mut(region_idx)
                .and_then(|r| r.settlements.get_mut(settlement_idx))
                .and_then(|s| s.people.get_mut(person_idx))
            {
                let person_id = person.id.clone();
                person.needs.satisfy(Need::Food, 0.2);
                if let (Some(pid), Some(sid)) = (&player_id, &settlement_id) {
                    let mut trust_bonus = 0.05;
                    let mut rep_bonus = 0.02;
                    if self.god_affinity.get(GodName::Ahjo) > 0.3 {
                        trust_bonus += 0.02;
                        rep_bonus += 0.01;
                    }
                    if self.god_affinity.get(GodName::Vayla) > 0.3 {
                        trust_bonus += 0.01;
                    }
                    let npc_people = PeopleKind::from_name(&person.people);
                    sim.relationships.update_relationship_biased_full(
                        pid,
                        &person_id,
                        "gave food",
                        sim.world.tick,
                        trust_bonus,
                        0.03,
                        self.inter_people_bias.player_people,
                        npc_people,
                        &person.personality,
                    );
                    sim.reputation.adjust_local_biased(
                        pid,
                        sid,
                        rep_bonus,
                        self.inter_people_bias.player_people,
                        npc_people,
                    );
                }
                self.status_msg = Some(format!("Gave food to {}", person.name));
                self.god_affinity.adjust(GodName::Ahjo, 0.02);
                self.god_affinity.adjust(GodName::Vayla, 0.01);
            }
        }
    }

    pub fn give_coin(&mut self, region_idx: usize, settlement_idx: usize, person_idx: usize) {
        if let Some(ref mut ps) = self.player_start {
            if !ps.inventory.remove(ItemType::Coin, 1) {
                self.status_msg = Some("No coin to give".into());
                return;
            }
        }
        let npc_people = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .and_then(|s| s.people.get(person_idx))
                .map(|p| PeopleKind::from_name(&p.people))
        });
        if let Some(npc_pk) = npc_people {
            let bias = self.inter_people_bias.player_people.bias_toward(npc_pk);
            if bias < -0.20 {
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Coin, 1);
                }
                self.status_msg = Some(format!(
                    "The coin is set back on the table. 'We don't take {} coin here.'",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
        }
        let player_id = self.player_start.as_ref().map(|ps| ps.person.id.clone());
        let settlement_id = self.sim.as_ref().and_then(|sim| {
            sim.world
                .regions
                .get(region_idx)
                .and_then(|r| r.settlements.get(settlement_idx))
                .map(|s| s.id.clone())
        });
        if let Some(ref mut sim) = self.sim {
            if let Some(person) = sim
                .world
                .regions
                .get_mut(region_idx)
                .and_then(|r| r.settlements.get_mut(settlement_idx))
                .and_then(|s| s.people.get_mut(person_idx))
            {
                let person_id = person.id.clone();
                person.needs.satisfy(Need::Money, 0.2);
                if let (Some(pid), Some(sid)) = (&player_id, &settlement_id) {
                    let mut trust_bonus = 0.03;
                    let mut rep_bonus = 0.01;
                    if self.god_affinity.get(GodName::Ahjo) > 0.3 {
                        trust_bonus += 0.01;
                        rep_bonus += 0.01;
                    }
                    if self.god_affinity.get(GodName::Vayla) > 0.3 {
                        trust_bonus += 0.01;
                    }
                    let npc_people = PeopleKind::from_name(&person.people);
                    sim.relationships.update_relationship_biased_full(
                        pid,
                        &person_id,
                        "gave coin",
                        sim.world.tick,
                        trust_bonus,
                        rep_bonus,
                        self.inter_people_bias.player_people,
                        npc_people,
                        &person.personality,
                    );
                    sim.reputation.adjust_local_biased(
                        pid,
                        sid,
                        0.01,
                        self.inter_people_bias.player_people,
                        npc_people,
                    );
                }
                self.status_msg = Some(format!("Gave coin to {}", person.name));
                self.god_affinity.adjust(GodName::Ahjo, 0.02);
                self.god_affinity.adjust(GodName::Vayla, 0.01);
            }
        }
    }

    pub fn enter_alerts(&mut self) {
        self.screen = Screen::WorldAlerts { scroll: 0 };
    }

    pub fn exit_alerts(&mut self) {
        self.screen = Screen::World;
    }

    pub fn enter_map(&mut self, region_idx: usize) {
        let (px, py) = if let Some(ref pos) = self.player_pos {
            if pos.region_idx == region_idx {
                (pos.px, pos.py)
            } else {
                self.find_settlement_pos(region_idx)
            }
        } else {
            self.find_settlement_pos(region_idx)
        };
        self.player_pos = Some(PlayerPos { region_idx, px, py });
        self.screen = Screen::Map { region_idx, px, py };
    }

    fn find_settlement_pos(&self, region_idx: usize) -> (usize, usize) {
        if let Some(ref sim) = self.sim {
            if let Some(region) = sim.world.regions.get(region_idx) {
                if let Some(pos) = region
                    .terrain
                    .tiles
                    .iter()
                    .position(|&t| t == Terrain::Settlement)
                {
                    return (pos % region.terrain.width, pos / region.terrain.width);
                }
            }
        }
        (20, 10)
    }

    pub fn exit_map(&mut self) {
        self.screen = Screen::World;
    }

    pub fn enter_overmap(&mut self) {
        let region_idx = match &self.screen {
            Screen::Map { region_idx, .. } => *region_idx,
            _ => 0,
        };
        self.screen = Screen::Overmap { region_idx };
    }

    pub fn exit_overmap(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn gather(&mut self) {
        if self.clock.time_of_day().is_dark() {
            self.status_msg = Some("Too dark to gather".into());
            return;
        }
        let (terrain_item, terrain) = self
            .player_pos
            .and_then(|pos| {
                self.sim.as_ref().map(|sim| {
                    let region = sim.world.regions.get(pos.region_idx);
                    let t = region.and_then(|r| r.terrain.get(pos.px, pos.py));
                    let item = t.and_then(ItemType::gather_from);
                    (item, t)
                })
            })
            .unwrap_or((None, None));
        if let (Some(item), Some(terrain)) = (terrain_item, terrain) {
            match terrain {
                Terrain::Forest => {
                    self.god_affinity.adjust(GodName::Metsik, 0.03);
                    self.god_affinity.adjust(GodName::Ahjo, -0.01);
                }
                Terrain::Grass | Terrain::Farmland => {
                    self.god_affinity.adjust(GodName::Ahjo, 0.03);
                    self.god_affinity.adjust(GodName::Metsik, -0.01);
                }
                _ => {}
            }
            let season = self.clock.season();
            let mult = season.gather_multiplier();
            let pp = self.inter_people_bias.player_people;
            let people_bonus = Terrain::people_gather_bonus(pp, terrain);
            let count = if mult > 0.5 {
                1 + people_bonus
            } else {
                people_bonus
            };
            if count == 0 {
                self.status_msg = Some(format!(
                    "Too scarce in {} to gather {}",
                    season,
                    item.name()
                ));
                return;
            }
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(item, count);
            }
            self.advance_clock_hour();
            self.status_msg = Some(format!(
                "Gathered {} {} (1h, {})",
                count,
                item.name(),
                season
            ));
        } else {
            self.status_msg = Some("Nothing to gather here".into());
        }
    }

    pub fn enter_inventory(&mut self) {
        self.screen = Screen::Inventory;
    }

    pub fn exit_inventory(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn enter_craft(&mut self) {
        self.screen = Screen::Craft { scroll: 0 };
    }

    pub fn exit_craft(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn craft_recipe(&mut self, recipe_idx: usize) {
        let player_people = self.inter_people_bias.player_people;
        let recipes: Vec<_> = craft_recipes()
            .into_iter()
            .filter(|r| r.people.is_none() || r.people == Some(player_people))
            .collect();
        if let Some(recipe) = recipes.get(recipe_idx) {
            if let Some(ref mut ps) = self.player_start {
                let inv = &mut ps.inventory;
                let can_craft = recipe
                    .inputs
                    .iter()
                    .all(|(item, count)| inv.get(*item) >= *count);
                if can_craft {
                    for (item, count) in &recipe.inputs {
                        inv.remove(*item, *count);
                    }
                    inv.add(recipe.output, recipe.output_count);
                    self.advance_clock(2);
                    self.status_msg = Some(format!(
                        "Crafted {} (x{}) (2h)",
                        recipe.name, recipe.output_count
                    ));
                } else {
                    self.status_msg = Some("Not enough materials".into());
                }
            }
        }
    }

    pub fn enter_market(&mut self, region_idx: usize, settlement_idx: usize) {
        self.screen = Screen::Market {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn exit_market(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn current_settlement_people(&self) -> Option<PeopleKind> {
        let pos = self.player_pos?;
        let sim = self.sim.as_ref()?;
        let region = sim.world.regions.get(pos.region_idx)?;
        let settlement = region.settlements.first()?;
        let dominant = settlement.people.first()?;
        Some(PeopleKind::from_name(&dominant.people))
    }

    pub fn buy_item(&mut self, item: ItemType) {
        if !item.tradeable() {
            self.status_msg = Some("Cannot buy that".into());
            return;
        }
        let base_price = item.base_price();
        let seller_people = self.current_settlement_people();
        let modifier = seller_people
            .map(|sp| self.inter_people_bias.price_modifier(sp))
            .unwrap_or(1.0);
        let price = ((base_price as f64 * modifier).ceil() as u32).max(1);
        if let Some(ref mut ps) = self.player_start {
            if ps.inventory.remove(ItemType::Coin, price) {
                ps.inventory.add(item, 1);
                self.advance_clock_hour();
                self.status_msg =
                    Some(format!("Bought 1 {} for {} coins (1h)", item.name(), price));
                self.god_affinity.adjust(GodName::Ahjo, 0.02);
            } else {
                self.status_msg = Some(format!("Need {} coins", price));
            }
        }
    }

    pub fn sell_item(&mut self, item: ItemType) {
        if !item.tradeable() {
            self.status_msg = Some("Cannot sell that".into());
            return;
        }
        let base_price = item.base_price();
        let buyer_people = self.current_settlement_people();
        let modifier = buyer_people
            .map(|bp| 2.0 - self.inter_people_bias.price_modifier(bp))
            .unwrap_or(1.0);
        let price = ((base_price as f64 * modifier).floor() as u32).max(1);
        if let Some(ref mut ps) = self.player_start {
            if ps.inventory.remove(item, 1) {
                ps.inventory.add(ItemType::Coin, price);
                self.advance_clock_hour();
                self.status_msg = Some(format!("Sold 1 {} for {} coins (1h)", item.name(), price));
                self.god_affinity.adjust(GodName::Ahjo, 0.01);
            } else {
                self.status_msg = Some(format!("No {} to sell", item.name()));
            }
        }
    }

    pub fn player_inventory(&self) -> Inventory {
        self.player_start
            .as_ref()
            .map(|ps| ps.inventory.clone())
            .unwrap_or_default()
    }

    pub fn advance_clock(&mut self, hours: u32) {
        let season = self.clock.season();
        self.clock.advance(hours);
        if let Some(ref mut ps) = self.player_start {
            self.vitals.tick(hours, &mut ps.inventory, season);
        }
        if let Some(ref mut sim) = self.sim {
            for _ in 0..hours {
                sim.step();
            }
        }
        self.check_collapse();
    }

    pub fn check_encounter(&mut self, terrain: Terrain) {
        let pp = Some(self.inter_people_bias.player_people);
        if let Some(enc) = Encounter::roll_biased(terrain, self.clock.hour, self.seed, pp) {
            self.encounter = Some(enc);
            self.screen = Screen::Encounter;
        }
    }

    pub fn dismiss_encounter(&mut self) {
        self.resolve_encounter(EncounterAction::Flee);
    }

    pub fn resolve_encounter(&mut self, action: EncounterAction) {
        let coins = action.coin_cost();
        if coins > 0 {
            if let Some(ref mut ps) = self.player_start {
                if !ps.inventory.remove(ItemType::Coin, coins) {
                    self.status_msg = Some("Not enough coins".into());
                    return;
                }
            }
        }
        self.vitals.energy = (self.vitals.energy - action.energy_cost()).max(0.0);
        self.vitals.hunger = (self.vitals.hunger - action.hunger_cost()).max(0.0);
        let hours = action.hours();
        if hours > 0 {
            self.advance_clock(hours);
        }
        if let Some((god, delta)) = action.god_affinity_effect() {
            self.god_affinity.adjust(god, delta);
        }
        let enc_kind = self.encounter.map(|e| e.kind);
        let enc_mod = self
            .encounter
            .and_then(|_| {
                self.sim.as_ref().and_then(|sim| {
                    let pos = self.player_pos?;
                    let region = sim.world.regions.get(pos.region_idx)?;
                    let settlement = region.settlements.first()?;
                    let person = settlement.people.first()?;
                    Some(InterPeopleBias::encounter_modifier(&person.personality))
                })
            })
            .unwrap_or_default();
        let people_bias_mod = self.current_settlement_people().map_or(0.0, |npc_people| {
            self.inter_people_bias.player_people.bias_toward(npc_people)
                + self.clock.season().bias_modifier()
        });
        let talk_success = people_bias_mod > -0.20;
        let trade_bonus = people_bias_mod > 0.05;
        let msg = match action {
            EncounterAction::Flee => {
                if enc_mod.flee > 0.05 {
                    "You fled quickly! Your instincts served you.".into()
                } else {
                    "You fled! (cost some energy)".into()
                }
            }
            EncounterAction::Bribe => {
                let effective_cost = ((coins as f64) * (1.0 + enc_mod.bribe_cost)).max(1.0) as u32;
                if effective_cost > coins {
                    if let Some(ref mut ps) = self.player_start {
                        let extra = effective_cost - coins;
                        if ps.inventory.get(ItemType::Coin) >= extra {
                            ps.inventory.remove(ItemType::Coin, extra);
                            format!(
                                "You paid {} coins total — they drove a hard bargain.",
                                effective_cost
                            )
                        } else {
                            "You paid the bandit off (2 coins).".into()
                        }
                    } else {
                        "You paid the bandit off.".into()
                    }
                } else {
                    "You paid them off (2 coins).".into()
                }
            }
            EncounterAction::Calm => {
                if enc_mod.calm > 0.03 {
                    "Your calm presence soothed the beast. It bows its head.".into()
                } else {
                    "The beast settled. It watches you leave.".into()
                }
            }
            EncounterAction::Intimidate => {
                if enc_mod.intimidate > 0.03 {
                    "You loomed large. They scattered before you.".into()
                } else {
                    "You stared them down. They backed off.".into()
                }
            }
            EncounterAction::Talk => {
                if !talk_success {
                    "They turned away coldly. No wisdom shared.".into()
                } else if enc_mod.talk > 0.03 {
                    "The traveler warmed to you quickly. Wisdom flows freely.".into()
                } else if enc_mod.talk < -0.02 {
                    "Words came slow. They barely shared a thing.".into()
                } else {
                    "The traveler shared road wisdom (1h).".into()
                }
            }
            EncounterAction::Trade => {
                if let Some(ref mut ps) = self.player_start {
                    let base_herbs = if trade_bonus { 2 } else { 1 };
                    let herbs = if enc_mod.trade > 0.02 {
                        base_herbs + 1
                    } else {
                        base_herbs
                    };
                    ps.inventory.add(ItemType::Herb, herbs);
                    if herbs >= 3 {
                        "A generous trade — three herbs for your news! (1h)".into()
                    } else if herbs == 2 {
                        "A good trade — two herbs (1h)".into()
                    } else {
                        "Traded news for herbs (1h)".into()
                    }
                } else {
                    "Traded news for herbs (1h)".into()
                }
            }
            EncounterAction::Shelter => "You waited out the storm (1h).".into(),
            EncounterAction::PushThrough => {
                if enc_mod.push_through > 0.03 {
                    "You surged ahead — nothing could slow you!".into()
                } else {
                    "You pushed through regardless!".into()
                }
            }
        };
        if let Some(ref mut sim) = self.sim {
            if let Some(kind) = enc_kind {
                let journal_text = format!("Encounter ({:?}): {} — {}", kind, action.label(), msg);
                sim.log_journal(sim.world.tick, journal_text);
            }
        }
        self.encounter = None;
        self.status_msg = Some(msg);
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn check_collapse(&mut self) {
        if self.vitals.hunger > 0.0 && self.vitals.energy > 0.0 {
            return;
        }
        let local_rep = self
            .player_start
            .as_ref()
            .and_then(|ps| {
                let pid = &ps.person.id;
                self.sim.as_ref().and_then(|sim| {
                    let pos = self.player_pos?;
                    let region = sim.world.regions.get(pos.region_idx)?;
                    let settlement = region.settlements.first()?;
                    Some(sim.reputation.get(pid, &settlement.id))
                })
            })
            .unwrap_or(0.0);
        let local_people = self.current_settlement_people();
        let collapse = Collapse::roll_biased(
            self.seed,
            &self.god_affinity,
            local_rep,
            self.inter_people_bias.player_people,
            local_people.unwrap_or(self.inter_people_bias.player_people),
        );
        let outcome = collapse.outcome;
        let hours = outcome.hours_passed();
        let died = collapse.died;
        self.vitals.hunger = (self.vitals.hunger + outcome.hunger_restore()).min(1.0);
        self.vitals.energy = (self.vitals.energy + outcome.energy_restore()).min(1.0);
        if let Some(ref mut ps) = self.player_start {
            let loss = outcome.coin_loss();
            ps.inventory.remove(ItemType::Coin, loss);
            if let Some(item) = outcome.item_loss() {
                ps.inventory.remove(item, 1);
            }
        }
        if outcome.is_divine() {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(ItemType::Herb, 3);
                ps.inventory.add(ItemType::Food, 2);
            }
        }
        if outcome.is_beast_aided() {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(ItemType::Herb, 1);
            }
        }
        if outcome.is_hostile() {
            self.vitals.hunger = 0.15;
            self.vitals.energy = 0.1;
        }
        self.advance_clock(hours);
        self.collapse = Some(collapse);
        if let Some(ref mut sim) = self.sim {
            let journal_text = if died {
                format!("COLLAPSED — {:?}. You did not wake.", outcome)
            } else if let Some(god) = collapse.rescued_by {
                format!("COLLAPSED — {:?}. {} intervened.", outcome, god.label())
            } else {
                format!("COLLAPSED — {:?}", outcome)
            };
            sim.log_journal(sim.world.tick, journal_text);
        }
        if died {
            self.screen = Screen::GameOver;
        } else {
            self.screen = Screen::Collapse;
        }
    }

    pub fn dismiss_collapse(&mut self) {
        self.collapse = None;
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        let px = self.player_pos.map(|p| p.px).unwrap_or(20);
        let py = self.player_pos.map(|p| p.py).unwrap_or(10);
        self.screen = Screen::Map { region_idx, px, py };
    }

    pub fn restart_game(&mut self) {
        self.sim = None;
        self.player_start = None;
        self.collapse = None;
        self.encounter = None;
        self.clock = GameClock::default();
        self.vitals = PlayerVitals::default();
        self.player_pos = None;
        self.screen = Screen::CharacterCreation;
        self.status_msg = None;
        self.running = true;
    }

    pub fn use_service(&mut self, service: SettlementService) {
        if let Some(npc_people) = self.current_settlement_people() {
            let mut bias = self.inter_people_bias.player_people.bias_toward(npc_people);
            bias += self.clock.season().bias_modifier();
            if bias < -0.15 {
                self.status_msg = Some(format!(
                    "They refuse to serve {} here. 'We don't serve your kind.'",
                    self.inter_people_bias.player_people.label()
                ));
                return;
            }
            if bias < -0.05 {
                let cost_extra = service.cost();
                let total = cost_extra + 1;
                if let Some(ref ps) = self.player_start {
                    if ps.inventory.get(ItemType::Coin) < total {
                        self.status_msg = Some(format!(
                            "They serve you grudgingly. Price is {} coins (surcharge for outsiders). You can't afford it.",
                            total
                        ));
                        return;
                    }
                }
            }
        }
        let mut cost = service.cost();
        if let Some(npc_people) = self.current_settlement_people() {
            let bias = self.inter_people_bias.player_people.bias_toward(npc_people);
            if bias < -0.05 {
                cost += 1;
            }
        }
        let npc_personality = self.sim.as_ref().and_then(|sim| {
            let pos = self.player_pos?;
            let region = sim.world.regions.get(pos.region_idx)?;
            let settlement = region.settlements.first()?;
            settlement.people.first().map(|p| p.personality.clone())
        });
        if let Some(ref personality) = npc_personality {
            let price_mod = InterPeopleBias::trade_price_modifier(personality);
            let extra = (service.cost() as f64 * price_mod).ceil() as u32;
            cost = cost.saturating_add(extra);
        }
        if let Some(ref mut ps) = self.player_start {
            if !ps.inventory.remove(ItemType::Coin, cost) {
                self.status_msg = Some(format!("Need {} coins for {}", cost, service.label()));
                return;
            }
        }
        match service {
            SettlementService::Tavern => {
                self.vitals.energy = (self.vitals.energy + 0.4).min(1.0);
                self.vitals.hunger = (self.vitals.hunger + 0.2).min(1.0);
                self.advance_clock(2);
                self.status_msg = Some("Rested at tavern (+energy, +hunger, 2h, 2 coins)".into());
            }
            SettlementService::Temple => {
                self.vitals.hunger = (self.vitals.hunger + 0.5).min(1.0);
                self.vitals.energy = (self.vitals.energy + 0.3).min(1.0);
                self.advance_clock(3);
                self.status_msg = Some("Blessed at temple (+hunger, +energy, 3h, 3 coins)".into());
            }
        }
    }

    pub fn advance_clock_hour(&mut self) {
        self.advance_clock(1);
    }

    pub fn rest(&mut self) {
        self.advance_clock(8);
        self.vitals.rest();
        self.god_affinity.adjust(GodName::Vayla, 0.02);
        self.status_msg = Some("Rested (8h)".into());
    }

    pub fn clock_str(&self) -> String {
        let tod = self.clock.time_of_day();
        let season = self.clock.season();
        format!(
            "D{} {:02}:00 {} {}",
            self.clock.day,
            self.clock.hour,
            tod.glyph(),
            season.glyph()
        )
    }
}

enum MoveResult {
    EdgeTransition {
        region_idx: usize,
        px: usize,
        py: usize,
    },
    Step {
        region_idx: usize,
        px: usize,
        py: usize,
    },
    Blocked {
        msg: String,
    },
}

impl App {
    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let result = self.compute_move(dx, dy);
        match result {
            Some(MoveResult::EdgeTransition { region_idx, px, py }) => {
                if let Some(ref mut p) = self.player_pos {
                    p.region_idx = region_idx;
                    p.px = px;
                    p.py = py;
                }
                let terrain = self
                    .sim
                    .as_ref()
                    .and_then(|sim| sim.world.regions.get(region_idx))
                    .and_then(|r| r.terrain.get(px, py))
                    .unwrap_or(Terrain::Grass);
                self.advance_clock(terrain.travel_hours());
                self.check_encounter(terrain);
                if self.encounter.is_none() {
                    self.screen = Screen::Map { region_idx, px, py };
                }
            }
            Some(MoveResult::Step { region_idx, px, py }) => {
                if let Some(ref mut p) = self.player_pos {
                    p.px = px;
                    p.py = py;
                }
                let terrain = self
                    .sim
                    .as_ref()
                    .and_then(|sim| sim.world.regions.get(region_idx))
                    .and_then(|r| r.terrain.get(px, py))
                    .unwrap_or(Terrain::Grass);
                self.advance_clock(terrain.travel_hours());
                self.check_encounter(terrain);
                if self.encounter.is_none() {
                    self.screen = Screen::Map { region_idx, px, py };
                }
            }
            Some(MoveResult::Blocked { msg }) => {
                self.status_msg = Some(msg);
            }
            None => {}
        }
    }

    fn compute_move(&self, dx: i32, dy: i32) -> Option<MoveResult> {
        let pos = self.player_pos?;
        let sim = self.sim.as_ref()?;
        let region = sim.world.regions.get(pos.region_idx)?;
        let map_w = region.terrain.width;
        let map_h = region.terrain.height;
        let nx = pos.px as i32 + dx;
        let ny = pos.py as i32 + dy;

        if nx < 0 {
            region
                .neighbors
                .west
                .map(|west| MoveResult::EdgeTransition {
                    region_idx: west,
                    px: map_w - 1,
                    py: pos.py,
                })
        } else if nx >= map_w as i32 {
            region
                .neighbors
                .east
                .map(|east| MoveResult::EdgeTransition {
                    region_idx: east,
                    px: 0,
                    py: pos.py,
                })
        } else if ny < 0 {
            region
                .neighbors
                .north
                .map(|north| MoveResult::EdgeTransition {
                    region_idx: north,
                    px: pos.px,
                    py: map_h - 1,
                })
        } else if ny >= map_h as i32 {
            region
                .neighbors
                .south
                .map(|south| MoveResult::EdgeTransition {
                    region_idx: south,
                    px: pos.px,
                    py: 0,
                })
        } else {
            let ux = nx as usize;
            let uy = ny as usize;
            let terrain = region.terrain.get(ux, uy);
            if let Some(t) = terrain {
                if t.passable() {
                    Some(MoveResult::Step {
                        region_idx: pos.region_idx,
                        px: ux,
                        py: uy,
                    })
                } else {
                    Some(MoveResult::Blocked {
                        msg: format!("Blocked: {:?}", t),
                    })
                }
            } else {
                None
            }
        }
    }

    pub fn player_on_settlement(&self) -> Option<(usize, usize)> {
        if let Some(ref pos) = self.player_pos {
            if let Some(ref sim) = self.sim {
                if let Some(region) = sim.world.regions.get(pos.region_idx) {
                    if region.terrain.get(pos.px, pos.py) == Some(Terrain::Settlement) {
                        let mut idx = 0;
                        for (si, _sett) in region.settlements.iter().enumerate() {
                            let spacing = 40 / region.settlements.len().max(1);
                            let sx = (spacing / 2 + si * spacing).min(39);
                            if sx == pos.px {
                                idx = si;
                                break;
                            }
                        }
                        return Some((pos.region_idx, idx));
                    }
                }
            }
        }
        None
    }

    pub fn critical_need_people(&self) -> Vec<(String, String, String, Need, f64)> {
        let mut out = Vec::new();
        if let Some(ref sim) = self.sim {
            let needs = [
                Need::Food,
                Need::Money,
                Need::Care,
                Need::Presence,
                Need::Safety,
            ];
            for region in &sim.world.regions {
                for settlement in &region.settlements {
                    for person in &settlement.people {
                        for need in &needs {
                            let val = person.needs.get(*need);
                            if val < 0.2 {
                                out.push((
                                    person.name.clone(),
                                    person.settlement.clone(),
                                    person.profession.clone(),
                                    *need,
                                    val,
                                ));
                            }
                        }
                    }
                }
            }
        }
        out.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key) => match self.screen {
                Screen::CharacterCreation => match key.code {
                    crossterm::event::KeyCode::Char('r') => {
                        self.reroll_player();
                    }
                    crossterm::event::KeyCode::Enter => {
                        if self.player_start.is_none() {
                            self.generate_player();
                        }
                        self.accept_player();
                    }
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.running = false;
                    }
                    _ => {}
                },
                Screen::World => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.running = false;
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Char('a') => {
                        if let Some(ref mut sim) = self.sim {
                            for _ in 0..10 {
                                sim.step();
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                        let idx = (c as usize) - ('1' as usize);
                        let list = self.settlement_list();
                        if let Some((ri, si, _)) = list.get(idx) {
                            self.enter_settlement(*ri, *si);
                        }
                    }
                    crossterm::event::KeyCode::Char('j') => {
                        self.enter_journal();
                    }
                    crossterm::event::KeyCode::Char('s') => {
                        self.save_game();
                    }
                    crossterm::event::KeyCode::Char('l') => {
                        self.load_game();
                    }
                    crossterm::event::KeyCode::Char('!') => {
                        self.enter_alerts();
                    }
                    crossterm::event::KeyCode::Char('m') => {
                        self.enter_map(0);
                    }
                    _ => {}
                },
                Screen::Map {
                    region_idx,
                    px: _,
                    py: _,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_map();
                    }
                    crossterm::event::KeyCode::Char('h') | crossterm::event::KeyCode::Left => {
                        self.move_player(-1, 0);
                    }
                    crossterm::event::KeyCode::Char('l') => {
                        self.move_player(1, 0);
                    }
                    crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                        self.move_player(0, -1);
                    }
                    crossterm::event::KeyCode::Char('j') => {
                        self.move_player(0, 1);
                    }
                    crossterm::event::KeyCode::Right => {
                        self.move_player(1, 0);
                    }
                    crossterm::event::KeyCode::Down => {
                        self.move_player(0, 1);
                    }
                    crossterm::event::KeyCode::Enter => {
                        if let Some((ri, si)) = self.player_on_settlement() {
                            self.enter_settlement(ri, si);
                        }
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Char(c @ '1'..='9') => {
                        let idx = (c as usize) - ('1' as usize);
                        if idx != region_idx {
                            self.enter_map(
                                idx.min(
                                    self.sim
                                        .as_ref()
                                        .map(|s| s.world.regions.len())
                                        .unwrap_or(1)
                                        - 1,
                                ),
                            );
                        }
                    }
                    crossterm::event::KeyCode::Char('M') => {
                        self.enter_overmap();
                    }
                    crossterm::event::KeyCode::Char('i') => {
                        self.enter_inventory();
                    }
                    crossterm::event::KeyCode::Char('g') => {
                        self.gather();
                    }
                    crossterm::event::KeyCode::Char('r') => {
                        self.rest();
                    }
                    crossterm::event::KeyCode::Char('c') => {
                        self.enter_craft();
                    }
                    _ => {}
                },
                Screen::Overmap { region_idx } => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Char('M') => {
                        self.exit_overmap();
                    }
                    crossterm::event::KeyCode::Char('h') | crossterm::event::KeyCode::Left => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(west) = region.neighbors.west {
                                    self.screen = Screen::Overmap { region_idx: west };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char('l') | crossterm::event::KeyCode::Right => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(east) = region.neighbors.east {
                                    self.screen = Screen::Overmap { region_idx: east };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(north) = region.neighbors.north {
                                    self.screen = Screen::Overmap { region_idx: north };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(south) = region.neighbors.south {
                                    self.screen = Screen::Overmap { region_idx: south };
                                }
                            }
                        }
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.screen = Screen::Map {
                            region_idx,
                            px: 20,
                            py: 10,
                        };
                    }
                    _ => {}
                },
                Screen::Inventory => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Char('i') => {
                        self.exit_inventory();
                    }
                    _ => {}
                },
                Screen::Craft { ref mut scroll } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_craft();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Char(c @ '1'..='9') => {
                        let idx = (c as usize) - ('1' as usize);
                        self.craft_recipe(idx);
                    }
                    _ => {}
                },
                Screen::Location {
                    ref mut scroll,
                    region_idx,
                    settlement_idx,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_settlement();
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.enter_npc(region_idx, settlement_idx, 0);
                    }
                    crossterm::event::KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                        let idx = (c as usize) - ('1' as usize);
                        self.enter_npc(region_idx, settlement_idx, idx);
                    }
                    crossterm::event::KeyCode::Char('m') => {
                        self.enter_market(region_idx, settlement_idx);
                    }
                    crossterm::event::KeyCode::Char('s') => {
                        if let Some(ref sim) = self.sim {
                            if let Some(region) = sim.world.regions.get(region_idx) {
                                if let Some(settlement) = region.settlements.get(settlement_idx) {
                                    if let Some(&service) = settlement.services.first() {
                                        self.use_service(service);
                                    } else {
                                        self.status_msg = Some("No services here".into());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Screen::Npc {
                    ref mut scroll,
                    region_idx,
                    settlement_idx,
                    person_idx,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_npc(region_idx, settlement_idx);
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        if let Some(ref mut sim) = self.sim {
                            sim.step();
                        }
                    }
                    crossterm::event::KeyCode::Char('t') => {
                        self.enter_talk(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::WorldAlerts { ref mut scroll } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_alerts();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::Journal { ref mut scroll } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_journal();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::Talk {
                    ref mut scroll,
                    region_idx,
                    settlement_idx,
                    person_idx,
                } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_talk(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Char('f') => {
                        self.give_food(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Char('c') => {
                        self.give_coin(region_idx, settlement_idx, person_idx);
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                Screen::Market { ref mut scroll, .. } => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.exit_market();
                    }
                    crossterm::event::KeyCode::Down => {
                        *scroll = scroll.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Up => {
                        *scroll = scroll.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Char(c) if ('1'..='6').contains(&c) => {
                        let idx = (c as usize) - ('1' as usize);
                        let items = ItemType::tradeable_items();
                        if let Some(&item) = items.get(idx) {
                            self.buy_item(item);
                        }
                    }
                    crossterm::event::KeyCode::Char(c) if ('a'..='f').contains(&c) => {
                        let idx = (c as usize) - ('a' as usize);
                        let items = ItemType::tradeable_items();
                        if let Some(&item) = items.get(idx) {
                            self.sell_item(item);
                        }
                    }
                    _ => {}
                },
                Screen::Encounter => match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.resolve_encounter(EncounterAction::Flee);
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.resolve_encounter(EncounterAction::Flee);
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        if let Some(enc) = self.encounter {
                            for action in enc.kind.available_actions() {
                                if action.key() == c {
                                    self.resolve_encounter(action);
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Screen::Collapse => match key.code {
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Enter => {
                        self.dismiss_collapse();
                    }
                    _ => {}
                },
                Screen::GameOver => match key.code {
                    crossterm::event::KeyCode::Char('r') => {
                        self.restart_game();
                    }
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        self.running = false;
                    }
                    _ => {}
                },
            },
            AppEvent::Tick => {}
        }
    }
}
