use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::mermaid_renderer::{is_mmdc_available, render_mermaid};

#[test]
fn mmdc未検出時はエラー結果を返す() {
    // 存在しないバイナリを指定してフォールバックを検証する。
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: "graph TD; A-->B".to_string(),
    };
    let result = render_mermaid(&block);
    assert!(matches!(result, DiagramResult::CommandNotFound { .. }));
    // テスト後に環境変数を戻す。
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

// mmdc がシステムで利用可能な場合のみ実行する結合テスト。
#[test]
fn mmdcが利用可能なら正しくsvgを返す() {
    // MERMAID_MMDC が nonexistent になっている場合はスキップ。
    if std::env::var("MERMAID_MMDC").as_deref() == Ok("/nonexistent/mmdc") {
        return;
    }
    if !is_mmdc_available() {
        return; // mmdc が未インストールならスキップ。
    }
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: "graph TD; A-->B".to_string(),
    };
    let result = render_mermaid(&block);
    assert!(matches!(result, DiagramResult::OkPng(_)));
}
