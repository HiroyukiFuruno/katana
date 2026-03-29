use super::MigrationStrategy;
use serde_json::{json, Value};

// WHY: Migrates settings from v0.1.2 (flat structure) to v0.1.3 (hierarchical).
pub struct Migration0_1_2;

impl MigrationStrategy for Migration0_1_2 {
    fn version(&self) -> &str {
        "0.1.2"
    }

    fn migrate(&self, json: Value) -> Value {
        let serde_json::Value::Object(mut map) = json else {
            return json;
        };

        map.insert("version".into(), json!("0.1.3"));

        let theme_val = serde_json::Value::Object(migrate_theme(&mut map));
        map.insert("theme".into(), theme_val);

        let font_val = serde_json::Value::Object(migrate_font(&mut map));
        map.insert("font".into(), font_val);

        let layout_val = serde_json::Value::Object(migrate_layout(&mut map));
        map.insert("layout".into(), layout_val);

        serde_json::Value::Object(map)
    }
}

fn migrate_theme(map: &mut serde_json::Map<String, Value>) -> serde_json::Map<String, Value> {
    let mut theme_map = serde_json::Map::new();
    theme_map.insert("theme".into(), map.remove("theme").unwrap_or(json!("dark")));
    if let Some(preset) = map.remove("selected_preset") {
        theme_map.insert("preset".into(), preset);
    }
    if let Some(custom) = map.remove("custom_color_overrides") {
        theme_map.insert("custom_color_overrides".into(), custom);
    }
    theme_map
}

fn migrate_font(map: &mut serde_json::Map<String, Value>) -> serde_json::Map<String, Value> {
    let mut font_map = serde_json::Map::new();
    font_map.insert(
        "size".into(),
        map.remove("font_size").unwrap_or(json!(14.0)),
    );
    font_map.insert(
        "family".into(),
        map.remove("font_family").unwrap_or(json!("monospace")),
    );
    font_map
}

fn migrate_layout(map: &mut serde_json::Map<String, Value>) -> serde_json::Map<String, Value> {
    let mut layout_map = serde_json::Map::new();
    layout_map.insert(
        "split_direction".into(),
        map.remove("split_direction").unwrap_or(json!("Horizontal")),
    );
    layout_map.insert(
        "pane_order".into(),
        map.remove("pane_order").unwrap_or(json!("EditorFirst")),
    );
    layout_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_0_1_2_to_0_1_3_all_fields_missing() {
        let strategy = Migration0_1_2;
        assert_eq!(strategy.version(), "0.1.2");
        let old_json = json!({});
        let new_json = strategy.migrate(old_json);
        assert_eq!(new_json.get("version").unwrap().as_str().unwrap(), "0.1.3");
        assert_eq!(
            new_json
                .get("theme")
                .unwrap()
                .get("theme")
                .unwrap()
                .as_str()
                .unwrap(),
            "dark"
        );
        assert_eq!(
            new_json
                .get("font")
                .unwrap()
                .get("family")
                .unwrap()
                .as_str()
                .unwrap(),
            "monospace"
        );
        assert_eq!(
            new_json
                .get("layout")
                .unwrap()
                .get("split_direction")
                .unwrap()
                .as_str()
                .unwrap(),
            "Horizontal"
        );
    }

    #[test]
    fn test_migration_0_1_2_to_0_1_3_all_fields_present() {
        let strategy = Migration0_1_2;
        let old_json = json!({
            "theme": "light",
            "selected_preset": "Nord",
            "custom_color_overrides": { "background": { "r": 0, "g": 0, "b": 0 } },
            "font_size": 16.0,
            "font_family": "Arial",
            "split_direction": "Vertical",
            "pane_order": "PreviewFirst"
        });
        let new_json = strategy.migrate(old_json);
        assert_eq!(
            new_json
                .get("theme")
                .unwrap()
                .get("theme")
                .unwrap()
                .as_str()
                .unwrap(),
            "light"
        );
        assert_eq!(
            new_json
                .get("theme")
                .unwrap()
                .get("preset")
                .unwrap()
                .as_str()
                .unwrap(),
            "Nord"
        );
        assert!(new_json
            .get("theme")
            .unwrap()
            .get("custom_color_overrides")
            .is_some());
        assert_eq!(
            new_json
                .get("font")
                .unwrap()
                .get("size")
                .unwrap()
                .as_f64()
                .unwrap(),
            16.0
        );
        assert_eq!(
            new_json
                .get("font")
                .unwrap()
                .get("family")
                .unwrap()
                .as_str()
                .unwrap(),
            "Arial"
        );
        assert_eq!(
            new_json
                .get("layout")
                .unwrap()
                .get("split_direction")
                .unwrap()
                .as_str()
                .unwrap(),
            "Vertical"
        );
        assert_eq!(
            new_json
                .get("layout")
                .unwrap()
                .get("pane_order")
                .unwrap()
                .as_str()
                .unwrap(),
            "PreviewFirst"
        );
    }

    #[test]
    fn test_migration_0_1_2_not_object() {
        let strategy = Migration0_1_2;
        let old_json = json!("not an object");
        let new_json = strategy.migrate(old_json.clone());
        assert_eq!(new_json, old_json);
    }
}
