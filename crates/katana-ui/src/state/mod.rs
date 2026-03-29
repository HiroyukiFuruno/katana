pub mod config;
pub mod document;
pub mod layout;
pub mod scroll;
pub mod search;
pub mod update;
pub mod workspace;

pub use config::*;
pub use document::*;
pub use layout::*;
pub use scroll::*;
pub use search::*;
pub use update::*;
pub use workspace::*;

#[cfg(test)]
pub mod app_state_tests {
    use crate::app_state::*;
    use katana_core::document::Document;
    use katana_platform::{PaneOrder, SplitDirection};
    use std::path::PathBuf;

    #[test]
    fn test_settings_section_and_tabs_mapping() {
        assert_eq!(SettingsTab::Theme.section(), SettingsSection::Appearance);
        assert_eq!(SettingsTab::Workspace.section(), SettingsSection::Behavior);
        assert_eq!(
            SettingsSection::Appearance.tabs(),
            &[SettingsTab::Theme, SettingsTab::Font, SettingsTab::Layout]
        );
    }

    fn make_state_with_doc(path: &str) -> AppState {
        let mut state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        let doc = Document {
            path: PathBuf::from(path),
            buffer: String::new(),
            is_dirty: false,
            is_loaded: true,
            is_pinned: false,
        };
        state.document.open_documents.push(doc);
        state.document.active_doc_idx = Some(0);
        state.initialize_tab_split_state(PathBuf::from(path));
        state.set_active_view_mode(ViewMode::Split);
        state
    }

    #[test]
    fn test_split_state_is_cached_per_tab_after_settings_change() {
        let mut state = make_state_with_doc("/tmp/a.md");
        state.config.settings.settings_mut().layout.split_direction = SplitDirection::Horizontal;
        state.initialize_tab_split_state("/tmp/a.md");
        state.config.settings.settings_mut().layout.split_direction = SplitDirection::Vertical;
        assert_eq!(state.active_split_direction(), SplitDirection::Horizontal);
    }

    #[test]
    fn test_pane_order_is_cached_per_tab_after_settings_change() {
        let mut state = make_state_with_doc("/tmp/b.md");
        state.config.settings.settings_mut().layout.pane_order = PaneOrder::EditorFirst;
        state.initialize_tab_split_state("/tmp/b.md");
        state.config.settings.settings_mut().layout.pane_order = PaneOrder::PreviewFirst;
        assert_eq!(state.active_pane_order(), PaneOrder::EditorFirst);
    }

    #[test]
    fn test_new_tab_uses_latest_persisted_split_defaults() {
        let mut state = make_state_with_doc("/tmp/a.md");
        assert_eq!(state.active_split_direction(), SplitDirection::Horizontal);

        state.config.settings.settings_mut().layout.split_direction = SplitDirection::Vertical;
        state.config.settings.settings_mut().layout.pane_order = PaneOrder::PreviewFirst;

        let doc = Document {
            path: PathBuf::from("/tmp/b.md"),
            buffer: String::new(),
            is_dirty: false,
            is_loaded: true,
            is_pinned: false,
        };
        state.document.open_documents.push(doc);
        state.document.active_doc_idx = Some(1);
        state.initialize_tab_split_state("/tmp/b.md");

        assert_eq!(state.active_split_direction(), SplitDirection::Vertical);
        assert_eq!(state.active_pane_order(), PaneOrder::PreviewFirst);
    }

    #[test]
    fn test_ensure_active_split_state() {
        let mut state = make_state_with_doc("/tmp/c.md");
        state.document.tab_split_states.clear();
        state.ensure_active_split_state();
        assert_eq!(state.document.tab_split_states.len(), 1);

        let mut empty_state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        empty_state.ensure_active_split_state();
        assert_eq!(empty_state.document.tab_split_states.len(), 0);
    }
}
