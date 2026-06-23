// Lore reconciliation (see LORE.md): the game's assertions now match canon
// (deep-world-history races overview) and the published novels.
use deep_world_tui::model::weather::{FestivalKind, GameClock, Season};
use deep_world_tui::model::{GodName, PeopleKind};

#[test]
fn non_human_patron_gods_match_canon_first_encounters() {
    // races/00_sapient_races_overview.md, "Relationship with the Five Gods".
    assert_eq!(PeopleKind::Tzakhar.patron_god(), Some(GodName::Kukri));
    assert_eq!(PeopleKind::Merak.patron_god(), Some(GodName::Masa));
    assert_eq!(
        PeopleKind::Shear.patron_god(),
        Some(GodName::Kukri),
        "She'ar hear Kukri in the steppe's silence"
    );
    assert_eq!(PeopleKind::Hal.patron_god(), Some(GodName::Keuru));
    assert_eq!(
        PeopleKind::Khor.patron_god(),
        Some(GodName::Kukri),
        "Khör's primary divine relationship is Talven-Hiljaisuus, Winter's-Silence"
    );
}

#[test]
fn festivals_honor_their_peoples_own_gods() {
    // A people's festival must be patroned by their own god (Varhaiset, who
    // honor all Five, keep the Midsummer Bonfire).
    for p in [
        PeopleKind::Metsik,
        PeopleKind::Arkit,
        PeopleKind::Vayla,
        PeopleKind::Sepat,
        PeopleKind::Ahjo,
        PeopleKind::Laakso,
        PeopleKind::Tzakhar,
        PeopleKind::Merak,
        PeopleKind::Shear,
        PeopleKind::Hal,
        PeopleKind::Khor,
        PeopleKind::Muistikansa,
        PeopleKind::Haramaki,
        PeopleKind::Porokansa,
    ] {
        if let Some(god) = p.patron_god() {
            let festival = FestivalKind::for_people(p);
            assert_eq!(
                festival.patron_god(),
                god,
                "{:?}'s festival {:?} should honor {:?}",
                p,
                festival,
                god
            );
        }
    }
    assert_eq!(
        FestivalKind::for_people(PeopleKind::Arkit),
        FestivalKind::StarReckoning,
        "the Arkit keep Sampsa's Star-Reckoning, not a Kukri vigil"
    );
}

#[test]
fn the_present_is_anchored_after_the_trilogy() {
    assert_eq!(Season::PRESENT_YEAR_AF, 183, "one year after 182 AF");
    let clock = GameClock::default();
    assert_eq!(clock.year_af(), 183);
    let later = GameClock {
        day: Season::YEAR_DAYS * 2 + 5,
        ..Default::default()
    };
    assert_eq!(later.year_af(), 185, "game years advance the AF count");
}

#[test]
fn shear_and_sharra() {
    // 'Sharra' is a wiki-layer error; the game accepts only the canonical
    // chain (she'ar + the Metsik exonym muraskala).
    assert_eq!(PeopleKind::from_name("she'ar"), PeopleKind::Shear);
    assert_eq!(PeopleKind::from_name("muraskala"), PeopleKind::Shear);
    assert_eq!(PeopleKind::Shear.label(), "She'ar");
}
