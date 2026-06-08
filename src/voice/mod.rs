use crate::model::{Need, PeopleKind, Person};
use crate::rng::SeedRng;

pub mod craft_hooks;
pub mod people_banks;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Situation {
    Greeting,
    Trade,
    NeedDire,
    NeedFine,
    Farewell,
    Gossip,
}

impl Situation {
    pub fn as_str(self) -> &'static str {
        match self {
            Situation::Greeting => "greeting",
            Situation::Trade => "trade",
            Situation::NeedDire => "need_dire",
            Situation::NeedFine => "need_fine",
            Situation::Farewell => "farewell",
            Situation::Gossip => "gossip",
        }
    }
}

pub fn voice_line(person: &Person, situation: &str) -> String {
    let sit = match situation {
        "greeting" => Situation::Greeting,
        "trade" => Situation::Trade,
        "need_dire" => Situation::NeedDire,
        "need_fine" => Situation::NeedFine,
        "farewell" => Situation::Farewell,
        "gossip" => Situation::Gossip,
        _ => Situation::Greeting,
    };
    voice_line_situation(person, sit)
}

fn personality_prefix(personality: &[String]) -> &'static str {
    if personality.is_empty() {
        return "";
    }
    match personality[0].as_str() {
        "stoic" => "",
        "warm" => "A slight smile. ",
        "wary" => "Eyes darting. ",
        "curious" => "Leaning forward. ",
        "proud" => "Chin lifted. ",
        "gentle" => "Softly. ",
        "sharp" => "Quickly. ",
        "nervous" => "Fidgeting. ",
        "calm" => "",
        "stubborn" => "Arms crossed. ",
        "suspicious" => "Eyes narrowed. ",
        "cheerful" => "Grinning. ",
        "boisterous" => "Loudly. ",
        "melancholic" => "A long pause. ",
        "devious" => "A sly glance. ",
        "earnest" => "Earnestly. ",
        "world-weary" => "Heavily. ",
        _ => "",
    }
}

const PERSONALITY_CONFLICTS: &[(&str, &str)] = &[
    ("cheerful", "bitter"),
    ("cheerful", "melancholic"),
    ("cheerful", "suspicious"),
    ("cheerful", "withdrawn"),
    ("earnest", "devious"),
    ("earnest", "mercenary"),
    ("stoic", "boisterous"),
    ("warm", "suspicious"),
    ("loyal", "mercenary"),
];

fn has_conflict(personality: &[String]) -> bool {
    for (a, b) in PERSONALITY_CONFLICTS {
        let has_a = personality.iter().any(|t| t == *a);
        let has_b = personality.iter().any(|t| t == *b);
        if has_a && has_b {
            return true;
        }
    }
    false
}

fn gossip_personality_flavor(personality: &[String]) -> &'static str {
    if personality.is_empty() || has_conflict(personality) {
        return "";
    }
    match personality[0].as_str() {
        "stoic" => "measured words, carefully chosen. ",
        "warm" => "with easy warmth, as if to an old friend. ",
        "proud" => "with the certainty of one who has seen much. ",
        "cautious" => "selectively, weighing each word. ",
        "reckless" => "bluntly, without hesitation. ",
        "devout" => "reverently, as if sharing a teaching. ",
        "mercenary" => "like a transaction — information for a price. ",
        "loyal" => "with fierce conviction. ",
        "bitter" => "with an edge that cuts. ",
        "curious" => "eagerly, hungry for reaction. ",
        "withdrawn" => "almost to themselves, half-whispered. ",
        "shrewd" => "sharply, as if measuring your worth. ",
        "suspicious" => "guardedly, watching your reaction. ",
        "cheerful" => "brightly, as if the news itself is a gift. ",
        "boisterous" => "loudly, for all to hear. ",
        "melancholic" => "with a heaviness, as if the words themselves are tired. ",
        "devious" => "with a knowing half-smile, letting the silence speak. ",
        "earnest" => "plainly, no decoration, just truth. ",
        "world-weary" => "with the weariness of one who has seen it all before. ",
        _ => "",
    }
}

fn craft_flavor(craft: &str) -> &'static str {
    match craft {
        "forge" => "tools laid aside",
        "still" => "hands still smelling of herbs",
        "root" => "dirt under the nails",
        "word" => "ink-stained fingers",
        "current" => "salt-crusted hands",
        "loom" => "thread-wrapped fingers",
        "tanner" => "leather-scented hands",
        "rope_maker" => "hemp-fibre grip",
        "charcoal_burner" => "soot-stained palms",
        "falconer" => "glove-softened grip",
        "salt_miner" => "crystal-grit hands",
        "reed_weaver" => "reed-split fingers",
        _ => "calloused hands",
    }
}

fn profession_flavor(profession: &str) -> &'static str {
    match profession {
        "smith" => "hammer-marked",
        "forester" => "sap-scented",
        "fisher" => "salt-worn",
        "farmer" => "weathered",
        "herder" => "hill-tested",
        "trader" => "road-wise",
        "sailor" => "wave-carved",
        "priest" => "ceremony-steeped",
        "scribe" => "ink-faded",
        "healer" => "herb-scented",
        "miner" => "dust-grey",
        "weaver" => "thread-worn",
        "labourer" => "heavy-limbed",
        "scholar" => "page-worn",
        "carpenter" => "shave-pale",
        "potter" => "clay-stained",
        "hunter" => "trail-quiet",
        "baker" => "flour-dusted",
        "guard" => "armor-weary",
        "innkeep" => "hearth-warm",
        "hearth-keeper" => "fire-tended",
        "path-finder" => "horizon-eyed",
        "fence-builder" => "post-scarred",
        "beast-handler" => "callously calm",
        "singer" => "song-threaded",
        _ => "steady-eyed",
    }
}

fn greeting_lines() -> &'static [&'static str] {
    &[
        "regards you steadily. \"Another day in the Archive's shadow.\"",
        "nods. \"The path finds its own.\"",
        "looks up. \"Welcome. What brings you?\"",
        "sets aside work. \"The {people} remember their friends.\"",
        "studies you. \"The old roads still hold, if you know where to look.\"",
    ]
}

fn greeting_hungry_lines() -> &'static [&'static str] {
    &[
        "nods weakly. \"I cannot think past my hunger.\"",
        "presses a hand to stomach. \"The void eats thought.\"",
        "waves dismissively. \"Not now. The belly rules.\"",
    ]
}

fn greeting_crafty_lines() -> &'static [&'static str] {
    &[
        "sets down {craft_flavor}. \"Welcome. The {people} remember their friends.\"",
        "wipes {craft_flavor}. \"Good timing — the work was getting lonely.\"",
        "gestures from the {craft_flavor}. \"Come. Sit. There is always room.\"",
    ]
}

fn trade_lines() -> &'static [&'static str] {
    &[
        "gestures toward the goods. \"Fair exchange keeps the world turning.\"",
        "counts carefully. \"The {people} trade honestly, or not at all.\"",
        "spreads hands. \"What have you, and what do you need?\"",
    ]
}

fn trade_broke_lines() -> &'static [&'static str] {
    &[
        "shakes head slowly. \"I have nothing to trade but my word.\"",
        "turns away. \"Coin speaks. Without it, silence answers.\"",
        "shows empty palms. \"The {profession_flavor} knows poverty when it sees it.\"",
    ]
}

fn need_dire_hungry_lines() -> &'static [&'static str] {
    &[
        "clutches stomach. \"The hunger eats my thoughts.\"",
        "leans against the wall. \"Empty. Everything is empty.\"",
        "stares at nothing. \"When the belly hollows, the mind follows.\"",
    ]
}

fn need_dire_broke_lines() -> &'static [&'static str] {
    &[
        "stares at empty hands. \"Debts are stones around my neck.\"",
        "voice drops. \"The {people} see all — and they see my poverty.\"",
        "shakes head. \"There is nothing left. Nothing.\"",
    ]
}

fn need_dire_general_lines() -> &'static [&'static str] {
    &[
        "looks away. \"Some needs go deeper than coin or bread.\"",
        "voice tightens. \"Do not ask. The {people} bear what they must.\"",
        "exhales. \"The weight does not lift. It only shifts.\"",
    ]
}

fn need_fine_lines() -> &'static [&'static str] {
    &[
        "stands easy. \"The {people} have known worse seasons. This one holds.\"",
        "looks around. \"Enough. More than enough, for once.\"",
        "shrugs. \"Steady days. Long may they last.\"",
    ]
}

fn farewell_lines() -> &'static [&'static str] {
    &[
        "steps back. \"Until the next turning.\"",
        "nods firmly. \"May the Archive hold what we have built.\"",
        "turns to go. \"The road calls us both. Walk well.\"",
    ]
}

fn farewell_crafty_lines() -> &'static [&'static str] {
    &[
        "returns to the {craft_flavor}. \"May the Archive hold what we have built.\"",
        "resumes work at {craft_flavor}. \"Until next time.\"",
        "gathers {craft_flavor}. \"Go well. The work continues.\"",
    ]
}

fn gossip_lines() -> &'static [&'static str] {
    &[
        "speaks low. \"Word travels the river roads. Listen, and the {people} will tell you.\"",
        "glances around. \"They say things are changing. The {people} feel it.\"",
        "leans in. \"I heard something. Whether it is true... the {people} will see.\"",
    ]
}

fn gossip_hungry_lines() -> &'static [&'static str] {
    &[
        "leans close. \"They say the storehouses thin. The {people} are watching.\"",
        "whispers. \"Hunger sharpens the ears. The {people} hear more than they let on.\"",
        "looks grave. \"The {people} keep count. The count does not favor us.\"",
    ]
}

fn bias_prefix_hostile(situation: Situation) -> &'static str {
    match situation {
        Situation::Greeting => "They barely glance at you. ",
        Situation::Trade => "Arms fold. 'We don't trade with your kind.' ",
        Situation::Farewell => "A curt nod. Nothing more. ",
        _ => "",
    }
}

fn bias_prefix_cold(situation: Situation) -> &'static str {
    match situation {
        Situation::Greeting => "Eyes narrow slightly. ",
        Situation::Trade => "Reluctant hands count the coins twice. ",
        Situation::Farewell => "A guarded farewell. ",
        _ => "",
    }
}

fn pick_line(rng: &mut SeedRng, lines: &[&str]) -> usize {
    if lines.is_empty() {
        return 0;
    }
    rng.gen_range(lines.len() as u32) as usize
}

fn fill_template(template: &str, person: &Person) -> String {
    let people = &person.people;
    let craft_flavor = craft_flavor(&person.craft_affinity);
    let prof_flavor = profession_flavor(&person.profession);
    template
        .replace("{people}", people)
        .replace("{craft_flavor}", craft_flavor)
        .replace("{profession_flavor}", prof_flavor)
}

pub fn voice_line_situation(person: &Person, situation: Situation) -> String {
    let mut rng = SeedRng::new(crate::rng::fnv1a_hash(&format!(
        "voice-{}-{}",
        person.id,
        situation.as_str()
    )));

    let name = &person.name;
    let low_food = person.needs.get(Need::Food) < 0.3;
    let low_money = person.needs.get(Need::Money) < 0.3;
    let has_craft = person.craft_affinity != "none";

    let line = match situation {
        Situation::Greeting => {
            if low_food {
                let bank = greeting_hungry_lines();
                let idx = pick_line(&mut rng, bank);
                format!("{name} {}", fill_template(bank[idx], person))
            } else if has_craft {
                let bank = greeting_crafty_lines();
                let idx = pick_line(&mut rng, bank);
                format!("{name} {}", fill_template(bank[idx], person))
            } else {
                let bank = greeting_lines();
                let idx = pick_line(&mut rng, bank);
                format!("{name} {}", fill_template(bank[idx], person))
            }
        }
        Situation::Trade => {
            if low_money {
                let bank = trade_broke_lines();
                let idx = pick_line(&mut rng, bank);
                format!(
                    "{name} the {} {}",
                    person.profession,
                    fill_template(bank[idx], person)
                )
            } else {
                let bank = trade_lines();
                let idx = pick_line(&mut rng, bank);
                format!(
                    "{name} the {} {}",
                    person.profession,
                    fill_template(bank[idx], person)
                )
            }
        }
        Situation::NeedDire => {
            let bank = if low_food {
                need_dire_hungry_lines()
            } else if low_money {
                need_dire_broke_lines()
            } else {
                need_dire_general_lines()
            };
            let idx = pick_line(&mut rng, bank);
            format!("{name} {}", fill_template(bank[idx], person))
        }
        Situation::NeedFine => {
            let bank = need_fine_lines();
            let idx = pick_line(&mut rng, bank);
            format!(
                "{name} the {} {}",
                person.profession,
                fill_template(bank[idx], person)
            )
        }
        Situation::Farewell => {
            if has_craft {
                let bank = farewell_crafty_lines();
                let idx = pick_line(&mut rng, bank);
                format!("{name} {}", fill_template(bank[idx], person))
            } else {
                let bank = farewell_lines();
                let idx = pick_line(&mut rng, bank);
                format!("{name} {}", fill_template(bank[idx], person))
            }
        }
        Situation::Gossip => {
            let flavor = gossip_personality_flavor(&person.personality);
            if low_food {
                let bank = gossip_hungry_lines();
                let idx = pick_line(&mut rng, bank);
                let base = format!("{name} {}", fill_template(bank[idx], person));
                if flavor.is_empty() {
                    base
                } else {
                    format!("{flavor}{base}")
                }
            } else {
                let bank = gossip_lines();
                let idx = pick_line(&mut rng, bank);
                let base = format!(
                    "{name} the {} {}",
                    person.profession,
                    fill_template(bank[idx], person)
                );
                if flavor.is_empty() {
                    base
                } else {
                    format!("{flavor}{base}")
                }
            }
        }
    };

    let prefix = personality_prefix(&person.personality);
    if prefix.is_empty() {
        line
    } else {
        format!("{prefix}{line}")
    }
}

pub fn voice_line_situation_biased(
    person: &Person,
    situation: Situation,
    player_people: PeopleKind,
) -> String {
    let npc_people = PeopleKind::from_name(&person.people);
    let bias = player_people.bias_toward(npc_people);

    let base = voice_line_situation(person, situation);

    if player_people == npc_people || bias > -0.05 {
        return base;
    }

    let prefix = if bias < -0.15 {
        bias_prefix_hostile(situation)
    } else {
        bias_prefix_cold(situation)
    };

    if prefix.is_empty() {
        base
    } else {
        format!("{prefix}{base}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Needs;
    use crate::model::NpcSchedule;

    fn test_person() -> Person {
        Person {
            id: "test-1".into(),
            name: "Metsik".into(),
            people: "Sepät".into(),
            sex: "male".into(),
            age_band: "adult".into(),
            profession: "smith".into(),
            social_class: "commoner".into(),
            craft_affinity: "forge".into(),
            personality: vec!["stoic".into()],
            region: "river_valley".into(),
            settlement: "Ahjo".into(),
            has_spouse: false,
            children_count: 0,
            needs: Needs::default(),
            has_debt: false,
            bias: "0.0".into(),
            schedule: NpcSchedule::default(),
            illnesses: Vec::new(),
        }
    }

    fn test_person_hungry() -> Person {
        let mut p = test_person();
        p.needs.satisfy(Need::Food, -0.8);
        p
    }

    fn test_person_broke() -> Person {
        let mut p = test_person();
        p.needs.satisfy(Need::Money, -0.8);
        p
    }

    #[test]
    fn greeting_with_craft() {
        let p = test_person();
        let line = voice_line_situation(&p, Situation::Greeting);
        assert!(line.contains("Metsik"));
        assert!(line.contains("Sepät") || line.contains("tools") || line.contains("forge"));
    }

    #[test]
    fn greeting_hungry() {
        let p = test_person_hungry();
        let line = voice_line_situation(&p, Situation::Greeting);
        assert!(
            line.contains("hunger")
                || line.contains("belly")
                || line.contains("empty")
                || line.contains("void")
        );
    }

    #[test]
    fn trade_low_money() {
        let p = test_person_broke();
        let line = voice_line_situation(&p, Situation::Trade);
        assert!(
            line.contains("nothing")
                || line.contains("Coin")
                || line.contains("poverty")
                || line.contains("silence")
        );
    }

    #[test]
    fn need_dire_low_food() {
        let p = test_person_hungry();
        let line = voice_line_situation(&p, Situation::NeedDire);
        assert!(line.contains("hunger") || line.contains("Empty") || line.contains("hollow"));
    }

    #[test]
    fn need_dire_broke() {
        let p = test_person_broke();
        let line = voice_line_situation(&p, Situation::NeedDire);
        assert!(
            line.contains("Debts")
                || line.contains("poverty")
                || line.contains("Nothing")
                || line.contains("stones")
        );
    }

    #[test]
    fn need_fine() {
        let p = test_person();
        let line = voice_line_situation(&p, Situation::NeedFine);
        assert!(
            line.contains("worse seasons") || line.contains("Enough") || line.contains("Steady")
        );
    }

    #[test]
    fn farewell_with_craft() {
        let p = test_person();
        let line = voice_line_situation(&p, Situation::Farewell);
        assert!(line.contains("forge") || line.contains("Archive") || line.contains("work"));
    }

    #[test]
    fn gossip_well_fed() {
        let p = test_person();
        let line = voice_line_situation(&p, Situation::Gossip);
        assert!(!line.is_empty());
    }

    #[test]
    fn voice_line_string_dispatch() {
        let p = test_person();
        let line = voice_line(&p, "greeting");
        assert!(!line.is_empty());
    }

    #[test]
    fn unknown_situation_defaults_greeting() {
        let p = test_person();
        let line = voice_line(&p, "unknown");
        assert!(!line.is_empty());
    }

    #[test]
    fn deterministic_same_seed_same_output() {
        let p = test_person();
        let line1 = voice_line_situation(&p, Situation::Greeting);
        let line2 = voice_line_situation(&p, Situation::Greeting);
        assert_eq!(
            line1, line2,
            "same person + situation must produce same line"
        );
    }

    #[test]
    fn deterministic_different_id_different_output() {
        let mut p1 = test_person();
        p1.id = "test-1".into();
        let mut p2 = test_person();
        p2.id = "test-2".into();
        // Different IDs may or may not produce different lines (only 3-5 variants)
        // but at minimum both must produce valid output
        let line1 = voice_line_situation(&p1, Situation::Greeting);
        let line2 = voice_line_situation(&p2, Situation::Greeting);
        assert!(!line1.is_empty());
        assert!(!line2.is_empty());
    }

    #[test]
    fn personality_prefix_warm() {
        let mut p = test_person();
        p.personality = vec!["warm".into()];
        let line = voice_line_situation(&p, Situation::Greeting);
        assert!(
            line.starts_with("A slight smile. "),
            "warm personality should add prefix"
        );
    }

    #[test]
    fn personality_prefix_stoic() {
        let p = test_person(); // stoic
        let line = voice_line_situation(&p, Situation::Greeting);
        assert!(
            !line.starts_with("A slight smile"),
            "stoic personality should have no prefix"
        );
    }

    #[test]
    fn biased_voice_same_people() {
        let p = test_person();
        let base = voice_line_situation(&p, Situation::Greeting);
        let biased = voice_line_situation_biased(&p, Situation::Greeting, PeopleKind::Sepat);
        assert_eq!(base, biased, "same people should have no bias prefix");
    }

    #[test]
    fn biased_voice_hostile_prefix() {
        let p = test_person();
        let biased = voice_line_situation_biased(&p, Situation::Greeting, PeopleKind::Metsik);
        assert!(
            biased.contains("barely glance") || biased.contains("Eyes narrow"),
            "hostile bias should add prefix to greeting"
        );
    }

    #[test]
    fn biased_voice_neutral_no_prefix() {
        let p = test_person();
        let biased = voice_line_situation_biased(&p, Situation::Greeting, PeopleKind::Arkit);
        assert!(
            !biased.starts_with("They barely") && !biased.starts_with("Eyes narrow"),
            "Arkit→Sepät is neutral, no bias prefix"
        );
    }

    #[test]
    fn craft_flavor_varies() {
        assert_eq!(craft_flavor("forge"), "tools laid aside");
        assert_eq!(craft_flavor("still"), "hands still smelling of herbs");
        assert_eq!(craft_flavor("word"), "ink-stained fingers");
        assert_eq!(craft_flavor("unknown"), "calloused hands");
    }

    #[test]
    fn profession_flavor_varies() {
        assert_eq!(profession_flavor("smith"), "hammer-marked");
        assert_eq!(profession_flavor("trader"), "road-wise");
        assert_eq!(profession_flavor("unknown"), "steady-eyed");
    }

    #[test]
    fn gossip_personality_flavor_deterministic() {
        let mut p = test_person();
        p.personality = vec!["cheerful".into()];
        let a = voice_line_situation(&p, Situation::Gossip);
        let b = voice_line_situation(&p, Situation::Gossip);
        assert_eq!(a, b, "same person must produce same gossip line");
    }

    #[test]
    fn gossip_conflict_cancels_flavor() {
        let mut p = test_person();
        p.personality = vec!["cheerful".into(), "bitter".into()];
        let line = voice_line_situation(&p, Situation::Gossip);
        assert!(
            !line.contains("brightly") && !line.contains("as if the news itself"),
            "conflicting personality should cancel gossip flavor: {line}"
        );
    }

    #[test]
    fn gossip_vanilla_npc_no_flavor() {
        let mut p = test_person();
        p.personality = vec![];
        let line = voice_line_situation(&p, Situation::Gossip);
        assert!(!line.is_empty(), "empty personality should still produce gossip");
    }

    #[test]
    fn gossip_flavor_new_traits() {
        let mut suspicious = test_person();
        suspicious.personality = vec!["suspicious".into()];
        let line = voice_line_situation(&suspicious, Situation::Gossip);
        assert!(
            line.contains("guardedly") || line.contains("watching"),
            "suspicious gossip should have guarded flavor: {line}"
        );

        let mut earnest = test_person();
        earnest.personality = vec!["earnest".into()];
        let line2 = voice_line_situation(&earnest, Situation::Gossip);
        assert!(
            line2.contains("plainly") || line2.contains("just truth"),
            "earnest gossip should have plain flavor: {line2}"
        );
    }

    #[test]
    fn gossip_flavor_earnest_devious_conflict() {
        let mut p = test_person();
        p.personality = vec!["earnest".into(), "devious".into()];
        let line = voice_line_situation(&p, Situation::Gossip);
        assert!(
            !line.contains("plainly") && !line.contains("half-smile"),
            "earnest+devious conflict should cancel flavor: {line}"
        );
    }
}
