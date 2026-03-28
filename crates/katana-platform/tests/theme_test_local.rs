use katana_platform::theme::{ThemeMode, ThemePreset};

#[test]
fn test_katana_light() {
    let light = ThemePreset::KatanaLight.colors();
    assert_eq!(light.mode, ThemeMode::Light);
    assert_eq!(light.system.background.r, 255);
}
