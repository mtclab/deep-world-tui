use crate::model::{Need, PeopleKind, Person};

#[derive(Debug, Clone, Copy)]
pub enum Situation {
    Greeting,
    Trade,
    NeedDire,
    NeedFine,
    Farewell,
    Gossip,
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

pub fn voice_line_situation(person: &Person, situation: Situation) -> String {
    let name = &person.name;
    let people = &person.people;
    let profession = &person.profession;
    let craft = &person.craft_affinity;

    let low_food = person.needs.get(Need::Food) < 0.3;
    let low_money = person.needs.get(Need::Money) < 0.3;
    let has_craft = craft != "none";

    match situation {
        Situation::Greeting => {
            if low_food {
                format!("{name} of the {people} nods weakly. \"I cannot think past my hunger.\"")
            } else if has_craft {
                format!("{name} sets down {craft} tools. \"Welcome. The {people} remember their friends.\"")
            } else {
                format!("{name} of the {people} regards you steadily. \"Another day in the Archive's shadow.\"")
            }
        }
        Situation::Trade => {
            if low_money {
                format!("{name} the {profession} shakes {people} head slowly. \"I have nothing to trade but my word.\"")
            } else {
                format!("{name} the {profession} gestures toward the goods. \"Fair exchange keeps the world turning.\"")
            }
        }
        Situation::NeedDire => {
            if low_food {
                format!("{name} clutches {people} stomach. \"The hunger eats my thoughts. I cannot focus.\"")
            } else if low_money {
                format!("{name} stares at empty hands. \"Debts are stones around my neck. The {people} see all.\"")
            } else {
                format!("{name} looks away. \"Some needs go deeper than coin or bread.\"")
            }
        }
        Situation::NeedFine => {
            format!("{name} the {profession} stands easy. \"The {people} have known worse seasons. This one holds.\"")
        }
        Situation::Farewell => {
            if has_craft {
                format!("{name} returns to the {craft} work. \"May the Archive hold what we have built.\"")
            } else {
                format!("{name} of the {people} steps back. \"Until the next turning.\"")
            }
        }
        Situation::Gossip => {
            if low_food {
                format!("{name} leans close. \"They say the storehouses thin. The {people} are watching.\"")
            } else {
                format!("{name} the {profession} speaks low. \"Word travels the river roads. Listen, and the {people} will tell you.\"")
            }
        }
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
        match situation {
            Situation::Greeting => "They barely glance at you. ",
            Situation::Trade => "Arms fold. 'We don't trade with your kind.' ",
            Situation::Farewell => "A curt nod. Nothing more. ",
            _ => "",
        }
    } else {
        match situation {
            Situation::Greeting => "Eyes narrow slightly. ",
            Situation::Trade => "Reluctant hands count the coins twice. ",
            Situation::Farewell => "A guarded farewell. ",
            _ => "",
        }
    };

    format!("{prefix}{base}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Needs;

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
        }
    }

    #[test]
    fn greeting_with_craft() {
        let p = test_person();
        let line = voice_line_situation(&p, Situation::Greeting);
        assert!(line.contains("Metsik"));
        assert!(line.contains("forge"));
        assert!(line.contains("Sepät"));
    }

    #[test]
    fn trade_low_money() {
        let mut p = test_person();
        p.needs.satisfy(Need::Money, -0.8);
        let line = voice_line_situation(&p, Situation::Trade);
        assert!(line.contains("nothing to trade"));
    }

    #[test]
    fn need_dire_low_food() {
        let mut p = test_person();
        p.needs.satisfy(Need::Food, -0.8);
        let line = voice_line_situation(&p, Situation::NeedDire);
        assert!(line.contains("hunger"));
    }

    #[test]
    fn need_fine() {
        let p = test_person();
        let line = voice_line_situation(&p, Situation::NeedFine);
        assert!(line.contains("worse seasons"));
    }

    #[test]
    fn farewell_with_craft() {
        let p = test_person();
        let line = voice_line_situation(&p, Situation::Farewell);
        assert!(line.contains("forge"));
    }

    #[test]
    fn gossip_well_fed() {
        let p = test_person();
        let line = voice_line_situation(&p, Situation::Gossip);
        assert!(line.contains("river roads"));
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
}
