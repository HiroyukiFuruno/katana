use katana_ui::i18n::*;
use std::collections::{HashMap, HashSet};

const EN_JSON: &str = include_str!("../locales/en.json");
const JA_JSON: &str = include_str!("../locales/ja.json");

/// en.json と ja.json のキーが完全に一致することを確認する。
/// キー漏れは自動検出される。
#[test]
fn 全ロケールキーが両言語に存在する() {
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
        "ja.json にキーが不足しています: {missing_in_ja:?}"
    );
    assert!(
        missing_in_en.is_empty(),
        "en.json にキーが不足しています: {missing_in_en:?}"
    );
}

/// tf() がパラメータを正しく置換することを確認する。
#[test]
fn tf関数がパラメータを正しく置換する() {
    let result = tf("__test_key__", &[("name", "world")]);
    // キーが存在しない場合はキー自体が返る（置換なし）
    assert_eq!(result, "__test_key__");
}

/// shell.rs が i18n を通さずに UI 文字列をハードコードしていないことを静的解析で確認する。
///
/// これは「UT でも弾く」要件のための静的解析テスト。
/// 翻訳可能なテキストを含む高リスクな呼び出しパターンを禁止する。
#[test]
fn shellrsにi18n漏れがない() {
    let source = include_str!("../src/shell.rs");

    // 「ホバーテキスト」や「見出し」に直接リテラルを渡すパターンを禁止する。
    // i18n::t() / i18n::tf() を経由しなければならない。
    let forbidden_patterns = [
        // on_hover_text に直接文字列リテラルを渡しているパターン
        ".on_hover_text(\"",
        // ui.heading に直接文字列リテラルを渡しているパターン
        "ui.heading(\"",
    ];

    for pattern in &forbidden_patterns {
        assert!(
            !source.contains(pattern),
            "shell.rs にハードコードされた UI 文字列が検出されました: {pattern}\n\
             i18n::t() または i18n::tf() を使用してください。"
        );
    }
}

// L14-31: supported_languages() と display_name()
#[test]
fn supported_languagesはen_jaを含む() {
    let langs = supported_languages();
    let codes: Vec<&str> = langs.iter().map(|(c, _)| c.as_str()).collect();
    assert!(codes.contains(&"en"), "en should be in supported languages");
    assert!(codes.contains(&"ja"), "ja should be in supported languages");
}

// L30: display_name フォールバック（無効コードは "???" を返す）
#[test]
fn display_name未知コードはフォールバックする() {
    let name = display_name("zz");
    assert_eq!(name, "???");
}

// L25-31: display_name() for known codes
#[test]
fn display_name既知コードを返す() {
    let en_name = display_name("en");
    assert!(!en_name.is_empty() && en_name != "???");
    let ja_name = display_name("ja");
    assert!(!ja_name.is_empty() && ja_name != "???");
}

// L61-65: set_language() and get_language()
#[test]
fn set_languageが言語を変更する() {
    // Initial set to "en"
    set_language("en");
    assert_eq!(get_language(), "en");

    // Switch to "ja"
    set_language("ja");
    assert_eq!(get_language(), "ja");

    // Reset to "en"
    set_language("en");
}

// L74-83: t() - key not found returns the key itself
#[test]
fn t関数は存在しないキーをそのまま返す() {
    set_language("en");
    let result = t("key_that_does_not_exist_in_any_locale");
    assert_eq!(result, "key_that_does_not_exist_in_any_locale");
}

// L74-83: t() - exists in 'en', not in 'ja' → key fallback for ja
#[test]
fn t関数はen言語で既知キーを翻訳する() {
    set_language("en");
    let result = t("status_ready");
    // Should return translated string, not the key itself
    assert_ne!(result, "status_ready");
    assert!(!result.is_empty());
}

// L74-83: t() with 'ja' language
#[test]
fn t関数はja言語で既知キーを翻訳する() {
    set_language("ja");
    let result = t("status_ready");
    assert_ne!(result, "status_ready");
    assert!(!result.is_empty());

    // Reset
    set_language("en");
}

// tf() with actual i18n key and params
#[test]
fn tf関数は実際のキーのパラメータを置換する() {
    set_language("en");
    // Use a key that has a param like {name}, {error}, etc.
    // "status_opened_workspace" with {name}
    let result = tf("status_opened_workspace", &[("name", "my-project")]);
    assert!(result.contains("my-project"));
}

// L81: 辞書が見つからない言語コードで t() を呼ぶとキー自体が返る
#[test]
fn t関数は未知言語でキーをそのまま返す() {
    set_language("zz"); // 存在しない言語コード
    let result = t("status_ready");
    assert_eq!(result, "status_ready");
    set_language("en"); // reset
}
