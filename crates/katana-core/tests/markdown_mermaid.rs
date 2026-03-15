use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::mermaid_renderer;

#[test]
fn returns_commandnotfound_when_mmdc_not_found() {
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: "graph TD; A-->B".to_string(),
    };
    let result = mermaid_renderer::render_mermaid(&block);
    assert!(matches!(result, DiagramResult::CommandNotFound { .. }));
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

#[test]
fn returns_its_path_when_resolve_mmdc_binary_env_var_is_set() {
    unsafe { std::env::set_var("MERMAID_MMDC", "/custom/mmdc") };
    let path = mermaid_renderer::resolve_mmdc_binary();
    assert_eq!(path, std::path::PathBuf::from("/custom/mmdc"));
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

#[test]
fn searches_system_path_when_resolve_mmdc_binary_env_var_is_not_set() {
    unsafe { std::env::remove_var("MERMAID_MMDC") };
    let path = mermaid_renderer::resolve_mmdc_binary();
    // Even if mmdc is not found, "mmdc" is returned as fallback
    assert!(!path.as_os_str().is_empty());
}

#[test]
fn create_input_file_creates_a_temporary_file() {
    let file = mermaid_renderer::create_input_file("graph TD; A-->B").unwrap();
    let path = file.path().to_path_buf();
    assert!(path.exists());
    assert!(path.to_string_lossy().ends_with(".mmd"));
}

#[test]
fn fake_binary_is_false_in_is_mmdc_available() {
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    assert!(!mermaid_renderer::is_mmdc_available());
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

// Integration test that runs only when mmdc is available on the system.
#[test]
fn returns_png_correctly_if_mmdc_is_available() {
    if std::env::var("MERMAID_MMDC").as_deref() == Ok("/nonexistent/mmdc") {
        return;
    }
    if !mermaid_renderer::is_mmdc_available() {
        return;
    }
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: "graph TD; A-->B".to_string(),
    };
    let result = mermaid_renderer::render_mermaid(&block);
    assert!(matches!(result, DiagramResult::OkPng(_)));
}

// run_mmdc_process error path when mmdc is not found
#[test]
fn run_mmdc_process_errors_when_mmdc_is_absent() {
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    let result = mermaid_renderer::run_mmdc_process("graph TD; A-->B");
    assert!(result.is_err());
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}
