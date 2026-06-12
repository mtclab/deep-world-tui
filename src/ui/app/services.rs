use crate::model::{GodName, InterPeopleBias, ItemType, SettlementService, Terrain};

use super::*;

impl App {
    pub fn enter_journal(&mut self) {
        self.screen = Screen::Journal { scroll: 0 };
    }

    pub fn exit_journal(&mut self) {
        self.screen = self.world_screen();
    }

    pub fn use_service(&mut self, service: SettlementService) {
        // Time-of-day gate: Night/DeepNight — doors shut, sleep. Refuse with journal line.
        let tod = crate::model::TimeOfDay::from_hour(self.clock.hour);
        if tod.blocks_services() {
            let line = "The door is shut. I sleep.".to_string();
            self.status_msg = Some(line.clone());
            if let Some(ref mut sim) = self.sim {
                sim.log_journal(sim.world.tick, line);
            }
            return;
        }
        // Check if service provider is available at current hour
        if let Some(ref sim) = self.sim {
            if let Some(pos) = self.player_pos {
                if let Some(region) = sim.world.regions.get(pos.region_idx) {
                    if let Some(settlement) = region.settlements.first() {
                        if let Some(person) = settlement.people.first() {
                            if !person.schedule.is_available_at_hour(self.clock.hour) {
                                self.status_msg = Some(format!(
                                    "The {} is closed. {} is {}.",
                                    service.label(),
                                    person.name,
                                    person.schedule.activity_at_hour(self.clock.hour).name()
                                ));
                                return;
                            }
                        }
                    }
                }
            }
        }
        if let Some(service_people) = service.people() {
            if self.inter_people_bias.player_people != service_people {
                let bias = self
                    .inter_people_bias
                    .player_people
                    .bias_toward(service_people);
                if bias < -0.05 {
                    self.status_msg = Some(format!(
                        "The {} is for {} hands only. You are not welcome.",
                        service.label(),
                        service_people.label()
                    ));
                    return;
                }
            }
        }
        if let Some(npc_people) = self.current_settlement_people() {
            let mut bias = self.inter_people_bias.effective_bias(npc_people);
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
            let bias = self.inter_people_bias.effective_bias(npc_people);
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
            let rep_mod = crate::sim::signals::reputation_price_modifier(
                self.reputation_in_current_settlement(),
            );
            let combined = price_mod * rep_mod;
            let extra = (service.cost() as f64 * combined).ceil() as u32;
            cost = cost.saturating_add(extra);
        }
        // Festival days: the doors are open and everything is half price.
        if self
            .current_settlement()
            .is_some_and(|s| s.in_festival(self.clock.day))
        {
            cost = (cost / 2).max(1);
        }
        // A resident pays neighbor's prices: owning a finished house on this
        // settlement's ground counts like a friend's vouching.
        let is_resident = self
            .player_pos
            .map(|pos| {
                self.sim
                    .as_ref()
                    .and_then(|s| s.world.regions.get(pos.region_idx))
                    .map(|r| {
                        r.structures.iter().any(|st| {
                            !st.is_npc_built
                                && matches!(
                                    st.kind,
                                    crate::sim::structures::BuildKind::Cabin
                                        | crate::sim::structures::BuildKind::Longhouse
                                        | crate::sim::structures::BuildKind::Home
                                )
                                && r.terrain.get(st.x as usize, st.y as usize)
                                    == Some(Terrain::Settlement)
                        })
                    })
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        // A friend in town vouches for you: a coin off, never below one.
        let has_friend = is_resident
            || self.current_settlement().is_some_and(|s| {
                s.people.iter().any(|p| {
                    self.sim
                        .as_ref()
                        .and_then(|sim| sim.npc_memories.get(&p.id))
                        .map(|m| m.cumulative_trust() >= 0.15)
                        .unwrap_or(false)
                })
            });
        if has_friend {
            cost = cost.saturating_sub(1).max(1);
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
                // Taverns are where word travels. Prefer a rumor grounded in
                // the actual world (famine, caravans, festivals, construction)
                // — actionable information — and fall back to flavor.
                let day = self.clock.day;
                let mut heard: Option<String> = None;
                if let Some(ref mut sim) = self.sim {
                    let tick = sim.world.tick;
                    let text =
                        crate::sim::rumors::informed_rumor(sim, day, tick).unwrap_or_else(|| {
                            let mut rng = crate::rng::SeedRng::new(sim.world.seed)
                                .fork_for(&format!("tavern-rumor-{tick}"));
                            crate::sim::journal::rumor_text(&mut rng)
                        });
                    sim.log(tick, crate::sim::journal::Voice::Rumor, text.clone());
                    heard = Some(text);
                }
                self.status_msg = Some(match heard {
                    Some(r) => {
                        format!("Rested at tavern ({} coins). You overhear: \"{}\"", cost, r)
                    }
                    None => format!("Rested at tavern (+energy, +hunger, 2h, {} coins)", cost),
                });
            }
            SettlementService::Temple => {
                self.vitals.hunger = (self.vitals.hunger + 0.5).min(1.0);
                self.vitals.energy = (self.vitals.energy + 0.3).min(1.0);
                self.advance_clock(3);
                // Restitution is more Masa than absolution: low standing is
                // mended only against a donation scaled to the offense, paid
                // into the poor-box on top of the visit price. No coin, no
                // ledger eased — the blessing is free, the making-good is not.
                let pid = self
                    .player_start
                    .as_ref()
                    .map(|ps| ps.person.id.clone())
                    .unwrap_or_default();
                let sid = self
                    .current_settlement()
                    .map(|s| s.id.clone())
                    .unwrap_or_default();
                let mut penance_note = String::new();
                let rep = self.reputation_in_current_settlement();
                if rep < 0.45 {
                    let donation = ((0.45 - rep) * 40.0).ceil() as u32;
                    let paid = self
                        .player_start
                        .as_mut()
                        .map(|ps| ps.inventory.remove(ItemType::Coin, donation))
                        .unwrap_or(false);
                    if paid {
                        if let Some(ref mut sim) = self.sim {
                            sim.reputation.adjust_local(&pid, &sid, 0.05);
                        }
                        penance_note =
                            format!(" Restitution made: {donation} coins to the poor-box.");
                    } else {
                        penance_note = format!(
                            " The keeper names your restitution: {donation} coins. \
                             You cannot pay it; the ledger stands."
                        );
                    }
                }
                // The temple healers also tend the sick — clear active illness.
                let cured = self
                    .player_start
                    .as_mut()
                    .map(|ps| {
                        let had = !ps.person.illnesses.is_empty();
                        ps.person.illnesses.clear();
                        had
                    })
                    .unwrap_or(false);
                self.status_msg = Some(if cured {
                    format!(
                        "Blessed and healed at temple (illness cured, 3h, {cost} coins){penance_note}"
                    )
                } else {
                    format!("Blessed at temple (+hunger, +energy, 3h, {cost} coins){penance_note}")
                });
            }
            SettlementService::Forge => {
                self.god_affinity.adjust(GodName::Oltzed, 0.02);
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Iron, 2);
                    let mut repaired = Vec::new();
                    for tool in [
                        ItemType::Iron,
                        ItemType::Wood,
                        ItemType::Stone,
                        ItemType::Cloth,
                    ] {
                        let cost = ps.inventory.repair_cost(tool);
                        if cost > 0 && ps.inventory.get(ItemType::Coin) >= cost {
                            ps.inventory.remove(ItemType::Coin, cost);
                            ps.inventory.repair(tool);
                            repaired.push(tool.name());
                        }
                    }
                    let repair_msg = if repaired.is_empty() {
                        String::new()
                    } else {
                        format!(" Repaired: {}.", repaired.join(", "))
                    };
                    ps.inventory.add(ItemType::Iron, 0); // no-op to ensure key exists
                    self.advance_clock(3);
                    self.status_msg = Some(format!(
                        "Worked at the forge (+2 Iron, 3h, {} coins){}",
                        cost, repair_msg
                    ));
                } else {
                    self.advance_clock(3);
                    self.status_msg = Some(format!("Worked at the forge (3h, {} coins)", cost));
                }
            }
            SettlementService::Hearth => {
                self.vitals.hunger = (self.vitals.hunger + 0.6).min(1.0);
                self.vitals.energy = (self.vitals.energy + 0.5).min(1.0);
                self.god_affinity.adjust(GodName::Oltzed, 0.03);
                self.advance_clock(2);
                self.status_msg = Some(format!(
                    "Warmed by the hearth (+hunger, +energy, 2h, {} coins)",
                    cost
                ));
            }
            SettlementService::TrapWorkshop => {
                self.god_affinity.adjust(GodName::Keuru, 0.03);
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Herb, 2);
                }
                self.advance_clock(2);
                self.status_msg = Some(format!(
                    "Learned trapping at the workshop (+2 Herb, 2h, {} coins)",
                    cost
                ));
            }
            SettlementService::Archive => {
                self.vitals.energy = (self.vitals.energy + 0.4).min(1.0);
                self.god_affinity.adjust(GodName::Sampsa, 0.02);
                self.advance_clock(3);
                self.status_msg = Some(format!(
                    "Studied in the archive (+energy, Sampsa +0.02, 3h, {} coins)",
                    cost
                ));
            }
            SettlementService::TradePost => {
                if let Some(ref mut ps) = self.player_start {
                    ps.inventory.add(ItemType::Coin, 2);
                }
                self.god_affinity.adjust(GodName::Masa, 0.02);
                self.advance_clock(2);
                self.status_msg = Some(format!(
                    "Traded at the post (+2 Coin, Masa +0.02, 2h, {} coins)",
                    cost
                ));
            }
            SettlementService::Shrine => {
                self.vitals.hunger = (self.vitals.hunger + 0.3).min(1.0);
                self.vitals.energy = (self.vitals.energy + 0.3).min(1.0);
                self.god_affinity.adjust(GodName::Kukri, 0.03);
                self.advance_clock(2);
                self.status_msg = Some(format!(
                    "Prayed at the shrine (+hunger, +energy, Kukri +0.03, 2h, {} coins)",
                    cost
                ));
            }
        }
    }
}
