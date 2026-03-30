pub mod v0_1_2;
pub mod v0_1_3_to_0_1_4;
pub mod v0_1_4_to_0_2_0;

use serde_json::Value;

// WHY: Strategy pattern for migrating settings JSON across versions.
pub trait MigrationStrategy: Send + Sync {
    // WHY: Returns the version string this strategy migrates FROM.
    fn version(&self) -> &str;

    // WHY: Migrates a JSON value from the old format to the new format.
    fn migrate(&self, json: Value) -> Value;
}

// WHY: Runs a sequence of migration strategies to bring a JSON object up to the target version.
pub struct MigrationRunner {
    strategies: Vec<Box<dyn MigrationStrategy>>,
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRunner {
    // WHY: Create a new MigrationRunner.
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    // WHY: Register a migration strategy.
    pub fn add_strategy(&mut self, strategy: Box<dyn MigrationStrategy>) {
        self.strategies.push(strategy);
    }

    // WHY: Migrates a JSON value incrementally until no more strategies match the current version.
    pub fn migrate(&self, mut json: Value) -> Value {
        loop {
            let current_version = json
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.1.2") // WHY: Default for unversioned files
                .to_string();

            let mut mapped = false;
            for strategy in &self.strategies {
                if strategy.version() == current_version {
                    tracing::info!("Migrating settings from version: {}", current_version);
                    json = strategy.migrate(json);
                    mapped = true;
                    // WHY: Restart loop to find the next strategy matching the new version.
                    break;
                }
            }
            if !mapped {
                break;
            }
        }
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct DummyStrategy {
        version: String,
    }

    impl MigrationStrategy for DummyStrategy {
        fn version(&self) -> &str {
            &self.version
        }

        fn migrate(&self, mut json: Value) -> Value {
            json.as_object_mut()
                .unwrap()
                .insert("version".to_string(), json!("next"));
            json
        }
    }

    #[test]
    fn test_migration_runner_default() {
        let runner = MigrationRunner::default();
        assert!(runner.strategies.is_empty());
    }

    #[test]
    fn test_migration_runner_loop_and_unmapped() {
        let mut runner = MigrationRunner::new();
        runner.add_strategy(Box::new(DummyStrategy {
            version: "0.1.2".to_string(),
        }));

        let initial_json = json!({"version": "0.1.2"});
        let migrated_json = runner.migrate(initial_json);
        assert_eq!(
            migrated_json.get("version").unwrap().as_str().unwrap(),
            "next"
        );
    }
}