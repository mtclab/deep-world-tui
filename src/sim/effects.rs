use serde::{Deserialize, Serialize};

/// An effect applied to the world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Immediate { description: String },
    Deferred { at_tick: u64, description: String },
}

impl Effect {
    pub fn immediate(desc: &str) -> Self {
        Effect::Immediate {
            description: desc.to_string(),
        }
    }

    pub fn deferred(desc: &str, at_tick: u64) -> Self {
        Effect::Deferred {
            at_tick,
            description: desc.to_string(),
        }
    }
}
