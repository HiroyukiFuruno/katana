use katana_platform::theme::{ThemeMode, ThemePreset};

#[test]
fn test_katana_dark() {
    let dark = ThemePreset::KatanaDark.colors();
    assert_eq!(dark.mode, ThemeMode::Dark, "KatanaDark is not Dark?!");
}
