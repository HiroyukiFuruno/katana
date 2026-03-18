use katana_platform::{AppSettings, SettingsService};

// ── Font Size Range ──

#[test]
fn font_size_default_is_14() {
    let settings = AppSettings::default();
    assert!((settings.font_size - 14.0).abs() < f32::EPSILON);
}

#[test]
fn font_size_clamp_enforces_minimum_8() {
    let mut settings = AppSettings::default();
    settings.set_font_size(5.0);
    assert!((settings.font_size - 8.0).abs() < f32::EPSILON);
}

#[test]
fn font_size_clamp_enforces_maximum_32() {
    let mut settings = AppSettings::default();
    settings.set_font_size(50.0);
    assert!((settings.font_size - 32.0).abs() < f32::EPSILON);
}

#[test]
fn font_size_within_range_is_accepted() {
    let mut settings = AppSettings::default();
    settings.set_font_size(20.0);
    assert!((settings.font_size - 20.0).abs() < f32::EPSILON);
}

#[test]
fn font_size_at_boundary_8_is_accepted() {
    let mut settings = AppSettings::default();
    settings.set_font_size(8.0);
    assert!((settings.font_size - 8.0).abs() < f32::EPSILON);
}

#[test]
fn font_size_at_boundary_32_is_accepted() {
    let mut settings = AppSettings::default();
    settings.set_font_size(32.0);
    assert!((settings.font_size - 32.0).abs() < f32::EPSILON);
}

// ── Font Family ──

#[test]
fn font_family_default_is_monospace() {
    let settings = AppSettings::default();
    assert_eq!(settings.font_family, "monospace");
}

#[test]
fn font_family_can_be_changed() {
    let mut settings = AppSettings::default();
    settings.font_family = "sans-serif".to_string();
    assert_eq!(settings.font_family, "sans-serif");
}

// ── Persistence ──

#[test]
fn font_size_roundtrip_via_json_repository() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("settings.json");

    let repo = katana_platform::JsonFileRepository::new(path.clone());
    let mut svc = SettingsService::new(Box::new(repo));
    svc.settings_mut().set_font_size(18.0);
    svc.save().unwrap();

    let repo2 = katana_platform::JsonFileRepository::new(path);
    let svc2 = SettingsService::new(Box::new(repo2));
    assert!((svc2.settings().font_size - 18.0).abs() < f32::EPSILON);
}

#[test]
fn font_family_roundtrip_via_json_repository() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("settings.json");

    let repo = katana_platform::JsonFileRepository::new(path.clone());
    let mut svc = SettingsService::new(Box::new(repo));
    svc.settings_mut().font_family = "sans-serif".to_string();
    svc.save().unwrap();

    let repo2 = katana_platform::JsonFileRepository::new(path);
    let svc2 = SettingsService::new(Box::new(repo2));
    assert_eq!(svc2.settings().font_family, "sans-serif");
}

#[test]
fn font_size_deserialization_clamps_out_of_range_value() {
    let json = r#"{"font_size": 100.0}"#;
    let settings: AppSettings = serde_json::from_str(json).unwrap();
    // clamped_font_size() should clamp out-of-range values after deserialization
    assert!((settings.clamped_font_size() - 32.0).abs() < f32::EPSILON);
}

#[test]
fn font_size_deserialization_clamps_below_minimum() {
    let json = r#"{"font_size": 2.0}"#;
    let settings: AppSettings = serde_json::from_str(json).unwrap();
    assert!((settings.clamped_font_size() - 8.0).abs() < f32::EPSILON);
}
