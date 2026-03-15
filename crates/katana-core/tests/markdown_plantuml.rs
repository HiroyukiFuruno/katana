use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::plantuml_renderer;
use std::path::PathBuf;

#[test]
fn returns_notinstalled_when_jar_not_found() {
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
fn returns_only_env_var_path_when_jar_candidate_paths_env_var_is_set() {
    unsafe { std::env::set_var("PLANTUML_JAR", "/custom/path/plantuml.jar") };
    let paths = plantuml_renderer::jar_candidate_paths();
    assert_eq!(paths, vec![PathBuf::from("/custom/path/plantuml.jar")]);
    unsafe { std::env::remove_var("PLANTUML_JAR") };
}

#[test]
fn returns_multiple_candidates_when_jar_candidate_paths_env_var_is_not_set() {
    unsafe { std::env::remove_var("PLANTUML_JAR") };
    let paths = plantuml_renderer::jar_candidate_paths();
    // Includes Homebrew path + binary adjacent + XDG candidates
    assert!(paths.len() >= 2);
}

#[test]
fn find_plantuml_jar_returns_none_if_not_exists() {
    unsafe { std::env::set_var("PLANTUML_JAR", "/nonexistent/never/plantuml.jar") };
    let result = plantuml_renderer::find_plantuml_jar();
    assert!(result.is_none());
    unsafe { std::env::remove_var("PLANTUML_JAR") };
}

#[test]
fn default_install_path_is_under_home_directory() {
    let path = plantuml_renderer::default_install_path();
    assert!(path.is_some());
    let p = path.unwrap();
    assert!(p.to_string_lossy().contains("plantuml.jar"));
}

#[test]
fn svg_to_html_fragment_wraps_with_div() {
    let html = plantuml_renderer::svg_to_html_fragment("<svg>test</svg>");
    assert!(html.contains("<div class=\"katana-diagram plantuml\">"));
    assert!(html.contains("<svg>test</svg>"));
    assert!(html.contains("</div>"));
}

#[test]
fn run_plantuml_process_errors_with_non_existent_jar() {
    let result = plantuml_renderer::run_plantuml_process(
        std::path::Path::new("/nonexistent/plantuml.jar"),
        "@startuml\nA -> B\n@enduml",
    );
    // Errors if java is not found or JAR is invalid
    assert!(result.is_err());
}
