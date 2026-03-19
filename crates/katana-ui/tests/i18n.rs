use katana_ui::i18n::*;

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

/// This is the ultimate test for the new type-safe i18n structure.
/// It attempts to deserialize ALL language JSON files into the `I18nMessages` struct.
/// If ANY file is missing a key, has an extra key, or has mismatched types/hierarchy,
/// this test will fail at compile time or runtime, enforcing TDD structural integrity.
#[test]
fn all_locale_files_deserialize_to_strong_struct() {
    let locales = vec![
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

    for (lang, json_str) in locales {
        // If Serde cannot map the JSON to the struct fields exactly, this panics.
        // This completely eliminates the need for manual verification of i18n keys!
        let _messages: I18nMessages = serde_json::from_str(json_str)
            .unwrap_or_else(|e| panic!("Failed to deserialize {lang}.json structure: {e}"));
    }
}

/// Verify via static analysis that shell.rs does not hardcode UI strings without using i18n.
#[test]
fn shell_rs_has_no_i18n_leaks() {
    let source = include_str!("../src/shell.rs");
    let forbidden_patterns = [".on_hover_text(\"", "ui.heading(\""];

    for pattern in &forbidden_patterns {
        assert!(
            !source.contains(pattern),
            "Hardcoded UI strings detected in shell.rs: {pattern}\n\
             Please use i18n::get().something."
        );
    }
}

#[test]
fn supported_languages_includes_all_requested() {
    let langs = supported_languages();
    let codes: Vec<&str> = langs.iter().map(|(c, _)| c.as_str()).collect();

    // Original test
    assert!(codes.contains(&"en"));
    assert!(codes.contains(&"ja"));

    // Requested v0.1.3 languages
    assert!(codes.contains(&"zh-CN"));
    assert!(codes.contains(&"zh-TW"));
    assert!(codes.contains(&"ko"));
    assert!(codes.contains(&"pt"));
    assert!(codes.contains(&"fr"));
    assert!(codes.contains(&"de"));
    assert!(codes.contains(&"es"));
    assert!(codes.contains(&"it"));
}

#[test]
fn display_name_returns_known_codes() {
    assert_eq!(display_name("zz"), "???");
    assert_ne!(display_name("en"), "???");
    assert_ne!(display_name("ja"), "???");
}

#[test]
fn tf_function_correctly_substitutes_parameters() {
    // tf() no longer looks up a key (since it is strongly typed string).
    // It just replaces `{param_name}` in the generated string.
    let string_format = "Hello {name}, welcome to {place}!";
    let result = tf(string_format, &[("name", "world"), ("place", "Earth")]);
    assert_eq!(result, "Hello world, welcome to Earth!");
}
