use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "deep-world-tui";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplayConfig {
    #[serde(default)]
    pub monochrome: bool,
    #[serde(default)]
    pub high_contrast: bool,
    #[serde(default)]
    pub reduced_motion: bool,
}

fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join(APP_NAME))
}

fn config_path() -> Option<PathBuf> {
    config_dir().map(|p| p.join(CONFIG_FILE))
}

pub fn load() -> UserConfig {
    if let Some(path) = config_path() {
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&data) {
                    return config;
                }
            }
        }
    }
    UserConfig::default()
}

pub fn save(config: &UserConfig) -> Result<(), String> {
    let dir = config_dir().ok_or_else(|| "No config directory available".to_string())?;
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(CONFIG_FILE);
    let data = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| format!("Failed to write config: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_sane() {
        let c = UserConfig::default();
        assert!(!c.display.monochrome);
        assert!(!c.display.high_contrast);
        assert!(!c.display.reduced_motion);
    }

    #[test]
    fn roundtrip_toml() {
        let c = UserConfig {
            display: DisplayConfig {
                monochrome: true,
                high_contrast: true,
                reduced_motion: true,
            },
        };
        let s = toml::to_string_pretty(&c).unwrap();
        let c2: UserConfig = toml::from_str(&s).unwrap();
        assert_eq!(c.display.monochrome, c2.display.monochrome);
        assert_eq!(c.display.high_contrast, c2.display.high_contrast);
        assert_eq!(c.display.reduced_motion, c2.display.reduced_motion);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let s = "";
        let c: UserConfig = toml::from_str(s).unwrap();
        assert!(!c.display.monochrome);
    }

    #[test]
    fn partial_display_config() {
        let s = "[display]\nhigh_contrast = true\n";
        let c: UserConfig = toml::from_str(s).unwrap();
        assert!(c.display.high_contrast);
        assert!(!c.display.monochrome);
        assert!(!c.display.reduced_motion);
    }
}
