/// Deterministic templated dialogue from a Person's traits.
/// Always available (LLM is optional/off by default).
/// Stub for issue #7.
use crate::model::Person;

pub fn voice_line(_person: &Person, _situation: &str) -> String {
    String::from("...")
}
