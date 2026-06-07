/// Optional LLM narrator (reqwest /v1). Feature-gated behind `llm`.
/// Player-toggled in settings; falls back to voice.rs on any error.
use crate::model::Person;
use crate::voice::Situation;

pub fn narrate(_person: &Person, _prompt: &str) -> Option<String> {
    None
}

pub fn narrate_with_fallback(
    llm_enabled: bool,
    person: &Person,
    situation: Situation,
    voice_text: &str,
) -> String {
    if llm_enabled {
        let prompt = build_persona_prompt(person, situation);
        narrate(person, &prompt).unwrap_or_else(|| voice_text.to_string())
    } else {
        voice_text.to_string()
    }
}

pub fn build_persona_prompt(person: &Person, situation: Situation) -> String {
    let people_ctx = format!(
        "You are {}, a {} {} of {} class.",
        person.name, person.people, person.profession, person.social_class
    );

    let personality_part = if person.personality.is_empty() {
        String::new()
    } else {
        format!(
            " Your personality traits are: {}.",
            person.personality.join(", ")
        )
    };

    let craft_part = if person.craft_affinity.is_empty() || person.craft_affinity == "none" {
        String::new()
    } else {
        format!(" Your craft affinity is {}.", person.craft_affinity)
    };

    let age_part = format!(" Your age band is {}.", person.age_band);

    let situation_ctx = format!(
        " Situation: {}. Respond naturally in character, staying brief (1-3 sentences).",
        situation.as_str()
    );

    let mut prompt = people_ctx;
    prompt.push_str(&personality_part);
    prompt.push_str(&craft_part);
    prompt.push_str(&age_part);
    prompt.push_str(&situation_ctx);
    prompt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Needs;

    fn test_person() -> Person {
        Person {
            id: "p1".into(),
            name: "Ketäva".into(),
            people: "metsik".into(),
            sex: "f".into(),
            age_band: "adult".into(),
            profession: "hunter".into(),
            social_class: "low".into(),
            craft_affinity: "wood".into(),
            personality: vec!["quiet".into(), "observant".into()],
            bias: "neutral".into(),
            needs: Needs::default(),
            region: "forest".into(),
            settlement: "Pöytäford".into(),
            has_spouse: false,
            children_count: 0,
            has_debt: false,
        }
    }

    #[test]
    fn prompt_is_non_empty() {
        let person = test_person();
        let prompt = build_persona_prompt(&person, Situation::Greeting);
        assert!(!prompt.is_empty());
    }

    #[test]
    fn prompt_includes_people_profession_class() {
        let person = test_person();
        let prompt = build_persona_prompt(&person, Situation::Greeting);
        assert!(prompt.contains("metsik"));
        assert!(prompt.contains("hunter"));
        assert!(prompt.contains("low"));
    }

    #[test]
    fn prompt_includes_personality() {
        let person = test_person();
        let prompt = build_persona_prompt(&person, Situation::Greeting);
        assert!(prompt.contains("quiet"));
        assert!(prompt.contains("observant"));
    }

    #[test]
    fn prompt_includes_craft_affinity() {
        let person = test_person();
        let prompt = build_persona_prompt(&person, Situation::Trade);
        assert!(prompt.contains("wood"));
    }

    #[test]
    fn prompt_excludes_empty_craft() {
        let mut person = test_person();
        person.craft_affinity = "none".into();
        let prompt = build_persona_prompt(&person, Situation::Greeting);
        assert!(!prompt.contains("Your craft affinity"));
    }

    #[test]
    fn prompt_includes_age_band() {
        let person = test_person();
        let prompt = build_persona_prompt(&person, Situation::Greeting);
        assert!(prompt.contains("adult"));
    }

    #[test]
    fn prompt_includes_situation() {
        let person = test_person();
        let prompt = build_persona_prompt(&person, Situation::NeedDire);
        assert!(prompt.contains("need_dire"));
    }

    #[test]
    fn same_person_same_situation_deterministic() {
        let person = test_person();
        let a = build_persona_prompt(&person, Situation::Greeting);
        let b = build_persona_prompt(&person, Situation::Greeting);
        assert_eq!(a, b);
    }

    #[test]
    fn different_people_produce_different_prompts() {
        let p1 = test_person();
        let mut p2 = test_person();
        p2.name = "Torvath".into();
        p2.people = "sepat".into();
        p2.profession = "smith".into();
        let prompt1 = build_persona_prompt(&p1, Situation::Greeting);
        let prompt2 = build_persona_prompt(&p2, Situation::Greeting);
        assert_ne!(prompt1, prompt2);
    }

    #[test]
    fn no_personality_produces_valid_prompt() {
        let mut person = test_person();
        person.personality = vec![];
        let prompt = build_persona_prompt(&person, Situation::Greeting);
        assert!(!prompt.is_empty());
        assert!(!prompt.contains("Your personality traits"));
    }

    #[test]
    fn narrate_with_fallback_returns_voice_when_disabled() {
        let person = test_person();
        let result = narrate_with_fallback(false, &person, Situation::Greeting, "Hello, traveler.");
        assert_eq!(result, "Hello, traveler.");
    }

    #[test]
    fn narrate_with_fallback_returns_voice_when_enabled_but_narrate_is_stub() {
        let person = test_person();
        let result = narrate_with_fallback(true, &person, Situation::Greeting, "Fallback text.");
        assert_eq!(result, "Fallback text.");
    }
}
