use super::MigrationStrategy;
use serde_json::{json, Value};

/* WHY: Migrates settings from 0.1.4 to 0.2.0.

Changes:
1. Nests `last_workspace` and `workspace_paths` under `workspace`.
2. Initializes `open_tabs` and `active_tab_idx`.
3. Updates version to 0.2.0. */
pub struct Migration014To020;

impl MigrationStrategy for Migration014To020 {
    fn version(&self) -> &str {
        "0.1.4"
    }

    fn migrate(&self, mut json: Value) -> Value {
        if let Some(obj) = json.as_object_mut() {
            let last_workspace = obj.remove("last_workspace").unwrap_or(Value::Null);
            let workspace_paths = obj.remove("workspace_paths").unwrap_or(json!([]));

            obj.insert(
                "workspace".to_string(),
                json!({
                    "last_workspace": last_workspace,
                    "paths": workspace_paths,
                    "open_tabs": [],
                    "active_tab_idx": Value::Null,
                }),
            );

            obj.insert("version".to_string(), json!("0.2.0"));
        }
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_migrate_from_0_1_4() {
        let strategy = Migration014To020;
        let old = json!({
            "version": "0.1.4",
            "last_workspace": "/path/to/ws",
            "workspace_paths": ["/path/to/ws"]
        });
        let migrated = strategy.migrate(old);
        assert_eq!(migrated["version"], "0.2.0");
        assert_eq!(migrated["workspace"]["last_workspace"], "/path/to/ws");
        assert_eq!(migrated["workspace"]["paths"][0], "/path/to/ws");
        assert!(migrated.get("last_workspace").is_none());
    }
}
