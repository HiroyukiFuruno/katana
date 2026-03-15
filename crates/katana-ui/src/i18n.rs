use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// Language definition JSON entry.
#[derive(serde::Deserialize)]
struct LanguageEntry {
    code: String,
    name: String,
}

/// List of supported languages. Loaded from `locales/languages.json`.
/// Each element is a pair of (language code, endonym).
/// To add a language, simply add a line to `languages.json`.
pub fn supported_languages() -> &'static [(String, String)] {
    static LANGS: OnceLock<Vec<(String, String)>> = OnceLock::new();
    LANGS.get_or_init(|| {
        let json = include_str!("../locales/languages.json");
        let entries: Vec<LanguageEntry> =
            serde_json::from_str(json).expect("Failed to parse languages.json");
        entries.into_iter().map(|e| (e.code, e.name)).collect()
    })
}

/// Returns the endonym from the language code.
pub fn display_name(lang_code: &str) -> &'static str {
    supported_languages()
        .iter()
        .find(|(code, _)| code == lang_code)
        .map(|(_, name)| name.as_str())
        .unwrap_or("???")
}

/// Definition of locale JSON data.
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
        // Constant JSON embedded at compile time via include_str!. Parse failure is impossible.
        map.insert(
            "en",
            serde_json::from_str(EN_JSON).expect("BUG: en.json is invalid"),
        );
        map.insert(
            "ja",
            serde_json::from_str(JA_JSON).expect("BUG: ja.json is invalid"),
        );
        map
    })
}

/// Sets the current language.
pub fn set_language(lang: &str) {
    if let Ok(mut current) = CURRENT_LANGUAGE.write() {
        *current = lang.to_string();
    }
}

/// Gets the current language.
pub fn get_language() -> String {
    init_current_language();
    CURRENT_LANGUAGE.read().unwrap().clone()
}

/// Gets the translated string corresponding to the specified key.
pub fn t(key: &str) -> String {
    let lang = get_language();
    let dict = get_dictionary();
    if let Some(d) = dict.get(lang.as_str()) {
        if let Some(val) = d.get(key) {
            return val.clone();
        }
    }
    key.to_string()
}

/// Gets the parameter-substituted translated string corresponding to the specified key.
///
/// Replaces `{param}` placeholders in the translated string with `params` values.
///
/// # Example
/// ```ignore
/// // en.json: "status_saved_as": "Saved as {name}"
/// let msg = i18n::tf("status_saved_as", &[("name", "foo.md")]);
/// assert_eq!(msg, "Saved as foo.md");
/// ```
pub fn tf(key: &str, params: &[(&str, &str)]) -> String {
    let mut text = t(key);
    for (k, v) in params {
        text = text.replace(&format!("{{{k}}}"), v);
    }
    text
}
