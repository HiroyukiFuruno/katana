mod types;
pub use types::*;

use std::sync::{OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(serde::Deserialize)]
struct LanguageEntry {
    code: String,
    name: String,
}

pub fn supported_languages() -> &'static [(String, String)] {
    static LANGS: OnceLock<Vec<(String, String)>> = OnceLock::new();
    LANGS.get_or_init(|| {
        let json = include_str!("../../locales/languages.json");
        let entries: Vec<LanguageEntry> =
            serde_json::from_str(json).expect("Failed to parse languages.json");
        entries.into_iter().map(|e| (e.code, e.name)).collect()
    })
}

pub fn display_name(lang_code: &str) -> &'static str {
    supported_languages()
        .iter()
        .find(|(code, _)| code == lang_code)
        .map(|(_, name)| name.as_str())
        .unwrap_or("???")
}

const EN_JSON: &str = include_str!("../../locales/en.json");
const JA_JSON: &str = include_str!("../../locales/ja.json");
const ZH_CN_JSON: &str = include_str!("../../locales/zh-CN.json");
const ZH_TW_JSON: &str = include_str!("../../locales/zh-TW.json");
const KO_JSON: &str = include_str!("../../locales/ko.json");
const PT_JSON: &str = include_str!("../../locales/pt.json");
const FR_JSON: &str = include_str!("../../locales/fr.json");
const DE_JSON: &str = include_str!("../../locales/de.json");
const ES_JSON: &str = include_str!("../../locales/es.json");
const IT_JSON: &str = include_str!("../../locales/it.json");
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

pub fn set_language(lang: &str) {
    let mut current = write_guard(&CURRENT_LANGUAGE);
    *current = lang.to_string();
}

pub fn get_language() -> String {
    init_current_language();
    read_guard(&CURRENT_LANGUAGE).clone()
}

pub fn get() -> &'static I18nMessages {
    let lang = get_language();
    let dict = get_dictionary();
    let fallback = &dict[0].messages;
    dict.iter()
        .find(|entry| entry.lang == lang.as_str())
        .map(|entry| &entry.messages)
        .unwrap_or(fallback)
}

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
        assert_eq!(default_show_more(), "Show more...");
        assert_eq!(default_show_less(), "Show less...");
        assert_eq!(default_ui_contrast_offset(), "UI Contrast Offset");
        assert_eq!(default_duplicate(), "Duplicate...");
        assert_eq!(default_custom_section(), "Custom");
        assert_eq!(default_delete_custom(), "Delete Custom Theme");
    }

    #[test]
    fn test_i18n_default_color_values() {
        assert_eq!(default_highlight(), "Highlight");
    }

    #[test]
    fn test_get_fallback_to_en() {
        set_language("unsupported-lang-code");
        let msgs = get();
        assert!(!msgs.menu.file.is_empty());
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
        let msgs = super::get();
        assert!(!msgs.menu.export.is_empty());
        assert!(!msgs.menu.export_html.is_empty());
        assert!(!msgs.menu.export_pdf.is_empty());
        assert!(!msgs.menu.export_png.is_empty());
        assert!(!msgs.menu.export_jpg.is_empty());
    }

    #[test]
    fn test_update_messages_serde_defaults_backward_compat() {
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
        assert_eq!(default_ui_contrast_offset(), "UI Contrast Offset");
        assert_eq!(default_no_extension_label(), "No Extension");
        assert_eq!(default_no_extension_warning_title(), "Warning");
        assert_eq!(
            default_no_extension_warning(),
            "There is no guarantee that files without extensions can be displayed correctly as Markdown. Furthermore, the application may crash due to unexpected behavior. Are you sure you want to enable this?"
        );
        assert_eq!(default_section_system(), "System");
        assert_eq!(default_section_code(), "Code");
        assert_eq!(default_section_preview(), "Preview");
        assert_eq!(default_code_text(), "Code Text");
        assert_eq!(default_preview_text(), "Preview Text");
    }
}
#[cfg(test)]
mod tests_defaults {
    use super::*;

    #[test]
    fn test_i18n_settings_color_defaults() {
        assert_eq!(d_title_bar_text(), "Title Bar Text");
        assert_eq!(d_active_file_highlight(), "Active File");
        assert_eq!(d_success_text(), "Success Text");
        assert_eq!(d_warning_text(), "Warning Text");
        assert_eq!(d_error_text(), "Error Text");
        assert_eq!(d_button_bg(), "Button Background");
        assert_eq!(d_button_active(), "Active Button");
        assert_eq!(d_splash_bg(), "Splash Background");
        assert_eq!(d_splash_prog(), "Splash Progress");
        assert_eq!(d_line_num(), "Line Number");
        assert_eq!(d_line_num_act(), "Active Line Num");
        assert_eq!(d_curr_bg(), "Current Line");
        assert_eq!(d_hover_bg(), "Hover Line");
        assert_eq!(d_file_tree_text(), "File Tree Text");
        assert_eq!(default_clear_http_cache(), "Clear All Caches");
        assert_eq!(default_cache_retention_days(), "Cache Retention Days");
        assert_eq!(default_days_suffix(), " days");
    }
}

#[cfg(test)]
mod default_group_tests {
    use super::*;

    #[test]
    fn test_custom_theme_group_defaults() {
        assert_eq!(default_group_basic(), "Basic");
        assert_eq!(default_group_text(), "Text & Typography");
        assert_eq!(default_group_ui_elements(), "UI Elements");
        assert_eq!(crate::i18n::default_menu_release_notes(), "Release Notes");
    }
}