use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const LOCALES_DIR: &str = "data/locales";
pub const DEFAULT_LOCALE: &str = "en";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Locale {
    pub code: String,
    pub strings: HashMap<String, String>,
}

impl Default for Locale {
    fn default() -> Self {
        Self::load(DEFAULT_LOCALE)
    }
}

impl Locale {
    pub fn load(code: &str) -> Self {
        let path = format!("{}/{}.ron", LOCALES_DIR, code);
        if Path::new(&path).exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(locale) = ron::from_str::<Locale>(&data) {
                    return locale;
                }
            }
        }
        Self {
            code: DEFAULT_LOCALE.into(),
            strings: HashMap::new(),
        }
    }

    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).map(String::as_str).unwrap_or(key)
    }

    pub fn t_fmt(&self, key: &str, args: &[&str]) -> String {
        let template = self.t(key);
        let mut out = String::with_capacity(template.len());
        let mut i = 0;
        let mut arg_idx = 0;
        let bytes = template.as_bytes();
        while i < bytes.len() {
            if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'}' {
                if let Some(arg) = args.get(arg_idx) {
                    out.push_str(arg);
                    arg_idx += 1;
                } else {
                    out.push_str("{}");
                }
                i += 2;
            } else {
                let ch = template[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
        }
        out
    }
}

pub fn current() -> Locale {
    let lang = crate::ui::settings::AppSettings::load().language;
    Locale::load(&lang)
}

pub fn t(key: &str) -> String {
    current().t(key).to_string()
}

pub fn t_fmt(key: &str, args: &[&str]) -> String {
    current().t_fmt(key, args)
}

#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::t($key)
    };
    ($key:expr, $( $arg:expr ),+ ) => {
        $crate::i18n::t_fmt($key, &[$( $arg ),+])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_locale_loads() {
        let locale = Locale::default();
        assert_eq!(locale.code, DEFAULT_LOCALE);
    }

    #[test]
    fn missing_locale_falls_back() {
        let locale = Locale::load("xx-nonexistent");
        assert_eq!(locale.code, DEFAULT_LOCALE);
        assert!(locale.strings.is_empty());
    }

    #[test]
    fn missing_key_returns_key() {
        let locale = Locale::default();
        assert_eq!(locale.t("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn t_macro_returns_string() {
        let result = t("missing.key");
        assert_eq!(result, "missing.key");
    }

    #[test]
    fn t_fmt_substitutes() {
        let mut locale = Locale::default();
        locale
            .strings
            .insert("greeting".into(), "Hello, {}!".into());
        let out = locale.t_fmt("greeting", &["World"]);
        assert_eq!(out, "Hello, World!");
    }

    #[test]
    fn t_fmt_missing_args_passthrough() {
        let mut locale = Locale::default();
        locale.strings.insert("two_args".into(), "{} and {}".into());
        let out = locale.t_fmt("two_args", &["Alice"]);
        assert_eq!(out, "Alice and {}");
    }

    #[test]
    fn t_fmt_missing_key_returns_key() {
        let locale = Locale::default();
        let out = locale.t_fmt("nope", &["x"]);
        assert_eq!(out, "nope");
    }
}
