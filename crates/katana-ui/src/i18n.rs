use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct I18nMessages {
    pub menu: MenuMessages,
    pub workspace: WorkspaceMessages,
    pub preview: PreviewMessages,
    pub plantuml: PlantumlMessages,
    pub view_mode: ViewModeMessages,
    pub split_toggle: SplitToggleMessages,
    pub error: ErrorMessages,
    pub status: StatusMessages,
    pub action: ActionMessages,
    pub ai: AiMessages,
    pub tool: ToolMessages,
    pub settings: SettingsMessages,
    pub tab: TabMessages,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MenuMessages {
    pub file: String,
    pub settings: String,
    pub language: String,
    pub open_workspace: String,
    pub save: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct WorkspaceMessages {
    pub no_workspace_open: String,
    pub no_document_selected: String,
    pub workspace_title: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PreviewMessages {
    pub preview_title: String,
    pub refresh_diagrams: String,
    pub rendering: String,
    pub no_preview: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PlantumlMessages {
    pub downloading_plantuml: String,
    pub plantuml_installed: String,
    pub download_error: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ViewModeMessages {
    pub preview: String,
    pub code: String,
    pub split: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SplitToggleMessages {
    pub horizontal: String,
    pub vertical: String,
    pub editor_first: String,
    pub preview_first: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ErrorMessages {
    pub missing_dependency: String,
    pub curl_launch_failed: String,
    pub download_failed: String,
    pub render_error: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StatusMessages {
    pub ready: String,
    pub saved: String,
    pub save_failed: String,
    pub opened_workspace: String,
    pub cannot_open_workspace: String,
    pub cannot_open_file: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ActionMessages {
    pub expand_all: String,
    pub collapse_all: String,
    pub collapse_sidebar: String,
    pub refresh_workspace: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct AiMessages {
    pub ai_unconfigured: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ToolMessages {
    pub not_installed: String,
    pub install_path: String,
    pub download: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SettingsMessages {
    pub title: String,
    pub tabs: Vec<SettingsTabMessage>,
    pub toc_visible: String,
    pub theme: SettingsThemeMessages,
    pub font: SettingsFontMessages,
    pub layout: SettingsLayoutMessages,
    pub preview: SettingsPreviewMessages,
    pub color: SettingsColorMessages,
}

impl SettingsMessages {
    pub fn tab_name(&self, key: &str) -> String {
        self.tabs
            .iter()
            .find(|t| t.key == key)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| key.to_string())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SettingsThemeMessages {
    pub preset: String,
    pub dark_section: String,
    pub light_section: String,
    pub custom_colors: String,
    pub reset_custom: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SettingsTabMessage {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SettingsFontMessages {
    pub size: String,
    pub family: String,
    pub size_slider_hint: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SettingsLayoutMessages {
    pub split_direction: String,
    pub horizontal: String,
    pub vertical: String,
    pub pane_order: String,
    pub editor_first: String,
    pub preview_first: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SettingsPreviewMessages {
    pub title: String,
    pub heading: String,
    pub normal_text: String,
    pub accent_link: String,
    pub secondary_text: String,
    pub code_sample: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SettingsColorMessages {
    pub background: String,
    pub panel_background: String,
    pub text: String,
    pub text_secondary: String,
    pub accent: String,
    pub border: String,
    pub selection: String,
    pub code_background: String,
    pub preview_background: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TabMessages {
    pub nav_prev: String,
    pub nav_next: String,
}

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
const ZH_CN_JSON: &str = include_str!("../locales/zh-CN.json");
const ZH_TW_JSON: &str = include_str!("../locales/zh-TW.json");
const KO_JSON: &str = include_str!("../locales/ko.json");
const PT_JSON: &str = include_str!("../locales/pt.json");
const FR_JSON: &str = include_str!("../locales/fr.json");
const DE_JSON: &str = include_str!("../locales/de.json");
const ES_JSON: &str = include_str!("../locales/es.json");
const IT_JSON: &str = include_str!("../locales/it.json");

static DICTIONARY: OnceLock<HashMap<&'static str, I18nMessages>> = OnceLock::new();
static CURRENT_LANGUAGE: RwLock<String> = RwLock::new(String::new());

fn init_current_language() {
    let mut lang = CURRENT_LANGUAGE.write().unwrap();
    if lang.is_empty() {
        *lang = "en".to_string();
    }
}

fn get_dictionary() -> &'static HashMap<&'static str, I18nMessages> {
    DICTIONARY.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert(
            "en",
            serde_json::from_str(EN_JSON).expect("BUG: en.json is invalid"),
        );
        map.insert(
            "ja",
            serde_json::from_str(JA_JSON).expect("BUG: ja.json is invalid"),
        );
        map.insert(
            "zh-CN",
            serde_json::from_str(ZH_CN_JSON).expect("BUG: zh-CN.json is invalid"),
        );
        map.insert(
            "zh-TW",
            serde_json::from_str(ZH_TW_JSON).expect("BUG: zh-TW.json is invalid"),
        );
        map.insert(
            "ko",
            serde_json::from_str(KO_JSON).expect("BUG: ko.json is invalid"),
        );
        map.insert(
            "pt",
            serde_json::from_str(PT_JSON).expect("BUG: pt.json is invalid"),
        );
        map.insert(
            "fr",
            serde_json::from_str(FR_JSON).expect("BUG: fr.json is invalid"),
        );
        map.insert(
            "de",
            serde_json::from_str(DE_JSON).expect("BUG: de.json is invalid"),
        );
        map.insert(
            "es",
            serde_json::from_str(ES_JSON).expect("BUG: es.json is invalid"),
        );
        map.insert(
            "it",
            serde_json::from_str(IT_JSON).expect("BUG: it.json is invalid"),
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

/// Access the strongly-typed message hierarchy.
pub fn get() -> &'static I18nMessages {
    let lang = get_language();
    let dict = get_dictionary();
    if let Some(msgs) = dict.get(lang.as_str()) {
        msgs
    } else {
        dict.get("en").unwrap()
    }
}

/// Gets the parameter-substituted translated string.
///
/// Replaces `{param}` placeholders in the template string with `params` values.
pub fn tf(template: &str, params: &[(&str, &str)]) -> String {
    let mut text = template.to_string();
    for (k, v) in params {
        text = text.replace(&format!("{{{k}}}"), v);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fallback_to_en() {
        // Test that an unsupported language defaults to 'en' dictionary without failing.
        set_language("unsupported-lang-code");
        let msgs = get();
        // Just verify it returned a valid dictionary (falling back to "en").
        assert!(!msgs.menu.file.is_empty());
    }
}
