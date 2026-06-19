use crate::charts::Charts;
use crate::model::{
    Collapse, DeathCause, Encounter, EncounterLog, GameClock, GodAffinity, InterPeopleBias,
    PlayerPos, PlayerStart, PlayerVitals,
};
use crate::rng::SeedRng;
use crate::save::LineageRecord;
use crate::sim::collapse_log::CollapseEvent;
use crate::sim::hints::HintTracker;
use crate::sim::SimState;

mod clock;
mod encounters;
mod events;
mod homestead;
mod items;
mod lifecycle;
mod market;
mod navigation;
mod persistence;
mod services;
mod social;
mod vow;

#[derive(Clone)]
pub enum Screen {
    TitleScreen,
    SaveBrowser {
        scroll: u16,
        delete_confirm: Option<usize>,
    },
    SaveSlots {
        scroll: u16,
    },
    RestPrompt {
        hours: u32,
    },
    CharacterCreation,
    World {
        region_idx: usize,
    },
    Overmap {
        region_idx: usize,
    },
    Inventory,
    Craft {
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
    Help,
    Settings,
    EncounterLog {
        scroll: u16,
    },
    /// The faith ledger (#457): a read-only panel of where you stand with each
    /// of the Five — affinity, devotion rank, the grace each tier grants.
    Faith {
        scroll: u16,
    },
    /// The great city reached by a journey (#456): a read-only panel of its
    /// scale, quarters, services, and long-haul market — `idx` into CANON_CITIES.
    CityVisit {
        idx: usize,
        scroll: u16,
    },
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
    pub collapse: Option<Collapse>,
    pub death_cause: Option<DeathCause>,
    pub encounter: Option<Encounter>,
    /// When the current encounter is an outlaw band crossed in the field (#630
    /// slice 5), the id of that band — so driving them off strikes the living
    /// band and answers its bounty. `None` for an ordinary cutpurse.
    pub encounter_band: Option<String>,
    pub god_affinity: GodAffinity,
    pub inter_people_bias: InterPeopleBias,
    pub llm_enabled: bool,
    pub llm_endpoint: String,
    pub llm_model: String,
    pub monochrome: bool,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub language: String,
    pub audio_enabled: bool,
    pub audio_volume: f32,
    pub previous_screen: Option<Screen>,
    pub encounters_had: u32,
    pub encounter_log: EncounterLog,
    /// The player's worked fields (plant/harvest at the homestead).
    pub player_farms: Vec<crate::model::economy::PlayerFarm>,
    /// Settlers camped by the homestead, drawn out of real settlements —
    /// at ~12 souls the world recognizes the place and a hamlet is born.
    pub homestead_settlers: Vec<crate::model::Person>,
    /// Word has gone out about the homestead (the rumor precedes the wagons).
    pub homestead_rumored: bool,
    /// Last day the founding check ran (it asks the roads every ten days).
    pub founding_check_day: u32,
    /// The person the player is wed to, if anyone (their Person id).
    pub spouse_id: Option<String>,
    /// Day the player was widowed (0 = never); grief holds the door a while.
    pub widowed_day: u32,
    /// Children of the house, by birth order. The eldest grown child is the
    /// heir before any friend is.
    pub household_children: Vec<crate::model::HouseholdChild>,
    /// Fractional walking time owed to the clock: on the fine grid two open
    /// tiles pass to the hour, and the half-hours accumulate here.
    pub travel_debt: f64,
    pub collapses_had: u32,
    pub collapse_log: Vec<CollapseEvent>,
    pub lineage: Vec<LineageRecord>,
    pub save_entries: Vec<crate::save::SaveEntry>,
    pub hint_tracker: HintTracker,
    pub milestones: crate::sim::milestones::MilestoneTracker,
    pub explored: Vec<crate::model::ExploredMap>,
    /// The peoples of the Five whose enclaves you've entered — so the
    /// first-visit lore reveal fires once per people (#454).
    pub enclaves_seen: Vec<crate::model::PeopleKind>,
    /// The god the player has sworn a vow to (#457): a Blessed devotee may bind
    /// to one of the Five for a lasting boon, at the cost of forsaking the rest.
    /// `None` until sworn; broken if the kept god slips below Devoted.
    pub god_vow: Option<crate::model::GodName>,
    /// A craft-sense learned by apprenticing to a master (#527/#529) — a lesser,
    /// taught echo of the innate Gift: it steadies the bench in that sense's
    /// craft (lower botch), but costs the body nothing and grants no Gift. `None`
    /// until learned; only the giftless can take one, and only one.
    pub learned_sense: Option<crate::model::CraftSense>,
    pub elder: bool,
    /// Age in years at the start of the current life (from age_band).
    pub start_age_years: u32,
    /// Calendar day on which the current life began (for elapsed-age math).
    pub birth_day: u32,
    /// Rolled maximum age for the current life; death of old age at/after this.
    pub lifespan_years: u32,
    /// The luck this life was born with — hidden, read only in omens. Tilts
    /// every risk a little; the cautious are safer, never safe.
    pub fortune: crate::model::Fortune,
    /// The craft-gift this life was born with — almost always none. Innate,
    /// hidden, shows in childhood or never (#426). The craft it grants costs
    /// the body to use (#427).
    pub gift: crate::model::Gift,
    /// Today's accumulated gift-strain: working the gift past a day's measure
    /// brings the flame-fever (#427). Resets at the day's turn.
    pub gift_strain: f64,
    /// Consecutive days the gift was worked to the bone — sustained overuse
    /// settles into the chronic iron-ache (#427).
    pub gift_overworked_days: u32,
    /// Whether the gift has surfaced to its bearer yet — it reveals itself the
    /// first time it is used (#431). Ephemeral; re-announces on a fresh load.
    pub gift_revealed: bool,
    /// Day the last omen showed, so the sky does not babble every step.
    pub last_omen_day: u32,
    /// Consecutive seasons a resident has owed the polity its hearth-tax and
    /// not paid in full. Drives the debt ladder: market closed, then residency
    /// revoked. Reset to zero on a season paid clear.
    pub tax_unpaid_seasons: u32,
    /// Day of the last hearth-tax assessment, so a season is reckoned once.
    pub last_tax_day: u32,
    pub tick_count: u64,
    pub flash_frames: u8,
    pub perf_slow_frames: u32,
    pub perf_last_render_us: u64,
    /// Re-entrancy guard: a collapse advances the clock for its unconscious
    /// hours, which must not recursively re-trigger another collapse.
    in_collapse: bool,
    seed: u64,
    charts: Charts,
    player_rng: Option<SeedRng>,
}

/// Calendar days that elapse per year of the player's life. Aging is decoupled
/// from the literal hour/day calendar so a full life is reachable in normal play.
const AGING_DAYS_PER_LIFE_YEAR: u32 = 3;
/// Years before death at which the player becomes an elder.
const ELDER_BAND_YEARS: u32 = 8;
/// Longest single rest the player may take (a full night).
const MAX_REST_HOURS: u32 = 12;
/// Default rest duration the picker opens on.
const DEFAULT_REST_HOURS: u32 = 6;

/// Starting age (years) for a generated age band.
fn start_age_from_band(band: &str) -> u32 {
    match band.to_ascii_lowercase().as_str() {
        "youth" | "young" => 18,
        "elder" | "old" => 58,
        _ => 32, // adult / unknown
    }
}

impl App {
    pub fn new(seed: u64, charts: Charts) -> Self {
        let settings = crate::ui::AppSettings::load();
        let user_config = crate::config::load();
        let monochrome = settings.monochrome || user_config.display.monochrome;
        let high_contrast = settings.high_contrast || user_config.display.high_contrast;
        let reduced_motion = settings.reduced_motion || user_config.display.reduced_motion;
        let player_rng = SeedRng::new(seed);
        App {
            sim: None,
            player_start: None,
            running: true,
            tick_interval: 100,
            screen: Screen::TitleScreen,
            status_msg: None,
            player_pos: None,
            clock: GameClock::default(),
            vitals: PlayerVitals::default(),
            encounter: None,
            encounter_band: None,
            collapse: None,
            death_cause: None,
            god_affinity: GodAffinity::new(),
            inter_people_bias: InterPeopleBias::default(),
            llm_enabled: settings.llm_enabled,
            llm_endpoint: settings.llm_endpoint,
            llm_model: settings.llm_model,
            monochrome,
            high_contrast,
            reduced_motion,
            language: settings.language,
            audio_enabled: settings.audio_enabled,
            audio_volume: settings.audio_volume,
            previous_screen: None,
            encounters_had: 0,
            encounter_log: EncounterLog::new(),
            player_farms: Vec::new(),
            enclaves_seen: Vec::new(),
            god_vow: None,
            learned_sense: None,
            homestead_settlers: Vec::new(),
            homestead_rumored: false,
            founding_check_day: 0,
            spouse_id: None,
            widowed_day: 0,
            household_children: Vec::new(),
            travel_debt: 0.0,
            collapses_had: 0,
            collapse_log: Vec::new(),
            lineage: Vec::new(),
            save_entries: Vec::new(),
            hint_tracker: HintTracker::default(),
            milestones: crate::sim::milestones::MilestoneTracker::new(),
            explored: Vec::new(),
            elder: false,
            start_age_years: 0,
            birth_day: 0,
            lifespan_years: 0,
            fortune: crate::model::Fortune::default(),
            gift: crate::model::Gift::default(),
            gift_strain: 0.0,
            gift_overworked_days: 0,
            gift_revealed: false,
            last_omen_day: 0,
            tax_unpaid_seasons: 0,
            last_tax_day: 0,
            tick_count: 0,
            flash_frames: 0,
            perf_slow_frames: 0,
            perf_last_render_us: 0,
            in_collapse: false,
            seed,
            charts,
            player_rng: Some(player_rng),
        }
    }
}
