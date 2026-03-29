use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;
use katana_core::Document;
use katana_ui::app_state::*;

#[test]
fn new_app_state_is_empty_and_default_view() {
    let state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    assert!(state.workspace.data.is_none());
    assert!(state.document.open_documents.is_empty());
    assert_eq!(state.document.active_doc_idx, None);
    assert_eq!(state.active_view_mode(), ViewMode::PreviewOnly);
}

#[test]
fn active_document_returns_correct_doc_when_set() {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let doc1 = Document::new("doc1.md", "Doc1");
    let doc2 = Document::new("doc2.md", "Doc2");

    state.document.open_documents.push(doc1);
    state.document.open_documents.push(doc2);

    state.document.active_doc_idx = Some(1);
    assert_eq!(state.active_document().unwrap().buffer, "Doc2");

    state.document.active_doc_idx = Some(0);
    assert_eq!(state.active_document().unwrap().buffer, "Doc1");

    state.document.active_doc_idx = Some(999);
    assert!(state.active_document().is_none());
}

#[test]
fn is_dirty_reflects_active_document_state() {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    assert!(!state.is_dirty()); // no document

    let mut doc = Document::new("test.md", "test");
    doc.is_dirty = true;
    state.document.open_documents.push(doc);
    state.document.active_doc_idx = Some(0);

    assert!(state.is_dirty());
}

#[test]
fn active_document_mut_returns_correct_mut_doc() {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let doc1 = Document::new("doc1.md", "Doc1");
    state.document.open_documents.push(doc1);
    state.document.active_doc_idx = Some(0);

    if let Some(mut_doc) = state.active_document_mut() {
        mut_doc.buffer = "Updated".to_string();
    }

    assert_eq!(state.active_document().unwrap().buffer, "Updated");
}

#[test]
fn active_path_returns_path_of_active_document() {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let doc1 = Document::new("doc1.md", "Doc1");
    state.document.open_documents.push(doc1);
    state.document.active_doc_idx = Some(0);

    assert_eq!(
        state.active_path(),
        Some(std::path::PathBuf::from("doc1.md"))
    );

    state.document.active_doc_idx = None;
    assert_eq!(state.active_path(), None);
}
