// Luck reaches the body (#403). Fortune is not only a death-roll — it leans
// every consequence the world decides. The unlucky sicken sooner in the same
// swamp. (The flee-wound festering tests went with the encounter screen's
// retirement (#649); wound-festering on the grid is a follow-up.)
use deep_world_tui::model::{Disease, Fortune, Need, Needs, Terrain};
use deep_world_tui::sim::illness::tick_illness_luck;

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
