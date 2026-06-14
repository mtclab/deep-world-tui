use crate::model::Person;
use crate::voice::Situation;

#[cfg(feature = "llm")]
use std::time::Duration;

#[cfg(feature = "llm")]
use std::sync::Mutex;
#[cfg(feature = "llm")]
use std::time::Instant;

#[cfg(feature = "llm")]
static LAST_CALL: Mutex<Option<Instant>> = Mutex::new(None);
#[cfg(feature = "llm")]
const RATE_LIMIT_SECS: u64 = 2;

pub fn narrate(_person: &Person, _prompt: &str) -> Option<String> {
    #[cfg(feature = "llm")]
    {
        crate::llm::call_ollama("http://localhost:11434", "deep-world", _prompt, 30)
            .map(|s| crate::llm::strip_numbers(&s))
    }
    #[cfg(not(feature = "llm"))]
    {
        None
    }
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
        " Situation: {}. Respond naturally in character, staying brief (a few sentences).",
        situation.as_str()
    );

    let mut prompt = people_ctx;
    prompt.push_str(&personality_part);
    prompt.push_str(&craft_part);
    prompt.push_str(&age_part);
    prompt.push_str(&situation_ctx);
    prompt
}

pub fn build_flavor_context(
    situation: Situation,
    terrain: &str,
    weather: &str,
    people_kind: &str,
) -> String {
    format!(
        "Situation: {}. Terrain: {}. Weather: {}. People: {}. \
         Respond naturally in character, staying brief (a few sentences). \
         Do not include numbers or statistics.",
        situation.as_str(),
        terrain,
        weather,
        people_kind,
    )
}

pub fn strip_numbers(text: &str) -> String {
    text.chars().filter(|c| !c.is_ascii_digit()).collect()
}

#[cfg(feature = "llm")]
pub fn call_ollama(endpoint: &str, model: &str, prompt: &str, timeout_secs: u64) -> Option<String> {
    {
        let mut last = LAST_CALL.lock().ok()?;
        if let Some(prev) = *last {
            let elapsed = prev.elapsed().as_secs();
            if elapsed < RATE_LIMIT_SECS {
                return None;
            }
        }
        *last = Some(Instant::now());
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .ok()?;

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
    });

    let resp = client
        .post(format!("{}/api/generate", endpoint.trim_end_matches('/')))
        .json(&body)
        .send()
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let json: serde_json::Value = resp.json().ok()?;
    json.get("response")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[cfg(feature = "llm")]
pub fn narrate_with_llm(endpoint: &str, model: &str, context: &str) -> Option<String> {
    call_ollama(endpoint, model, context, 30).map(|s| strip_numbers(&s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Needs;
    use crate::model::NpcSchedule;

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
            schedule: NpcSchedule::default(),
            illnesses: Vec::new(),
            relations: vec![],
            wants: vec![],
            gift: Default::default(),
            age_years: 0,
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

    #[test]
    fn strip_numbers_removes_digits() {
        assert_eq!(strip_numbers("abc123def"), "abcdef");
        assert_eq!(strip_numbers("no digits"), "no digits");
        assert_eq!(strip_numbers("42"), "");
        assert_eq!(strip_numbers(""), "");
    }

    #[test]
    fn strip_numbers_preserves_other_chars() {
        assert_eq!(strip_numbers("hello, world!"), "hello, world!");
        assert_eq!(strip_numbers("price: 5 gold"), "price:  gold");
    }

    #[test]
    fn build_flavor_context_is_number_free() {
        let ctx = build_flavor_context(Situation::Greeting, "forest", "rain", "metsik");
        for c in ctx.chars() {
            assert!(
                !c.is_ascii_digit(),
                "context must not contain digits: found '{}'",
                c
            );
        }
    }

    #[test]
    fn build_flavor_context_includes_fields() {
        let ctx = build_flavor_context(Situation::Trade, "mountain", "snow", "sepat");
        assert!(ctx.contains("trade"));
        assert!(ctx.contains("mountain"));
        assert!(ctx.contains("snow"));
        assert!(ctx.contains("sepat"));
    }

    #[cfg(feature = "llm")]
    #[test]
    fn call_ollama_returns_none_on_bad_endpoint() {
        let result = call_ollama("http://127.0.0.1:1", "test-model", "hello", 2);
        assert!(result.is_none());
    }

    #[cfg(feature = "llm")]
    #[test]
    fn narrate_with_llm_returns_none_on_bad_endpoint() {
        let result = narrate_with_llm("http://127.0.0.1:1", "test-model", "hello");
        assert!(result.is_none());
    }
}
