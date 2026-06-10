// Second-pass audit regressions for the load path and economy:
// - loading re-anchors App.seed to the saved world (it didn't, so a loaded
//   game rolled encounters/collapses from the session's startup seed)
// - apply_save_data restores explored + encounter_log (the save browser had
//   its own drifting field list; encounter_log was never persisted at all)
// - the market spread can't invert: selling never pays >= buying
// - quest regen within one day produces varying batches (day-salt regenerated
//   the identical batch, enabling same-day repeat rewards)
// - journal cap drops oldest entries without unbounded growth
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::{Encounter, EncounterAction, EncounterKind, ItemType, Terrain};
use deep_world_tui::sim::journal::{Journal, Voice};
use deep_world_tui::ui::app::App;

fn played_app(seed: u64) -> App {
    let charts = load_charts().expect("charts");
    let mut a = App::new(seed, charts);
    a.generate_player();
    a.accept_player();
    a.running = true;
    a.enter_map(0);
    a.move_player(1, 0);
    a.encounter = Some(Encounter {
        kind: EncounterKind::Traveler,
        terrain: Terrain::Grass,
    });
    a.resolve_encounter(EncounterAction::Talk);
    a
}

#[test]
fn load_restores_seed_explored_and_encounter_log() {
    let mut original = played_app(4242);
    let log_len = original.encounter_log.len();
    assert!(log_len > 0, "setup should have logged an encounter");
    original.save_to_slot(1);

    // A different session: new App with a different startup seed.
    let charts = load_charts().expect("charts");
    let mut other = App::new(999_999, charts);
    other.load_game();

    assert_eq!(
        other.seed(),
        4242,
        "loading must re-anchor the RNG seed to the saved world"
    );
    assert_eq!(
        other.encounter_log.len(),
        log_len,
        "encounter history must survive save/load"
    );
    let pos = other.player_pos.expect("pos restored");
    assert!(
        other.explored[pos.region_idx].is_explored(pos.px, pos.py),
        "explored map must survive save/load"
    );
}

#[test]
fn market_spread_never_inverts() {
    let mut a = played_app(42);
    // Max out reputation — the historical exploit window.
    let pid = a.player_start.as_ref().unwrap().person.id.clone();
    let sid = a.sim.as_ref().unwrap().world.regions[0].settlements[0]
        .id
        .clone();
    a.sim
        .as_mut()
        .unwrap()
        .reputation
        .adjust_settlement(&pid, &sid, 1.0);
    for item in [
        ItemType::Herb,
        ItemType::Food,
        ItemType::Wood,
        ItemType::Iron,
    ] {
        let buy = a.quote_buy_price(item);
        let sell = a.quote_sell_price(item);
        assert!(
            sell < buy || (sell == 1 && buy == 1),
            "{}: sell {sell} must stay below buy {buy}",
            item.name()
        );
    }
}

#[test]
fn quest_regen_varies_within_a_day() {
    use deep_world_tui::model::quest::{Quest, QuestKind, QuestReward};
    let mut a = played_app(7);
    let mut batches: Vec<String> = Vec::new();
    for _ in 0..5 {
        // Regen fires on completion: hand the player an instantly-completable
        // delivery, complete it, and snapshot the regenerated board.
        let day = a.clock.day;
        {
            let sim = a.sim.as_mut().unwrap();
            sim.quests.clear();
            sim.quests.push(Quest {
                kind: QuestKind::FetchItem {
                    item: ItemType::Herb,
                    count: 1,
                },
                description: "deliver".into(),
                reward: QuestReward::Reputation { amount: 0.01 },
                progress: 0,
                target: 1,
                deadline_day: day + 10,
                assigned_day: day,
            });
        }
        a.player_start
            .as_mut()
            .unwrap()
            .inventory
            .add(ItemType::Herb, 1);
        a.advance_clock(1); // completes the delivery; regen fires at this tick
        let snapshot = a
            .sim
            .as_ref()
            .unwrap()
            .quests
            .iter()
            .map(|q| format!("{:?}", q.kind))
            .collect::<Vec<_>>()
            .join("|");
        batches.push(snapshot);
    }
    let distinct: std::collections::HashSet<_> = batches.iter().collect();
    assert!(
        distinct.len() > 1,
        "same-day quest regens must vary (got identical batches: {batches:?})"
    );
}

#[test]
fn journal_cap_keeps_newest() {
    let mut j = Journal::default();
    for i in 0..300u64 {
        j.log(i, Voice::Travel, format!("entry {i}"));
    }
    assert!(j.iter().count() <= 200, "cap respected");
    let last = j.iter_rev().next().unwrap();
    assert_eq!(last.tick, 299, "newest entry kept");
}
