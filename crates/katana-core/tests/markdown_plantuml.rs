use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::plantuml_renderer::render_plantuml;

#[test]
fn jar未検出時はnotinstalledを返す() {
    // PLANTUML_JAR を意図的に存在しないパスに向ける。
    unsafe { std::env::set_var("PLANTUML_JAR", "/nonexistent/plantuml.jar") };
    let block = DiagramBlock {
        kind: DiagramKind::PlantUml,
        source: "@startuml\nA -> B\n@enduml".to_string(),
    };
    let result = render_plantuml(&block);
    assert!(matches!(result, DiagramResult::NotInstalled { .. }));
    unsafe { std::env::remove_var("PLANTUML_JAR") };
}
