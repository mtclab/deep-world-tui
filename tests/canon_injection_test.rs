// Canon injection: tavern talk and region descriptions now reach past the
// horizon — places and affairs from the canon gazetteer and the post-Fall
// chronicle (Velkarath, the Pilgrimage Road, the Wells-Compact, Kaelva...).
use deep_world_tui::rng::SeedRng;
use deep_world_tui::sim::journal::rumor_text;

#[test]
fn tavern_talk_names_the_wider_world() {
    let canon_markers = [
        "Velkarath",
        "Karsath",
        "Wells-Compact",
        "Pilgrimage Road",
        "Tähti",
        "Sampsara",
        "Oltzafell",
        "Kaelva",
        "Fort Verath",
        "Sampa",
    ];
    let mut hits = 0;
    for seed in 0..200u64 {
        let mut rng = SeedRng::new(seed);
        let r = rumor_text(&mut rng);
        if canon_markers.iter().any(|m| r.contains(m)) {
            hits += 1;
        }
    }
    assert!(
        hits > 30,
        "roughly half of tavern rumors should name the canon world (got {hits}/200)"
    );
}
