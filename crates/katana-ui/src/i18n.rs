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
    #[serde(default)]
    pub dialog: DialogMessages,
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
    #[serde(default = "default_downloading")]
    pub downloading: String,
    #[serde(default = "default_installing")]
    pub installing: String,
    #[serde(default = "default_restart_confirm")]
    pub restart_confirm: String,
    #[serde(default = "default_action_later")]
    pub action_later: String,
    #[serde(default = "default_action_skip_version")]
    pub action_skip_version: String,
    #[serde(default = "default_action_restart")]
    pub action_restart: String,
}

fn default_install_update() -> String {
    "Install and Relaunch".to_string()
}
fn default_downloading() -> String {
    "Downloading update...".to_string()
}
fn default_installing() -> String {
    "Installing update...".to_string()
}
fn default_restart_confirm() -> String {
    "Update is ready. Restart now?".to_string()
}
fn default_action_later() -> String {
    "Later".to_string()
}
fn default_action_skip_version() -> String {
    "Skip This Version".to_string()
}
fn default_action_restart() -> String {
    "Restart Now".to_string()
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
    #[serde(default = "default_action_new_file")]
    pub new_file: String,
    #[serde(default = "default_action_new_directory")]
    pub new_directory: String,
    #[serde(default = "default_action_open")]
    pub open: String,
    #[serde(default = "default_action_rename")]
    pub rename: String,
    #[serde(default = "default_action_delete")]
    pub delete: String,
    #[serde(default = "default_action_copy_path")]
    pub copy_path: String,
    #[serde(default = "default_action_copy_relative_path")]
    pub copy_relative_path: String,
    #[serde(default = "default_action_reveal_in_os")]
    pub reveal_in_os: String,
    #[serde(default = "default_action_save")]
    pub save: String,
    #[serde(default = "default_action_cancel")]
    pub cancel: String,
    #[serde(default = "default_action_discard")]
    pub discard: String,
    #[serde(default = "default_action_confirm")]
    pub confirm: String,
}

fn default_action_confirm() -> String {
    "Confirm".to_string()
}

fn default_action_new_file() -> String {
    "New File".to_string()
}
fn default_action_new_directory() -> String {
    "New Folder".to_string()
}
fn default_action_open() -> String {
    "Open".to_string()
}
fn default_action_rename() -> String {
    "Rename".to_string()
}
fn default_action_delete() -> String {
    "Delete".to_string()
}
fn default_action_copy_path() -> String {
    "Copy Path".to_string()
}
fn default_action_copy_relative_path() -> String {
    "Copy Relative Path".to_string()
}
fn default_action_reveal_in_os() -> String {
    "Reveal in OS".to_string()
}
fn default_action_save() -> String {
    "Save".to_string()
}
fn default_action_cancel() -> String {
    "Cancel".to_string()
}
fn default_action_discard() -> String {
    "Discard".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct DialogMessages {
    pub new_file_title: String,
    pub new_directory_title: String,
    pub rename_title: String,
    pub delete_title: String,
    pub delete_confirm_msg: String,
    #[serde(default = "default_unsaved_changes_title")]
    pub unsaved_changes_title: String,
    #[serde(default = "default_unsaved_changes_msg")]
    pub unsaved_changes_msg: String,
}

fn default_unsaved_changes_title() -> String {
    "Unsaved Changes".to_string()
}
fn default_unsaved_changes_msg() -> String {
    "Do you want to save the changes you made to {name}?\n\nYour changes will be lost if you don't save them.".to_string()
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
    #[serde(default)]
    pub behavior: SettingsBehaviorMessages,
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
    #[serde(default = "default_custom_section")]
    pub custom_section: String,
    #[serde(default = "default_delete_custom")]
    pub delete_custom: String,
    #[serde(default = "default_save_custom_theme")]
    pub save_custom_theme: String,
    #[serde(default = "default_save_custom_theme_title")]
    pub save_custom_theme_title: String,
    #[serde(default = "default_theme_name_label")]
    pub theme_name_label: String,
    #[serde(default = "default_duplicate")]
    pub duplicate: String,
}

fn default_duplicate() -> String {
    "Duplicate...".to_string()
}

fn default_custom_section() -> String {
    "Custom".to_string()
}

fn default_delete_custom() -> String {
    "Delete Custom Theme".to_string()
}

fn default_save_custom_theme() -> String {
    "Save as Custom Theme...".to_string()
}

fn default_save_custom_theme_title() -> String {
    "Save Custom Theme".to_string()
}

fn default_theme_name_label() -> String {
    "Theme Name:".to_string()
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
    #[serde(default = "default_visible_extensions_msg")]
    pub visible_extensions: String,
    #[serde(default = "default_no_extension_label")]
    pub no_extension_label: String,
    #[serde(default = "default_no_extension_warning_title")]
    pub no_extension_warning_title: String,
    #[serde(default = "default_no_extension_warning")]
    pub no_extension_warning: String,
    #[serde(default = "default_extensionless_excludes")]
    pub extensionless_excludes: String,
    #[serde(default = "default_extensionless_excludes_hint")]
    pub extensionless_excludes_hint: String,
}

fn default_extensionless_excludes() -> String {
    "Ignored Extensionless Files".to_string()
}
fn default_extensionless_excludes_hint() -> String {
    "Comma-separated list of exact file names to ignore when 'No Extension' is enabled (e.g., .DS_Store, .gitignore).".to_string()
}

fn default_no_extension_label() -> String {
    "No Extension".to_string()
}
fn default_no_extension_warning_title() -> String {
    "Warning".to_string()
}
fn default_no_extension_warning() -> String {
    "There is no guarantee that files without extensions can be displayed correctly as Markdown. Furthermore, the application may crash due to unexpected behavior. Are you sure you want to enable this?".to_string()
}

fn default_visible_extensions_msg() -> String {
    "Visible Extensions".to_string()
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

/// i18n messages for the Behavior settings tab.
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct SettingsBehaviorMessages {
    /// Section heading.
    pub section_title: String,
    /// Label for the confirm-close-dirty-tab toggle.
    pub confirm_close_dirty_tab: String,
    /// Label for the scroll-sync toggle.
    pub scroll_sync: String,
    /// Label for the auto-save toggle.
    pub auto_save: String,
    /// Label for the auto-save interval input.
    pub auto_save_interval: String,
    /// Unit label for seconds.
    pub seconds: String,
    /// Confirmation dialog title.
    pub close_confirm_title: String,
    /// Confirmation dialog message template.
    pub close_confirm_msg: String,
    /// Confirmation dialog discard button.
    pub close_confirm_discard: String,
    /// Confirmation dialog cancel button.
    pub close_confirm_cancel: String,
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
    fn test_i18n_default_action_values() {
        assert_eq!(default_action_new_file(), "New File");
        assert_eq!(default_action_new_directory(), "New Folder");
        assert_eq!(default_action_open(), "Open");
        assert_eq!(default_action_rename(), "Rename");
        assert_eq!(default_action_delete(), "Delete");
        assert_eq!(default_action_copy_path(), "Copy Path");
        assert_eq!(default_action_copy_relative_path(), "Copy Relative Path");
        assert_eq!(default_action_reveal_in_os(), "Reveal in OS");
        assert_eq!(default_action_save(), "Save");
        assert_eq!(default_action_cancel(), "Cancel");
    }

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
    fn test_default_extensionless_excludes() {
        assert_eq!(
            super::default_extensionless_excludes(),
            "Ignored Extensionless Files"
        );
        assert_eq!(
            super::default_extensionless_excludes_hint(),
            "Comma-separated list of exact file names to ignore when 'No Extension' is enabled (e.g., .DS_Store, .gitignore)."
        );
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

    #[test]
    fn test_update_messages_serde_defaults_backward_compat() {
        // Exercises all serde(default) fallback functions for UpdateMessages,
        // simulating a locale file that predates the v0.7.2 update keys.
        let minimal_json = r#"{
            "title": "T",
            "checking_for_updates": "C",
            "update_available": "A",
            "update_available_desc": "D",
            "up_to_date": "U",
            "up_to_date_desc": "UD",
            "failed_to_check": "F",
            "action_close": "OK"
        }"#;
        let msgs: super::UpdateMessages = serde_json::from_str(minimal_json).unwrap();
        assert_eq!(msgs.install_update, "Install and Relaunch");
        assert_eq!(msgs.downloading, "Downloading update...");
        assert_eq!(msgs.installing, "Installing update...");
        assert_eq!(msgs.restart_confirm, "Update is ready. Restart now?");
        assert_eq!(msgs.action_later, "Later");
        assert_eq!(msgs.action_skip_version, "Skip This Version");
        assert_eq!(msgs.action_restart, "Restart Now");
    }

    #[test]
    fn test_default_visible_extensions_msg() {
        assert_eq!(
            super::default_visible_extensions_msg(),
            "Visible Extensions"
        );
    }

    #[test]
    fn test_default_custom_theme_messages() {
        assert_eq!(super::default_custom_section(), "Custom");
        assert_eq!(super::default_delete_custom(), "Delete Custom Theme");
        assert_eq!(
            super::default_save_custom_theme(),
            "Save as Custom Theme..."
        );
        assert_eq!(
            super::default_save_custom_theme_title(),
            "Save Custom Theme"
        );
        assert_eq!(super::default_theme_name_label(), "Theme Name:");
    }
}

#[cfg(test)]
mod additional_coverage_tests {
    use super::*;

    #[test]
    fn test_i18n_defaults_coverage() {
        assert_eq!(default_action_confirm(), "Confirm");
        assert_eq!(default_action_discard(), "Discard");
        assert_eq!(default_duplicate(), "Duplicate...");
        assert_eq!(default_no_extension_label(), "No Extension");
        assert_eq!(default_no_extension_warning_title(), "Warning");
        assert_eq!(
            default_no_extension_warning(),
            "There is no guarantee that files without extensions can be displayed correctly as Markdown. Furthermore, the application may crash due to unexpected behavior. Are you sure you want to enable this?"
        );
    }
}
