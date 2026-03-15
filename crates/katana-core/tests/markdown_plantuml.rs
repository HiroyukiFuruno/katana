use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::plantuml_renderer;
use std::path::PathBuf;

#[test]
fn jar未検出時はnotinstalledを返す() {
    unsafe { std::env::set_var("PLANTUML_JAR", "/nonexistent/plantuml.jar") };
    let block = DiagramBlock {
        kind: DiagramKind::PlantUml,
        source: "@startuml\nA -> B\n@enduml".to_string(),
    };
    let result = plantuml_renderer::render_plantuml(&block);
    assert!(matches!(result, DiagramResult::NotInstalled { .. }));
    unsafe { std::env::remove_var("PLANTUML_JAR") };
}

#[test]
fn jar_candidate_paths_env_var設定時は環境変数パスのみ返す() {
    unsafe { std::env::set_var("PLANTUML_JAR", "/custom/path/plantuml.jar") };
    let paths = plantuml_renderer::jar_candidate_paths();
    assert_eq!(paths, vec![PathBuf::from("/custom/path/plantuml.jar")]);
    unsafe { std::env::remove_var("PLANTUML_JAR") };
}

#[test]
fn jar_candidate_paths_env_var未設定時は複数候補を返す() {
    unsafe { std::env::remove_var("PLANTUML_JAR") };
    let paths = plantuml_renderer::jar_candidate_paths();
    // Homebrewパス + バイナリ隣接 + XDG候補が含まれる
    assert!(paths.len() >= 2);
}

#[test]
fn find_plantuml_jar_存在しない場合はnone() {
    unsafe { std::env::set_var("PLANTUML_JAR", "/nonexistent/never/plantuml.jar") };
    let result = plantuml_renderer::find_plantuml_jar();
    assert!(result.is_none());
    unsafe { std::env::remove_var("PLANTUML_JAR") };
}

#[test]
fn default_install_path_はhomeディレクトリ配下() {
    let path = plantuml_renderer::default_install_path();
    assert!(path.is_some());
    let p = path.unwrap();
    assert!(p.to_string_lossy().contains("plantuml.jar"));
}

#[test]
fn svg_to_html_fragment_はdivで囲む() {
    let html = plantuml_renderer::svg_to_html_fragment("<svg>test</svg>");
    assert!(html.contains("<div class=\"katana-diagram plantuml\">"));
    assert!(html.contains("<svg>test</svg>"));
    assert!(html.contains("</div>"));
}

#[test]
fn run_plantuml_process_存在しないjarでエラー() {
    let result = plantuml_renderer::run_plantuml_process(
        std::path::Path::new("/nonexistent/plantuml.jar"),
        "@startuml\nA -> B\n@enduml",
    );
    // java が見つからないか、JAR が無効でエラーになる
    assert!(result.is_err());
}
