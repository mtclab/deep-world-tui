// The naming baseline (canon: NAME_RESOLUTION Diverse-Branch Opacity +
// owner's rules): (1) peoples never carry god-names in their names — places
// may; (2) names are invented opaque Finno-Ugric-calibrated roots, not
// transparent Finnish, and never English.
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::PeopleKind;

const GODS: [&str; 5] = ["keuru", "oltzed", "sampsa", "masa", "kukri"];

fn all_peoples() -> Vec<PeopleKind> {
    use PeopleKind as P;
    vec![
        P::Metsik,
        P::Arkit,
        P::Vayla,
        P::Sepat,
        P::Ahjo,
        P::Laakso,
        P::Varhaiset,
        P::Metsareunat,
        P::Porokansa,
        P::Koskimetsa,
        P::Muistikansa,
        P::Taulukansa,
        P::Kirjakansa,
        P::Takovaki,
        P::Rantavaki,
        P::Saarivaki,
        P::Hiekkakavelijat,
        P::Haramaki,
        P::Jamavaki,
        P::Pohjavaki,
        P::Tzakhar,
        P::Merak,
        P::Shear,
        P::Hal,
        P::Khor,
    ]
}

#[test]
fn no_people_name_carries_a_god_name() {
    for p in all_peoples() {
        for name in [p.label(), p.true_endonym(), p.arkit_name()] {
            let lower = name.to_lowercase();
            for god in GODS {
                assert!(
                    !lower.contains(god),
                    "{:?}: people-name {name:?} carries god-name {god:?} (places may, peoples may not)",
                    p
                );
            }
        }
        // The pilgrimage exonyms ARE god-derived by definition — they exist as
        // a historical register and must never be the display label.
        assert_ne!(
            p.label(),
            p.pilgrimage_exonym(),
            "{:?}: god-derived exonym used as the display name",
            p
        );
    }
}

#[test]
fn stayed_peoples_display_their_own_names() {
    // The fourteen stayed peoples are called by their true endonyms, not the
    // Arkit scholarly compounds (transparent Finnish).
    assert_eq!(PeopleKind::Porokansa.label(), "Tuorva");
    assert_eq!(PeopleKind::Varhaiset.label(), "Körvä");
    assert_eq!(PeopleKind::Pohjavaki.label(), "Väškam");
    // The six SAST names are locked by the published novels.
    assert_eq!(PeopleKind::Metsik.label(), "Metsik");
    assert_eq!(PeopleKind::Vayla.label(), "Väylä");
}

#[test]
fn generated_names_are_opaque_and_unenglish() {
    let charts = load_charts().expect("charts");
    let banned_english = [
        "ford", "bridge", "haven", "port", "bay", "shore", "anchor", "wood", "grove", "stone",
        "field", "plain", "marsh", "reed", "mill", "dock", "peak", "ridge",
    ];
    let banned_finnish = [
        "lumi", "talvi", "kettu", "poro", "koski", "ranta", "saari", "metsä", "järvi", "niemi",
    ];
    let world = deep_world_tui::gen::world::generate_world(31337, &charts);
    for region in &world.regions {
        let names: Vec<String> = std::iter::once(region.name.clone())
            .chain(region.settlements.iter().map(|s| s.name.clone()))
            .chain(
                region
                    .settlements
                    .iter()
                    .flat_map(|s| s.people.iter().map(|p| p.name.clone())),
            )
            .collect();
        for n in names {
            let lower = n.to_lowercase();
            for b in banned_english {
                assert!(!lower.contains(b), "{n:?} contains English {b:?}");
            }
            for b in banned_finnish {
                assert!(
                    !lower.contains(b),
                    "{n:?} contains transparent Finnish {b:?}"
                );
            }
            for god in GODS {
                assert!(!lower.contains(god), "{n:?} carries god-name {god:?}");
            }
            assert!(
                !n.chars().skip(1).any(|c| c.is_uppercase()),
                "{n:?} has inner CamelCase mash"
            );
        }
    }
}
