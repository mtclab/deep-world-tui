// Regression: a completed FetchItem quest is a delivery — the goods must be
// consumed on completion. Previously the reward was granted but the items
// stayed in inventory, so one stack could satisfy repeated fetch quests
// (reward farming).
use deep_world_tui::charts::load::load_charts;
use deep_world_tui::model::quest::{Quest, QuestKind, QuestReward};
use deep_world_tui::model::ItemType;
use deep_world_tui::ui::app::App;

#[test]
fn completing_fetch_quest_consumes_the_delivered_items() {
    let charts = load_charts().expect("charts");
    let mut app = App::new(42, charts);
    app.generate_player();
    app.accept_player();
    app.running = true;
    app.enter_map(0);

    // Inject a known FetchItem quest and give the player exactly enough +1.
    let day = app.clock.day;
    let quest = Quest {
        kind: QuestKind::FetchItem {
            item: ItemType::Herb,
            count: 2,
        },
        description: "deliver herbs".into(),
        reward: QuestReward::Reputation { amount: 0.1 },
        progress: 0,
        target: 2,
        deadline_day: day + 30,
        assigned_day: day,
    };
    {
        let sim = app.sim.as_mut().expect("sim");
        sim.quests.clear();
        sim.quests.push(quest);
    }
    {
        let ps = app.player_start.as_mut().expect("player");
        ps.inventory.add(ItemType::Herb, 3);
    }

    // advance_clock runs check_quests_on_tick.
    app.advance_clock(1);

    let herbs_left = app
        .player_start
        .as_ref()
        .unwrap()
        .inventory
        .get(ItemType::Herb);
    assert_eq!(herbs_left, 1, "2 of 3 herbs should be delivered, 1 left");

    // The fetch quest is gone (completed + removed). It may be replaced by
    // regen, but no completed FetchItem(Herb) should remain unconsumed.
    let lingering_fetch = app.sim.as_ref().unwrap().quests.iter().any(|q| {
        matches!(q.kind, QuestKind::FetchItem { item, .. } if item == ItemType::Herb)
            && q.is_complete()
    });
    assert!(!lingering_fetch, "no completed fetch quest should linger");
}
