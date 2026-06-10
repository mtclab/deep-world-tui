// Regression for the living-world content pass:
// - Archive/TradePost/Shrine actually generate (were implemented, unreachable)
// - NPC wants use people identity (string mismatch made every NPC generic)
// - formerly flavor-only personality traits now have effects
// - childbirth complication is gated to those who can give birth
// - taverns log a Rumor-voice journal entry (voice existed, never logged)
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::gen::world::generate_world;
use deep_world_tui::model::{InterPeopleBias, SettlementService};
use deep_world_tui::sim::illness::can_contract_childbirth;
use deep_world_tui::sim::wants::{generate_npc_wants, WantKind};

#[test]
fn signature_services_generate_for_their_peoples() {
    let charts = load_charts().expect("charts");
    let mut found_archive = false;
    let mut found_tradepost = false;
    let mut found_shrine = false;
    for seed in 0..40u64 {
        let world = generate_world(seed, &charts);
        for region in &world.regions {
            for s in &region.settlements {
                found_archive |= s.services.contains(&SettlementService::Archive);
                found_tradepost |= s.services.contains(&SettlementService::TradePost);
                found_shrine |= s.services.contains(&SettlementService::Shrine);
            }
        }
        if found_archive && found_tradepost && found_shrine {
            break;
        }
    }
    assert!(
        found_archive,
        "Arkit settlements should generate an Archive"
    );
    assert!(
        found_tradepost,
        "Väylä settlements should generate a TradePost"
    );
    assert!(found_shrine, "Laakso settlements should generate a Shrine");
}

#[test]
fn npc_wants_reflect_people_identity() {
    // Chart people strings are lowercase; the old match on display names
    // ("Sepät") never hit, so smith-folk never wanted to repair a roof.
    let mut sepat_specific = 0;
    for i in 0..200u64 {
        for w in generate_npc_wants(i, &format!("p{i}"), "sepat") {
            if matches!(w.kind, WantKind::RepairRoof | WantKind::FindApprentice) {
                sepat_specific += 1;
            }
        }
    }
    assert!(
        sepat_specific > 0,
        "sepat NPCs should sometimes want roof-repair/apprentice (people weights applied)"
    );
}

#[test]
fn formerly_flavor_traits_now_have_effects() {
    let t = |s: &str| vec![s.to_string()];
    assert!(InterPeopleBias::personality_mod(&t("cheerful")) > 0.0);
    assert!(InterPeopleBias::personality_mod(&t("melancholic")) < 0.0);
    assert!(InterPeopleBias::trade_price_modifier(&t("devious")) > 0.0);
    assert!(InterPeopleBias::encounter_modifier(&t("proud")).intimidate > 0.0);
    assert!(InterPeopleBias::encounter_modifier(&t("boisterous")).talk > 0.0);
    assert!(InterPeopleBias::encounter_modifier(&t("earnest")).talk > 0.0);
    assert!(InterPeopleBias::encounter_modifier(&t("world-weary")).calm > 0.0);
}

#[test]
fn childbirth_complication_is_gated() {
    assert!(can_contract_childbirth("f", "adult"));
    assert!(can_contract_childbirth("f", "youth"));
    assert!(!can_contract_childbirth("m", "adult"));
    assert!(!can_contract_childbirth("f", "elder"));
    assert!(!can_contract_childbirth("f", "child"));
}

#[test]
fn tavern_visit_logs_a_rumor() {
    use deep_world_tui::ui::app::App;
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);
    app.clock.hour = 10;
    app.enter_settlement(0, 0);
    app.use_service(SettlementService::Tavern);
    let has_rumor = app
        .sim
        .as_ref()
        .unwrap()
        .journal
        .iter()
        .any(|e| e.voice == deep_world_tui::sim::journal::Voice::Rumor);
    assert!(
        has_rumor,
        "a tavern visit should log a Rumor-voice journal entry; got: {:?}",
        app.status_msg
    );
}
