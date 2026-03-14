use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// ロケールJSONデータの定義。
const EN_JSON: &str = include_str!("../locales/en.json");
const JA_JSON: &str = include_str!("../locales/ja.json");

static DICTIONARY: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
static CURRENT_LANGUAGE: RwLock<String> = RwLock::new(String::new());

fn init_current_language() {
    let mut lang = CURRENT_LANGUAGE.write().unwrap();
    if lang.is_empty() {
        *lang = "en".to_string();
    }
}

fn get_dictionary() -> &'static HashMap<&'static str, HashMap<String, String>> {
    DICTIONARY.get_or_init(|| {
        let mut map = HashMap::new();
        if let Ok(json) = serde_json::from_str(EN_JSON) {
            map.insert("en", json);
        }
        if let Ok(json) = serde_json::from_str(JA_JSON) {
            map.insert("ja", json);
        }
        map
    })
}

/// 現在の言語を設定する。
pub fn set_language(lang: &str) {
    if let Ok(mut current) = CURRENT_LANGUAGE.write() {
        *current = lang.to_string();
    }
}

/// 現在の言語を取得する。
pub fn get_language() -> String {
    init_current_language();
    CURRENT_LANGUAGE.read().unwrap().clone()
}

/// 指定したキーに対応する翻訳文字列を取得する。
pub fn t(key: &str) -> String {
    let lang = get_language();
    let dict = get_dictionary();
    if let Some(d) = dict.get(lang.as_str()) {
        if let Some(val) = d.get(key) {
            return val.clone();
        }
    }
    // Fallback to english if missing in current lang
    if lang != "en" {
        if let Some(d) = dict.get("en") {
            if let Some(val) = d.get(key) {
                return val.clone();
            }
        }
    }
    
    key.to_string()
}
