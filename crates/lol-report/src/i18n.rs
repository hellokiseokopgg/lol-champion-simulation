use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct LocaleData {
    pub buffs: HashMap<String, String>,
}

pub struct Translator {
    pub locale_data: LocaleData,
}

impl Translator {
    pub fn new(lang: &str) -> Self {
        let path = format!("data/locales/{}.json", lang);

        let json_content = if let Ok(content) = fs::read_to_string(&path) {
            content
        } else {
            // Fallback to 'ko' if the specified language file is not found
            tracing::warn!("Locale file {} not found, falling back to 'ko.json'", path);
            fs::read_to_string("data/locales/ko.json").unwrap_or_else(|_| {
                // If even fallback fails, just return empty mappings
                r#"{"buffs": {}}"#.to_string()
            })
        };

        let locale_data: LocaleData =
            serde_json::from_str(&json_content).unwrap_or_else(|_| LocaleData {
                buffs: HashMap::new(),
            });

        Self { locale_data }
    }

    pub fn translate_buff(&self, english_name: &str) -> String {
        if let Some(translated) = self.locale_data.buffs.get(english_name) {
            translated.clone()
        } else {
            english_name.to_string()
        }
    }
}
