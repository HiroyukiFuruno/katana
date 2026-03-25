use serde::Deserialize;
use std::sync::{OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

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
    pub search: SearchMessages,
    pub about: AboutMessages,
    pub update: UpdateMessages,
    pub toc: TocMessages,
    #[serde(default)]
    pub export: ExportMessages,
    #[serde(default)]
    pub terms: TermsMessages,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct ExportMessages {
    pub success: String,
    pub failed: String,
    pub tool_missing: String,
    pub temp_file_error: String,
    pub write_error: String,
    pub persist_error: String,
    pub exporting: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct TermsMessages {
    pub title: String,
    pub version_label: String,
    pub content: String,
    pub accept: String,
    pub decline: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TocMessages {
    pub title: String,
    pub empty: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SearchMessages {
    pub modal_title: String,
    pub query_hint: String,
    pub include_pattern_hint: String,
    pub exclude_pattern_hint: String,
    pub no_results: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct AboutMessages {
    pub basic_info: String,
    pub version: String,
    pub build: String,
    pub copyright: String,
    pub runtime: String,
    pub platform: String,
    pub architecture: String,
    pub rust: String,
    pub license: String,
    pub links: String,
    pub source_code: String,
    pub documentation: String,
    pub report_issue: String,
    pub support: String,
    pub sponsor: String,
    pub coming_soon: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct UpdateMessages {
    pub title: String,
    pub checking_for_updates: String,
    pub update_available: String,
    pub update_available_desc: String,
    pub up_to_date: String,
    pub up_to_date_desc: String,
    pub failed_to_check: String,
    pub action_close: String,
    #[serde(default = "default_install_update")]
    pub install_update: String,
}

fn default_install_update() -> String {
    "Install and Relaunch".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MenuMessages {
    pub file: String,
    pub settings: String,
    pub language: String,
    pub open_workspace: String,
    pub save: String,
    pub open_all: String,
    pub about: String,
    pub quit: String,
    pub hide: String,
    pub hide_others: String,
    pub show_all: String,
    #[serde(default)]
    pub export: String,
    #[serde(default)]
    pub export_html: String,
    #[serde(default)]
    pub export_pdf: String,
    #[serde(default)]
    pub export_png: String,
    #[serde(default)]
    pub export_jpg: String,
    pub help: String,
    pub check_updates: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct WorkspaceMessages {
    pub no_workspace_open: String,
    pub no_document_selected: String,
    pub workspace_title: String,
    pub recent_workspaces: String,
    #[serde(default = "default_metadata_tooltip")]
    pub metadata_tooltip: String,
    pub path_label: String,
}

fn default_metadata_tooltip() -> String {
    "Size: {size} B\nModified: {mod_time}".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DiagramControllerMessages {
    pub pan_up: String,
    pub pan_down: String,
    pub pan_left: String,
    pub pan_right: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub reset: String,
    pub fullscreen: String,
    pub close: String,
    #[serde(default = "default_trackpad_help")]
    pub trackpad_help: String,
}

fn default_trackpad_help() -> String {
    "Trackpad: 2-finger pinch to zoom, 1-finger drag to pan".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PreviewMessages {
    pub preview_title: String,
    pub refresh_diagrams: String,
    pub rendering: String,
    pub no_preview: String,
    #[serde(default = "default_diagram_controller")]
    pub diagram_controller: DiagramControllerMessages,
}

fn default_diagram_controller() -> DiagramControllerMessages {
    DiagramControllerMessages {
        pan_up: "Move up".to_string(),
        pan_down: "Move down".to_string(),
        pan_left: "Move left".to_string(),
        pan_right: "Move right".to_string(),
        zoom_in: "Zoom in".to_string(),
        zoom_out: "Zoom out".to_string(),
        reset: "Reset position and size".to_string(),
        fullscreen: "Fullscreen".to_string(),
        close: "Close".to_string(),
        trackpad_help: default_trackpad_help(),
    }
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

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct ActionMessages {
    pub expand_all: String,
    pub collapse_all: String,
    pub collapse_sidebar: String,
    pub refresh_workspace: String,
    pub toggle_filter: String,
    pub remove_workspace: String,
    #[serde(default)]
    pub recursive_expand: String,
    #[serde(default)]
    pub recursive_open_all: String,
    #[serde(default)]
    pub toggle_toc: String,
    pub show_meta_info: String,
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
    pub workspace: SettingsWorkspaceMessages,
    #[serde(default)]
    pub updates: SettingsUpdatesMessages,
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
    pub toc_position: String,
    pub left: String,
    pub right: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SettingsWorkspaceMessages {
    pub max_depth: String,
    pub ignored_directories: String,
    pub ignored_directories_hint: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct SettingsUpdatesMessages {
    pub section_title: String,
    pub interval: String,
    pub never: String,
    pub daily: String,
    pub weekly: String,
    pub monthly: String,
    pub check_now: String,
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
    #[serde(default)]
    pub close: String,
    #[serde(default)]
    pub close_others: String,
    #[serde(default)]
    pub close_all: String,
    #[serde(default)]
    pub close_right: String,
    #[serde(default)]
    pub close_left: String,
    #[serde(default)]
    pub pin: String,
    #[serde(default)]
    pub unpin: String,
    #[serde(default)]
    pub restore_closed: String,
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
pub struct I18nDictionaryEntry {
    pub lang: &'static str,
    pub messages: I18nMessages,
}

static DICTIONARY: OnceLock<Vec<I18nDictionaryEntry>> = OnceLock::new();
static CURRENT_LANGUAGE: RwLock<String> = RwLock::new(String::new());

fn read_guard<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(PoisonError::into_inner)
}

fn write_guard<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write().unwrap_or_else(PoisonError::into_inner)
}

fn init_current_language() {
    let mut lang = write_guard(&CURRENT_LANGUAGE);
    if lang.is_empty() {
        *lang = "en".to_string();
    }
}

fn get_dictionary() -> &'static Vec<I18nDictionaryEntry> {
    DICTIONARY.get_or_init(|| {
        let entries = vec![
            ("en", EN_JSON),
            ("ja", JA_JSON),
            ("zh-CN", ZH_CN_JSON),
            ("zh-TW", ZH_TW_JSON),
            ("ko", KO_JSON),
            ("pt", PT_JSON),
            ("fr", FR_JSON),
            ("de", DE_JSON),
            ("es", ES_JSON),
            ("it", IT_JSON),
        ];
        let mut vec = Vec::new();
        for (code, json) in entries {
            vec.push(I18nDictionaryEntry {
                lang: code,
                messages: serde_json::from_str(json)
                    .unwrap_or_else(|e| panic!("BUG: {code}.json is invalid: {e}")),
            });
        }
        vec
    })
}

/// Sets the current language.
pub fn set_language(lang: &str) {
    let mut current = write_guard(&CURRENT_LANGUAGE);
    *current = lang.to_string();
}

/// Gets the current language.
pub fn get_language() -> String {
    init_current_language();
    read_guard(&CURRENT_LANGUAGE).clone()
}

/// Access the strongly-typed message hierarchy.
pub fn get() -> &'static I18nMessages {
    let lang = get_language();
    let dict = get_dictionary();
    let fallback = &dict[0].messages;
    dict.iter()
        .find(|entry| entry.lang == lang.as_str())
        .map(|entry| &entry.messages)
        .unwrap_or(fallback)
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
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::RwLock;

    #[test]
    fn test_get_fallback_to_en() {
        // Test that an unsupported language defaults to 'en' dictionary without failing.
        set_language("unsupported-lang-code");
        let msgs = get();
        // Just verify it returned a valid dictionary (falling back to "en").
        assert!(!msgs.menu.file.is_empty());
        // Restore to avoid polluting global state for other tests running in parallel.
        set_language("en");
    }

    #[test]
    fn test_default_metadata_tooltip() {
        assert_eq!(
            super::default_metadata_tooltip(),
            "Size: {size} B\nModified: {mod_time}"
        );
    }

    #[test]
    fn test_rwlock_helpers_recover_from_poison() {
        let lock = RwLock::new(String::new());
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = lock.write().expect("poison test must acquire write lock");
            panic!("poison language lock");
        }));

        {
            let mut guard = write_guard(&lock);
            *guard = "en".to_string();
        }

        assert_eq!(read_guard(&lock).as_str(), "en");
    }

    #[test]
    fn test_default_diagram_controller() {
        let dc = super::default_diagram_controller();
        assert_eq!(dc.pan_up, "Move up");
        assert_eq!(dc.zoom_in, "Zoom in");
        assert_eq!(dc.close, "Close");
    }

    #[test]
    fn test_export_menu_keys_exist() {
        // Red phase: testing that export menu strings are present
        let msgs = super::get();
        assert!(!msgs.menu.export.is_empty());
        assert!(!msgs.menu.export_html.is_empty());
        assert!(!msgs.menu.export_pdf.is_empty());
        assert!(!msgs.menu.export_png.is_empty());
        assert!(!msgs.menu.export_jpg.is_empty());
    }
}
