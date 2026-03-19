use super::migration::MigrationStrategy;
use serde_json::{json, Value};

/// Migrates settings from v0.1.2 (flat structure) to v0.1.3 (hierarchical).
pub struct Migration0_1_2;

impl MigrationStrategy for Migration0_1_2 {
    fn version(&self) -> &str {
        "0.1.2"
    }

    fn migrate(&self, json: Value) -> Value {
        if let serde_json::Value::Object(mut map) = json {
            let mut theme_map = serde_json::Map::new();
            if let Some(theme) = map.remove("theme") {
                theme_map.insert("theme".to_string(), theme);
            } else {
                theme_map.insert("theme".to_string(), json!("dark"));
            }
            if let Some(preset) = map.remove("selected_preset") {
                theme_map.insert("preset".to_string(), preset);
            }
            if let Some(custom) = map.remove("custom_color_overrides") {
                theme_map.insert("custom_color_overrides".to_string(), custom);
            }

            let mut font_map = serde_json::Map::new();
            if let Some(size) = map.remove("font_size") {
                font_map.insert("size".to_string(), size);
            } else {
                font_map.insert("size".to_string(), json!(14.0));
            }
            if let Some(family) = map.remove("font_family") {
                font_map.insert("family".to_string(), family);
            } else {
                font_map.insert("family".to_string(), json!("monospace"));
            }

            let mut layout_map = serde_json::Map::new();
            if let Some(split) = map.remove("split_direction") {
                layout_map.insert("split_direction".to_string(), split);
            } else {
                layout_map.insert("split_direction".to_string(), json!("Horizontal"));
            }
            if let Some(order) = map.remove("pane_order") {
                layout_map.insert("pane_order".to_string(), order);
            } else {
                layout_map.insert("pane_order".to_string(), json!("EditorFirst"));
            }

            // `toc_visible` and `last_workspace` remain at the top level;
            // only theme/font/layout are grouped into nested objects.
            map.insert("version".to_string(), json!("0.1.3"));
            map.insert("theme".to_string(), serde_json::Value::Object(theme_map));
            map.insert("font".to_string(), serde_json::Value::Object(font_map));
            map.insert("layout".to_string(), serde_json::Value::Object(layout_map));

            serde_json::Value::Object(map)
        } else {
            json
        }
    }
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
