use crate::model::{GodName, ItemType};
use crate::sim::hints;

use super::*;

impl App {
    pub fn enter_market(&mut self, region_idx: usize, settlement_idx: usize) {
        self.screen = Screen::Market {
            region_idx,
            settlement_idx,
            scroll: 0,
        };
    }

    pub fn exit_market(&mut self) {
        let region_idx = self.player_pos.map(|p| p.region_idx).unwrap_or(0);
        self.screen = Screen::World { region_idx };
    }

    /// Supply effect of arrived caravans on the current settlement's price for
    /// an item (more goods in town → lower price). 1.0 when no caravan applies.
    fn caravan_price_modifier(&self, item: ItemType) -> f64 {
        let Some(name) = self.current_settlement().map(|s| s.name.clone()) else {
            return 1.0;
        };
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let mut m = 1.0;
        if let Some(ref sim) = self.sim {
            for c in &sim.caravans {
                if c.destination == name && c.has_arrived(tick) {
                    m *= c.price_modifier(item, tick);
                }
            }
        }
        m
    }

    /// Escalation: at deep hostility the market simply closes to the player.
    fn market_barred(&self) -> bool {
        // Two seasons of unpaid hearth-tax and the polity shuts the stalls to
        // you (#396); a deep grudge does the same.
        if self.tax_unpaid_seasons >= 2 {
            return true;
        }
        self.current_settlement_people()
            .map(|p| self.inter_people_bias.effective_bias(p) < -0.25)
            .unwrap_or(false)
    }

    pub fn buy_item(&mut self, item: ItemType) {
        if !item.tradeable() {
            self.status_msg = Some("Cannot buy that".into());
            return;
        }
        if self.market_barred() {
            self.status_msg = Some("The market is closed to your kind here.".into());
            return;
        }
        // Single source of truth with the displayed quote.
        let price = self.quote_buy_price(item);
        if let Some(ref mut ps) = self.player_start {
            if ps.inventory.remove(ItemType::Coin, price) {
                ps.inventory.add(item, 1);
                self.advance_clock_hour();
                self.fire_hint(hints::HINT_FIRST_TRADE);
                self.check_quests_on_tick();
                self.status_msg =
                    Some(format!("Bought 1 {} for {} coins (1h)", item.name(), price));
                self.god_affinity.adjust(GodName::Masa, 0.02);
                // Honest trade slowly mends what tension breaks.
                if let Some(np) = self.current_settlement_people() {
                    self.inter_people_bias.mod_toward(np, 0.005);
                }
                self.charge_gift_for_trade();
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
        if self.market_barred() {
            self.status_msg = Some("The market is closed to your kind here.".into());
            return;
        }
        // Single source of truth with the displayed quote (spread-clamped so
        // selling never pays more than buying costs).
        let price = self.quote_sell_price(item);
        if let Some(ref mut ps) = self.player_start {
            if ps.inventory.remove(item, 1) {
                ps.inventory.add(ItemType::Coin, price);
                self.advance_clock_hour();
                self.fire_hint(hints::HINT_FIRST_TRADE);
                self.check_quests_on_tick();
                self.status_msg = Some(format!("Sold 1 {} for {} coins (1h)", item.name(), price));
                self.god_affinity.adjust(GodName::Masa, 0.01);
                if let Some(np) = self.current_settlement_people() {
                    self.inter_people_bias.mod_toward(np, 0.005);
                }
                self.charge_gift_for_trade();
            } else {
                self.status_msg = Some(format!("No {} to sell", item.name()));
            }
        }
    }

    /// Try to palm an item off a market stall. The witness roll decides:
    /// unseen takes it clean, rumored takes it with a whisper attached, seen
    /// gets nothing and a name for thieving. Crime is a choice with weight.
    pub fn steal_item(&mut self, item: ItemType) {
        if !item.tradeable() {
            self.status_msg = Some("Cannot steal that".into());
            return;
        }
        if self.current_settlement().is_none() {
            self.status_msg = Some("Nothing here to steal.".into());
            return;
        }
        let tick = self.sim.as_ref().map_or(0, |s| s.world.tick);
        let mut rng = crate::rng::SeedRng::new(self.seed.wrapping_add(tick)).fork_for("steal");
        let roll = rng.gen_range(100);
        let pid = self
            .player_start
            .as_ref()
            .map(|ps| ps.person.id.clone())
            .unwrap_or_default();
        let sid = self
            .current_settlement()
            .map(|s| s.id.clone())
            .unwrap_or_default();
        let npc_people = self.current_settlement_people();
        // More eyes in a city: witness odds scale with the place. A hamlet's
        // lanes are empty half the day; a city street is never unwatched.
        let (clean_under, whisper_under) = match self
            .current_settlement()
            .map(|s| s.size.clone())
            .unwrap_or_default()
            .as_str()
        {
            "hamlet" => (60, 85),
            "town" => (40, 70),
            "city" => (25, 60),
            _ => (50, 80), // village — the old odds
        };
        self.advance_clock_hour();
        if roll < clean_under {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(item, 1);
            }
            self.god_affinity.adjust(GodName::Masa, -0.03);
            self.status_msg = Some(format!("No one saw. The {} is yours.", item.name()));
        } else if roll < whisper_under {
            if let Some(ref mut ps) = self.player_start {
                ps.inventory.add(item, 1);
            }
            self.god_affinity.adjust(GodName::Masa, -0.05);
            if let Some(ref mut sim) = self.sim {
                sim.reputation.adjust_local(&pid, &sid, -0.05);
            }
            self.status_msg = Some(format!(
                "The {} is yours — but someone is whispering already.",
                item.name()
            ));
        } else {
            self.god_affinity.adjust(GodName::Masa, -0.05);
            if let Some(ref mut sim) = self.sim {
                sim.reputation.adjust_local(&pid, &sid, -0.15);
            }
            if let Some(np) = npc_people {
                self.inter_people_bias.mod_toward(np, -0.03);
            }
            self.status_msg = Some("Caught with your hand out. They will remember this.".into());
        }
    }

    pub fn reputation_in_current_settlement(&self) -> f64 {
        let mut rep = 0.5;
        if let (Some(ref ps), Some(ref sim), Some(pos)) =
            (&self.player_start, &self.sim, self.player_pos)
        {
            if let Some(region) = sim.world.regions.get(pos.region_idx) {
                if let Some(settlement) = region.settlements.first() {
                    rep = sim.reputation.get(&ps.person.id, &settlement.id);
                }
            }
        }
        rep
    }

    pub fn politics_price_modifier(&self) -> f64 {
        if let (Some(ref sim), Some(pos)) = (&self.sim, self.player_pos) {
            if let Some(region) = sim.world.regions.get(pos.region_idx) {
                if let Some(settlement) = region.settlements.first() {
                    return settlement.politics.price_modifier();
                }
            }
        }
        1.0
    }

    /// Whether the player carries the scale-hand — the Väylä weight-sense that
    /// reads true value in a trade (#439).
    fn scale_hand(&self) -> bool {
        self.gift.sense().map(|s| s.aids_trade()).unwrap_or(false)
    }

    /// A gifted crafter living in this settlement makes their craft-goods truer
    /// and cheaper here (#441): an iron-ear smith's tools, a root-eye's salves.
    /// Returns the price multiplier for the item (1.0 if no matching gift).
    fn settlement_gift_discount(&self, item: ItemType) -> f64 {
        let Some(s) = self.current_settlement() else {
            return 1.0;
        };
        for p in &s.people {
            if let Some(sense) = p.gift.sense() {
                let matches_goods = match sense {
                    crate::model::CraftSense::IronEar => item == ItemType::Tool,
                    crate::model::CraftSense::RootEye => {
                        matches!(item, ItemType::Bandage | ItemType::Salve)
                    }
                    _ => false,
                };
                if matches_goods {
                    return 0.85;
                }
            }
        }
        1.0
    }

    /// Reading the true price is the gift at work — and the gift costs the body.
    /// Only the scale-hand pays here (#439).
    fn charge_gift_for_trade(&mut self) {
        if self.scale_hand() {
            if let Some(note) = self.use_gift() {
                if let Some(m) = self.status_msg.as_mut() {
                    m.push_str(&note);
                }
            }
        }
    }

    /// How well coin trades in the ruling polity's markets (canon: no universal
    /// currency). 1.0 in coin economies; a discount where coin is debased
    /// (Remnant) or a foreign convenience (grain/in-kind economies).
    pub fn coin_value_here(&self) -> f64 {
        self.sim
            .as_ref()
            .map(|s| s.world.polity.coin_value_modifier())
            .unwrap_or(1.0)
    }

    /// What the market charges the player. Includes ALL the modifiers the
    /// transaction applies (politics + caravan supply too — the quotes used to
    /// omit those, so the displayed price could differ from the charged one).
    pub fn quote_buy_price(&self, item: ItemType) -> u32 {
        self.buy_price_inner(item, true)
    }

    /// The buy price, optionally without the scale-hand discount. The sell-side
    /// spread clamp uses the gift-free price, so a scale-hand's cheaper buying
    /// does not also drag down what it can sell for (#439).
    fn buy_price_inner(&self, item: ItemType, with_gift: bool) -> u32 {
        let base = item.base_price();
        let inter_mod = self
            .current_settlement_people()
            .map(|sp| self.inter_people_bias.price_modifier(sp))
            .unwrap_or(1.0);
        let rep_mod =
            crate::sim::signals::reputation_price_modifier(self.reputation_in_current_settlement());
        // The fortunate strike a slightly better bargain — a few coppers, not a
        // fortune, and bounded; the cursed pay a little over the odds.
        let luck = 1.0 - self.fortune.value() * 0.08;
        // Coin worth less here means it takes more of it to buy: divide the
        // price by how well coin trades in this polity.
        let coin = self.coin_value_here();
        // A market fair makes goods cheaper this season (#417).
        let event = self
            .current_world_event()
            .map(|e| e.buy_price_modifier())
            .unwrap_or(1.0);
        // The scale-hand reads the fair price and buys under it (#439).
        let gift = if with_gift && self.scale_hand() {
            0.90
        } else {
            1.0
        };
        // A gifted crafter in town makes their goods truer and cheaper (#441).
        let town_gift = self.settlement_gift_discount(item);
        let m = inter_mod
            * rep_mod
            * self.politics_price_modifier()
            * self.caravan_price_modifier(item)
            * self.food_scarcity_modifier(item)
            * luck
            * event
            * gift
            * town_gift
            / coin;
        ((base as f64 * m).ceil() as u32).max(1)
    }

    /// Hunger in the settlement moves the price of what can be eaten: lean
    /// stores raise Food/Herb prices, full ones lower them.
    fn food_scarcity_modifier(&self, item: ItemType) -> f64 {
        if !matches!(item, ItemType::Food | ItemType::Herb) {
            return 1.0;
        }
        self.current_settlement()
            .map(|s| s.food_scarcity_modifier())
            .unwrap_or(1.0)
    }

    /// What the market pays the player — always below the buy price, merchants
    /// take a margin. Without the clamp, high reputation inverted the spread
    /// (buy at 0.6x base, sell at 1.4x) and buy->sell was an infinite-coin loop.
    pub fn quote_sell_price(&self, item: ItemType) -> u32 {
        let base = item.base_price();
        let inter_mod = self
            .current_settlement_people()
            .map(|bp| 2.0 - self.inter_people_bias.price_modifier(bp))
            .unwrap_or(1.0);
        let rep_mod = 2.0
            - crate::sim::signals::reputation_price_modifier(
                self.reputation_in_current_settlement(),
            );
        // The fortunate sell a little dearer — the mirror of their buying luck.
        let luck = 1.0 + self.fortune.value() * 0.08;
        // Coin worth less here means you receive fewer of it for your goods.
        let coin = self.coin_value_here();
        // The scale-hand will not be short-weighted: it sells a little dearer.
        let gift = if self.scale_hand() { 1.10 } else { 1.0 };
        let m = inter_mod
            * rep_mod
            * self.politics_price_modifier()
            * self.caravan_price_modifier(item)
            * self.food_scarcity_modifier(item)
            * luck
            * coin
            * gift;
        let raw = ((base as f64 * m).floor() as u32).max(1);
        // Clamp against the gift-free buy price so the scale-hand's cheaper
        // buying does not also cap what it can sell for (#439).
        let buy = self.buy_price_inner(item, false);
        if buy > 1 {
            raw.clamp(1, buy - 1)
        } else {
            1
        }
    }
}
