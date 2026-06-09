use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const SETTINGS_FILE: &str = "data/settings.ron";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub llm_enabled: bool,
    pub llm_endpoint: String,
    pub llm_model: String,
    pub monochrome: bool,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub language: String,
    pub audio_enabled: bool,
    pub audio_volume: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            llm_enabled: false,
            llm_endpoint: "http://localhost:11434/v1".into(),
            llm_model: "llama3".into(),
            monochrome: false,
            high_contrast: false,
            reduced_motion: false,
            language: "en".into(),
            audio_enabled: false,
            audio_volume: 0.5,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        if Path::new(SETTINGS_FILE).exists() {
            if let Ok(data) = fs::read_to_string(SETTINGS_FILE) {
                if let Ok(settings) = ron::from_str(&data) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(data) = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            let _ = fs::write(SETTINGS_FILE, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings() {
        let s = AppSettings::default();
        assert!(!s.llm_enabled);
        assert!(!s.monochrome);
        assert!(!s.llm_endpoint.is_empty());
        assert!(!s.llm_model.is_empty());
        assert!(!s.audio_enabled);
        assert!(s.audio_volume > 0.0 && s.audio_volume <= 1.0);
    }

    #[test]
    fn roundtrip_serde() {
        let s = AppSettings {
            llm_enabled: true,
            llm_endpoint: "http://test:8080/v1".into(),
            llm_model: "test-model".into(),
            monochrome: true,
            high_contrast: true,
            reduced_motion: true,
            language: "fi".into(),
            audio_enabled: true,
            audio_volume: 0.42,
        };
        let data = ron::ser::to_string_pretty(&s, ron::ser::PrettyConfig::default()).unwrap();
        let s2: AppSettings = ron::from_str(&data).unwrap();
        assert_eq!(s.llm_enabled, s2.llm_enabled);
        assert_eq!(s.llm_endpoint, s2.llm_endpoint);
        assert_eq!(s.llm_model, s2.llm_model);
        assert_eq!(s.monochrome, s2.monochrome);
        assert_eq!(s.high_contrast, s2.high_contrast);
        assert_eq!(s.reduced_motion, s2.reduced_motion);
        assert_eq!(s.language, s2.language);
        assert_eq!(s.audio_enabled, s2.audio_enabled);
        assert!((s.audio_volume - s2.audio_volume).abs() < 1e-6);
    }
}
