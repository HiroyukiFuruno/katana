use super::MigrationStrategy;
use serde_json::{json, Value};

/// Migrates settings from 0.1.3 to 0.1.4.
///
/// Changes:
/// 1. Converts `extra` from `HashMap<String, String>` to `Vec<ExtraSetting>`
pub struct Migration013To014;

impl MigrationStrategy for Migration013To014 {
    fn version(&self) -> &str {
        "0.1.3"
    }

    fn migrate(&self, mut json: Value) -> Value {
        if let Some(obj) = json.as_object_mut() {
            if let Some(extra_val) = obj.get("extra") {
                if let Some(extra_map) = extra_val.as_object() {
                    let mut new_extra = Vec::new();
                    for (k, v) in extra_map {
                        if let Some(v_str) = v.as_str() {
                            new_extra.push(json!({
                                "key": k,
                                "value": v_str,
                            }));
                        }
                    }
                    obj.insert("extra".to_string(), json!(new_extra));
                }
            }
            obj.insert("version".to_string(), json!("0.1.4"));
        }
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_migrate_from_0_1_3() {
        let strategy = Migration013To014;
        let old = json!({"version": "0.1.3", "extra": {"a": "A", "b": "B"}});
        let migrated = strategy.migrate(old);
        assert_eq!(migrated["version"], "0.1.4");
        let extra = migrated["extra"].as_array().unwrap();
        assert_eq!(extra.len(), 2);
    }
}
