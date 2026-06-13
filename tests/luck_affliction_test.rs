// Luck reaches the body (#403). Fortune is not only a death-roll — it leans
// every consequence the world decides. The unlucky sicken sooner in the same
// swamp; a wound may fester or a bite may envenom, decided by luck at the
// moment it lands, not by how fast you tend it. Treatment is the counter; the
// star only decides whether the turn comes.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::wildlife::WildSpecies;
use deep_world_tui::model::{
    Disease, Encounter, EncounterAction, EncounterKind, Fortune, Need, Needs, PlayerPos, Terrain,
};
use deep_world_tui::sim::illness::tick_illness_luck;
use deep_world_tui::ui::app::App;

fn worn_needs() -> Needs {
    let mut n = Needs::default();
    n.values.insert(Need::Food, 0.5); // a little hungry — the land bites harder
    n.values.insert(Need::Safety, 0.2); // no shelter
    n
}

#[test]
fn the_unlucky_sicken_sooner_in_the_same_swamp() {
    // Same ground, same days — only the star differs. The cursed take ill more.
    fn illnesses(luck_mult: f64) -> u32 {
        let needs = worn_needs();
        let mut count = 0;
        for tick in 0..6000u64 {
            if tick_illness_luck(7, tick, Terrain::Swamp, &needs, false, 0, luck_mult).is_some() {
                count += 1;
            }
        }
        count
    }
    let blessed = illnesses(Fortune::from_value(1.0).bad_multiplier()); // < 1
    let cursed = illnesses(Fortune::from_value(-1.0).bad_multiplier()); // > 1
    assert!(
        cursed > blessed,
        "the cursed sicken sooner ({cursed} vs {blessed})"
    );
    assert!(blessed > 0, "but the blessed are not immune ({blessed})");
}

fn app(seed: u64, fortune: f64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.player_pos = Some(PlayerPos {
        region_idx: 0,
        px: 5,
        py: 5,
    });
    if let Some(ps) = a.player_start.as_mut() {
        ps.companions.clear();
    }
    a.fortune = Fortune::from_value(fortune);
    a
}

fn flee_once(a: &mut App, tick: u64, species: WildSpecies, terrain: Terrain, fortune: f64) {
    // Re-pin the life each turn: a fatal run-down swaps in an heir with a fresh
    // star, which would otherwise drift the measurement.
    a.fortune = Fortune::from_value(fortune);
    a.sim.as_mut().unwrap().world.tick = tick;
    a.vitals.energy = 1.0;
    a.vitals.hunger = 1.0;
    a.collapse = None;
    a.death_cause = None;
    if let Some(ps) = a.player_start.as_mut() {
        ps.person.illnesses.clear(); // each flee independent for counting
    }
    a.encounter = Some(Encounter {
        kind: EncounterKind::Wildlife,
        terrain,
        species: Some(species),
    });
    a.resolve_encounter(EncounterAction::Flee);
}

fn has(a: &App, d: Disease) -> bool {
    a.player_start
        .as_ref()
        .is_some_and(|ps| ps.person.illnesses.iter().any(|x| x.disease == d))
}

#[test]
fn a_predators_wound_can_fester() {
    // Flee wolves in dirty country; some of the gashes turn to infection.
    let mut a = app(7, 0.0);
    let mut infections = 0;
    for tick in 0..4000u64 {
        flee_once(&mut a, tick, WildSpecies::Wolf, Terrain::Swamp, 0.0);
        if has(&a, Disease::Infection) {
            infections += 1;
        }
    }
    assert!(
        infections > 0,
        "a torn-open wound in the mire sometimes festers ({infections})"
    );
}

#[test]
fn a_venomous_bite_envenoms() {
    // The adder stands its ground; flee it and the strike that lands is venom,
    // not a scratch. Most adder-wounds carry it.
    let mut a = app(7, 0.0);
    let mut venoms = 0;
    let mut wounds = 0;
    for tick in 0..4000u64 {
        flee_once(&mut a, tick, WildSpecies::Adder, Terrain::Grass, 0.0);
        // a wound shows as lost energy (clean adder flee costs only the base)
        if a.vitals.energy < 0.8 {
            wounds += 1;
            if has(&a, Disease::Venom) {
                venoms += 1;
            }
        }
        // never an infection from a clean venomous strike rather than dirt
        assert!(
            !has(&a, Disease::Infection),
            "an adder gives venom, not rot"
        );
    }
    assert!(wounds > 0, "the adder strikes sometimes ({wounds})");
    assert!(
        venoms * 2 > wounds,
        "most strikes that land envenom ({venoms} of {wounds})"
    );
}

#[test]
fn the_cursed_wound_festers_more_than_the_blessed() {
    fn festers(fortune: f64) -> u32 {
        let mut a = app(7, fortune);
        let mut n = 0;
        for tick in 0..4000u64 {
            flee_once(&mut a, tick, WildSpecies::Wolf, Terrain::Swamp, fortune);
            if has(&a, Disease::Infection) {
                n += 1;
            }
        }
        n
    }
    let cursed = festers(-1.0);
    let blessed = festers(1.0);
    assert!(
        cursed > blessed,
        "the cursed wound turns more often ({cursed} vs {blessed})"
    );
}

#[test]
fn venom_is_never_taken_from_the_land() {
    // It only ever comes from a bite — terrain alone cannot give it.
    for t in [
        Terrain::Swamp,
        Terrain::Grass,
        Terrain::Forest,
        Terrain::Sand,
        Terrain::DeepDesert,
        Terrain::Cave,
    ] {
        assert_eq!(Disease::Venom.contraction_probability(t), 0.0, "{t:?}");
    }
}
