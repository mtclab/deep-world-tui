// Regression: the journal serialized a bare `voice:Scar` (the field is a plain
// Voice enum) but deserialized into Option<Voice>. Compact RON has no
// implicit-some, so a played save failed to load with "Expected option".
use deep_world_tui::sim::journal::{Journal, Voice};

#[test]
fn journal_roundtrips_through_compact_ron() {
    let mut j = Journal::default();
    j.log(1, Voice::Scar, "My years weigh on me now.".into());
    j.log(2, Voice::Travel, "I walked into the day.".into());
    j.log(3, Voice::Encounter, "A traveler on the road.".into());

    let s = ron::ser::to_string(&j).expect("serialize");
    let back: Journal = ron::from_str(&s).expect("deserialize compact RON");

    let got: Vec<_> = back
        .iter()
        .map(|e| (e.tick, e.voice, e.text.clone()))
        .collect();
    assert_eq!(got.len(), 3);
    assert_eq!(got[0].1, Voice::Scar);
    assert_eq!(got[1].1, Voice::Travel);
    assert_eq!(got[2].1, Voice::Encounter);
}

#[test]
fn journal_migrates_old_entry_without_voice() {
    // Older saves wrote entries with no `voice` field; default must fill it.
    let old = r#"[(tick:5,text:"legacy line")]"#;
    let back: Journal = ron::from_str(old).expect("deserialize legacy");
    let e = back.iter().next().expect("one entry");
    assert_eq!(e.tick, 5);
    assert_eq!(e.voice, Voice::Encounter);
}
