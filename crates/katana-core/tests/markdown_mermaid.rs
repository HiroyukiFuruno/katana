use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::mermaid_renderer;

#[test]
fn mmdc未検出時はcommandnotfoundを返す() {
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
fn resolve_mmdc_binary_env_var設定時はそのパスを返す() {
    unsafe { std::env::set_var("MERMAID_MMDC", "/custom/mmdc") };
    let path = mermaid_renderer::resolve_mmdc_binary();
    assert_eq!(path, std::path::PathBuf::from("/custom/mmdc"));
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

#[test]
fn resolve_mmdc_binary_env_var未設定時はシステムパスを探す() {
    unsafe { std::env::remove_var("MERMAID_MMDC") };
    let path = mermaid_renderer::resolve_mmdc_binary();
    // mmdc が見つからなくてもフォールバックとして "mmdc" が返る
    assert!(!path.as_os_str().is_empty());
}

#[test]
fn create_input_file_は一時ファイルを作成する() {
    let file = mermaid_renderer::create_input_file("graph TD; A-->B").unwrap();
    let path = file.path().to_path_buf();
    assert!(path.exists());
    assert!(path.to_string_lossy().ends_with(".mmd"));
}

#[test]
fn is_mmdc_available_偽バイナリはfalse() {
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    assert!(!mermaid_renderer::is_mmdc_available());
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

// mmdc がシステムで利用可能な場合のみ実行する結合テスト。
#[test]
fn mmdcが利用可能なら正しくpngを返す() {
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

// mmdc が見つからない場合の run_mmdc_process エラーパス
#[test]
fn run_mmdc_process_mmdc不在時はエラー() {
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    let result = mermaid_renderer::run_mmdc_process("graph TD; A-->B");
    assert!(result.is_err());
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}
