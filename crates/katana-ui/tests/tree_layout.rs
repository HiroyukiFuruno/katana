#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;
    use egui_kittest::Harness;
    use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
    use katana_ui::app_state::{AppAction, AppState};
    use katana_ui::shell::KatanaApp;

    fn setup_harness() -> Harness<'static, KatanaApp> {
        let settings_path =
            std::env::temp_dir().join(format!("katana_test_layout_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&settings_path);

        Harness::builder().build_eframe(move |_cc| {
            let ai_registry = AiProviderRegistry::new();
            let plugin_registry = PluginRegistry::new();
            let mut state = AppState::new(
                ai_registry,
                plugin_registry,
                katana_platform::SettingsService::new(Box::new(
                    katana_platform::JsonFileRepository::new(settings_path.clone()),
                )),
                std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
            );
            // Pre-accept terms for testing
            state.config.settings.settings_mut().terms_accepted_version =
                Some(katana_ui::about_info::APP_VERSION.to_string());
            katana_ui::i18n::set_language("en");
            let mut app = KatanaApp::new(state);
            app.skip_splash();
            app
        })
    }

    #[test]
    fn test_tree_entry_alignment_regression() {
        let mut harness = setup_harness();

        // Create a temporary directory and file at level 0
        let temp_dir = std::env::temp_dir().join("katana_layout_repro");
        let temp_dir = temp_dir.canonicalize().unwrap_or(temp_dir);
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let sub_dir = temp_dir.join("aa_dir"); // Level 0
        std::fs::create_dir_all(&sub_dir).unwrap();
        std::fs::write(sub_dir.join("child.md"), "test").unwrap();

        let root_file = temp_dir.join("zz_file.md"); // Level 0
        std::fs::write(&root_file, "# Root").unwrap();

        // Open workspace
        harness
            .state_mut()
            .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));

        for _ in 0..100 {
            harness.step();
            if !harness.state_mut().app_state_mut().workspace.is_loading
                && harness.state_mut().app_state_mut().workspace.data.is_some()
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        harness.step();

        // Verify nodes exist
        // For aa_dir, find the exact inner Label node (which has value="aa_dir") to get its X bounds.
        let all_labels: Vec<_> = harness
            .get_all_by_role(egui::accesskit::Role::Label)
            .collect();
        let dir_node = all_labels
            .iter()
            .find(|n| n.value().as_deref() == Some("aa_dir"))
            .expect("should find aa_dir label");

        // zz_file.md is a SelectableLabel which is typically a single Button node, so this matches perfectly.
        let file_node = harness.get_by_label_contains("zz_file.md");

        let dir_rect = dir_node.rect();
        let file_rect = file_node.rect();

        println!("Dir rect: {:?}", dir_rect);
        println!("File rect: {:?}", file_rect);

        // REGRESSION: dir_rect.min.x and file_rect.min.x should be almost identical for level 0.
        // In the broken implementation, dir starts much further to the right.
        let diff = (dir_rect.min.x - file_rect.min.x).abs();
        assert!(
            diff <= 2.0,
            "Alignment mismatch at level 0: dir.x={}, file.x={}, diff={}",
            dir_rect.min.x,
            file_rect.min.x,
            diff
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
