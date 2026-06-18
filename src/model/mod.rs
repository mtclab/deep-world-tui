use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub mod calendar;
pub mod companion;
pub mod discovery;
pub mod economy;
pub mod encounter;
pub mod fortune;
pub mod gift;
pub mod memorial;
pub mod person;
pub mod polity;
pub mod province;
pub mod quest;
pub mod relation;
pub mod terrain;
pub mod vitals;
pub mod weather;
pub mod wildlife;

// Re-export everything so call sites don't change
pub use calendar::WorldEvent;
pub use companion::*;
pub use discovery::{Discovery, DiscoveryKind, DiscoveryStore};
pub use economy::*;
pub use encounter::*;
pub use fortune::Fortune;
pub use gift::{CraftSense, Gift};
pub use person::*;
pub use polity::Polity;
pub use quest::{Quest, QuestKind, QuestReward};
pub use terrain::*;
pub use vitals::*;
pub use weather::*;
pub use wildlife::*;

// Keep existing re-exports from submodules that define their own types
pub use memorial::Memorial;
pub use relation::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct World {
    pub seed: u64,
    pub tick: u64,
    pub regions: Vec<Region>,
    pub charts_version: String,
    #[serde(default)]
    pub region_cols: usize,
    /// The territorial power this province answers to — its tax, its law. Old
    /// saves default to the Remnant until a fresh world re-derives it.
    #[serde(default)]
    pub polity: Polity,
}

impl World {
    pub fn set_wants_for_person(
        &mut self,
        person_id: &str,
        wants: Vec<crate::sim::wants::NpcWant>,
    ) {
        for region in &mut self.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    if person.id == person_id {
                        person.wants = wants;
                        return;
                    }
                }
            }
        }
    }

    pub fn tick_npc_wants(&mut self, current_tick: u64, ticks_per_day: u64) {
        let day = (current_tick / ticks_per_day.max(1)) as u32;
        let season = if day == 0 {
            Season::Thaw
        } else {
            Season::from_day(day)
        };
        let is_frost = matches!(season, Season::Frost);
        for region in &mut self.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    crate::sim::wants::tick_wants(
                        &mut person.wants,
                        current_tick,
                        ticks_per_day,
                        is_frost,
                    );
                }
            }
        }
    }

    pub fn recompute_all_schedules(&mut self, hour: u32) {
        for region in &mut self.regions {
            for settlement in &mut region.settlements {
                for person in &mut settlement.people {
                    let base = person.schedule.clone();
                    person.schedule =
                        crate::sim::wants::recompute_schedule(&person.wants, &base, hour);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RegionNeighbors {
    pub north: Option<usize>,
    pub south: Option<usize>,
    pub east: Option<usize>,
    pub west: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub region_type: String,
    #[serde(default)]
    pub region_subtype: String,
    pub description: String,
    pub settlements: Vec<Settlement>,
    #[serde(default)]
    pub terrain: TerrainMap,
    #[serde(default)]
    pub neighbors: RegionNeighbors,
    #[serde(default)]
    pub structures: Vec<crate::sim::structures::Structure>,
    /// Today's sky over this region. Set daily by the weather-front system
    /// (persistence + drift from neighbors) instead of an hourly hash, so
    /// weather has continuity — fronts arrive, sit, and move on.
    #[serde(default = "default_region_weather")]
    pub weather: Weather,
    /// How much wild game the region still carries (1.0 = untouched). Traps,
    /// hunting companions, and beast-handlers draw it down; it recovers with
    /// the seasons. Over-trap a valley and it goes quiet.
    #[serde(default = "default_game_richness")]
    pub game_richness: f64,
    /// A march (#630): an edge region beyond the province's settled reach —
    /// wilderness, few or no towns, where the frontier's bands and holds (#623)
    /// and the deep wild's dreads concentrate. Ungoverned country, not by fiat
    /// but because it lies past the towns' grasp.
    #[serde(default)]
    pub is_march: bool,
}

fn default_game_richness() -> f64 {
    1.0
}

fn default_region_weather() -> Weather {
    Weather::Clear
}

impl Region {
    pub fn danger_level(&self) -> DangerLevel {
        let total = self.terrain.width * self.terrain.height;
        if total == 0 {
            return DangerLevel::Safe;
        }
        let mut hostile = 0u32;
        for y in 0..self.terrain.height {
            for x in 0..self.terrain.width {
                if let Some(
                    Terrain::Forest
                    | Terrain::Mountain
                    | Terrain::Swamp
                    | Terrain::Cave
                    | Terrain::DeepDesert,
                ) = self.terrain.get(x, y)
                {
                    hostile += 1;
                }
            }
        }
        let ratio = hostile as f64 / total as f64;
        if ratio > 0.5 {
            DangerLevel::Dangerous
        } else if ratio > 0.25 {
            DangerLevel::Risky
        } else {
            DangerLevel::Safe
        }
    }

    pub fn danger_level_biased(&self, player_people: PeopleKind) -> DangerLevel {
        let base = self.danger_level();
        let dominant = self.settlements.first().and_then(|s| s.people.first());
        let bias = dominant.map_or(0.0, |p| {
            player_people.bias_toward(PeopleKind::from_name(&p.people))
        });
        match base {
            DangerLevel::Safe => {
                if bias < -0.15 {
                    DangerLevel::Risky
                } else {
                    DangerLevel::Safe
                }
            }
            DangerLevel::Risky => {
                if bias < -0.15 {
                    DangerLevel::Dangerous
                } else if bias > 0.05 {
                    DangerLevel::Safe
                } else {
                    DangerLevel::Risky
                }
            }
            DangerLevel::Dangerous => {
                if bias > 0.05 {
                    DangerLevel::Risky
                } else {
                    DangerLevel::Dangerous
                }
            }
        }
    }
}
