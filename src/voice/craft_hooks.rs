use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CraftHookBank {
    pub greeting_openers: Vec<String>,
    pub trade_phrases: Vec<String>,
    pub farewell_phrases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CraftHooks {
    pub forge: CraftHookBank,
    pub still: CraftHookBank,
    pub word: CraftHookBank,
    pub current: CraftHookBank,
    pub root: CraftHookBank,
}

impl CraftHooks {
    pub fn load(path: &str) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read craft hooks: {}", e))?;
        ron::from_str(&contents).map_err(|e| format!("Failed to parse craft hooks: {}", e))
    }

    pub fn bank_for(&self, craft: &str) -> Option<&CraftHookBank> {
        match craft {
            "forge" => Some(&self.forge),
            "still" => Some(&self.still),
            "word" => Some(&self.word),
            "current" => Some(&self.current),
            "root" => Some(&self.root),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_craft_hooks() {
        let hooks = CraftHooks::load("data/voice/craft_hooks.ron").expect("should load");
        assert!(hooks.forge.greeting_openers.len() >= 5);
        assert!(hooks.still.trade_phrases.len() >= 5);
        assert!(hooks.word.farewell_phrases.len() >= 5);
        assert!(hooks.current.greeting_openers.len() >= 5);
        assert!(hooks.root.trade_phrases.len() >= 5);
    }

    #[test]
    fn bank_for_crafts() {
        let hooks = CraftHooks::load("data/voice/craft_hooks.ron").unwrap();
        for craft in &["forge", "still", "word", "current", "root"] {
            let bank = hooks
                .bank_for(craft)
                .unwrap_or_else(|| panic!("craft '{}' should have a bank", craft));
            assert!(
                bank.greeting_openers.len() >= 5,
                "{} should have >= 5 greeting openers",
                craft
            );
            assert!(
                bank.trade_phrases.len() >= 5,
                "{} should have >= 5 trade phrases",
                craft
            );
            assert!(
                bank.farewell_phrases.len() >= 5,
                "{} should have >= 5 farewell phrases",
                craft
            );
        }
    }

    #[test]
    fn bank_for_none_returns_none() {
        let hooks = CraftHooks::load("data/voice/craft_hooks.ron").unwrap();
        assert!(
            hooks.bank_for("none").is_none(),
            "no craft affinity should return None"
        );
    }
}
