use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// 言語定義 JSON のエントリ。
#[derive(serde::Deserialize)]
struct LanguageEntry {
    code: String,
    name: String,
}

/// サポートする言語一覧。`locales/languages.json` から読み込む。
/// 各要素は (言語コード, 自称名) のペア。
/// 言語を追加する場合は `languages.json` に1行追加するだけでよい。
pub fn supported_languages() -> &'static [(String, String)] {
    static LANGS: OnceLock<Vec<(String, String)>> = OnceLock::new();
    LANGS.get_or_init(|| {
        let json = include_str!("../locales/languages.json");
        let entries: Vec<LanguageEntry> =
            serde_json::from_str(json).expect("languages.json の解析に失敗");
        entries.into_iter().map(|e| (e.code, e.name)).collect()
    })
}

/// 言語コードから自称名を返す。
pub fn display_name(lang_code: &str) -> &'static str {
    supported_languages()
        .iter()
        .find(|(code, _)| code == lang_code)
        .map(|(_, name)| name.as_str())
        .unwrap_or("???")
}

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
    key.to_string()
}

/// 指定したキーに対応する翻訳文字列をパラメータ置換して取得する。
///
/// キーの翻訳文字列中の `{param}` プレースホルダを `params` の値に置き換える。
///
/// # 例
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
