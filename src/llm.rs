/// Optional LLM narrator (reqwest /v1). Feature-gated behind `llm`.
/// Player-toggled in settings; falls back to voice.rs on any error.
/// Stub for issue #7.
use crate::model::Person;

pub fn narrate(_person: &Person, _prompt: &str) -> Option<String> {
    None
}
