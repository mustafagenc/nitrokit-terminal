use serde_json::Value;
use std::collections::HashMap;

pub struct I18n {
    translations: HashMap<String, Value>,
    current_locale: String,
}

impl I18n {
    pub fn new() -> Self {
        let mut i18n = I18n {
            translations: HashMap::new(),
            current_locale: "en".to_string(),
        };

        i18n.load_translations();
        i18n
    }

    fn load_translations(&mut self) {
        // English
        let en_json = include_str!("../locales/en.json");
        let en_translations: Value = serde_json::from_str(en_json).unwrap();
        self.translations.insert("en".to_string(), en_translations);

        // Turkish
        let tr_json = include_str!("../locales/tr.json");
        let tr_translations: Value = serde_json::from_str(tr_json).unwrap();
        self.translations.insert("tr".to_string(), tr_translations);
    }

    pub fn set_locale(&mut self, locale: &str) {
        if self.translations.contains_key(locale) {
            self.current_locale = locale.to_string();
        }
    }

    pub fn t(&self, key: &str) -> String {
        if let Some(translations) = self.translations.get(&self.current_locale) {
            if let Some(translation) = translations.get(key) {
                return translation.as_str().unwrap_or(key).to_string();
            }
        }
        key.to_string()
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}
