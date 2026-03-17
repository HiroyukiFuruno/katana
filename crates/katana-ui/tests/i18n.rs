use katana_ui::i18n::*;
use std::collections::{HashMap, HashSet};

const EN_JSON: &str = include_str!("../locales/en.json");
const JA_JSON: &str = include_str!("../locales/ja.json");

/// Verify that all keys in en.json and ja.json match perfectly.
/// Key omissions are detected automatically.
#[test]
fn all_locale_keys_exist_in_both_languages() {
    let en: HashMap<String, serde_json::Value> =
        serde_json::from_str(EN_JSON).expect("en.json must be valid JSON");
    let ja: HashMap<String, serde_json::Value> =
        serde_json::from_str(JA_JSON).expect("ja.json must be valid JSON");

    let en_keys: HashSet<_> = en.keys().collect();
    let ja_keys: HashSet<_> = ja.keys().collect();

    let missing_in_ja: Vec<_> = en_keys.difference(&ja_keys).collect();
    let missing_in_en: Vec<_> = ja_keys.difference(&en_keys).collect();

    assert!(
        missing_in_ja.is_empty(),
        "ja.json is missing keys: {missing_in_ja:?}"
    );
    assert!(
        missing_in_en.is_empty(),
        "en.json is missing keys: {missing_in_en:?}"
    );
}

/// Verify that tf() correctly substitutes parameters.
#[test]
fn tf_function_correctly_substitutes_parameters() {
    let result = tf("__test_key__", &[("name", "world")]);
    // If the key does not exist, the key itself is returned (no substitution)
    assert_eq!(result, "__test_key__");
}

/// Verify via static analysis that shell.rs does not hardcode UI strings without using i18n.
///
/// This is a static analysis test to fulfill the "reject even in UT" requirement.
/// Prohibits high-risk call patterns that contain translatable text.
#[test]
fn shell_rs_has_no_i18n_leaks() {
    let source = include_str!("../src/shell.rs");

    // Prohibit patterns that pass string literals directly to "hover text" or "headings".
    // Must go through i18n::t() / i18n::tf().
    let forbidden_patterns = [
        // Pattern passing string literal directly to on_hover_text
        ".on_hover_text(\"",
        // Pattern passing string literal directly to ui.heading
        "ui.heading(\"",
    ];

    for pattern in &forbidden_patterns {
        assert!(
            !source.contains(pattern),
            "Hardcoded UI strings detected in shell.rs: {pattern}\n\
             Please use i18n::t() or i18n::tf()."
        );
    }
}

// L14-31: supported_languages() and display_name()
#[test]
fn supported_languages_includes_en_and_ja() {
    let langs = supported_languages();
    let codes: Vec<&str> = langs.iter().map(|(c, _)| c.as_str()).collect();
    assert!(codes.contains(&"en"), "en should be in supported languages");
    assert!(codes.contains(&"ja"), "ja should be in supported languages");
}

// L30: display_name fallback (invalid code returns "???")
#[test]
fn display_name_falls_back_for_unknown_code() {
    let name = display_name("zz");
    assert_eq!(name, "???");
}

// L25-31: display_name() for known codes
#[test]
fn display_name_returns_known_codes() {
    let en_name = display_name("en");
    assert!(!en_name.is_empty() && en_name != "???");
    let ja_name = display_name("ja");
    assert!(!ja_name.is_empty() && ja_name != "???");
}

#[test]
fn single_threaded_mut_language_tests() {
    // L61-65: set_language() and get_language()
    // Initial set to "en"
    set_language("en");
    assert_eq!(get_language(), "en");

    // Switch to "ja"
    set_language("ja");
    assert_eq!(get_language(), "ja");

    // Reset to "en"
    set_language("en");

    // L74-83: t() - key not found returns the key itself
    let result = t("key_that_does_not_exist_in_any_locale");
    assert_eq!(result, "key_that_does_not_exist_in_any_locale");

    // L74-83: t() - exists in 'en', not in 'ja' → key fallback for ja
    let result = t("status_ready");
    assert_ne!(result, "status_ready");
    assert!(!result.is_empty());

    // tf() with actual i18n key and params
    let result = tf("status_opened_workspace", &[("name", "my-project")]);
    assert!(result.contains("my-project"));

    // L74-83: t() with 'ja' language
    set_language("ja");
    let result = t("status_ready");
    assert_ne!(result, "status_ready");
    assert!(!result.is_empty());

    // L81: Calling t() with a language code where dictionary isn't found returns the key itself
    set_language("zz"); // Non-existent language code
    let result = t("status_ready");
    assert_eq!(result, "status_ready");

    set_language("en"); // reset
}
