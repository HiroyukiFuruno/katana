use katana_core::markdown::diagram::*;

#[test]
fn mermaid_kind_parsed() {
    assert_eq!(
        DiagramKind::from_info("mermaid"),
        Some(DiagramKind::Mermaid)
    );
    assert_eq!(
        DiagramKind::from_info("Mermaid"),
        Some(DiagramKind::Mermaid)
    );
}

#[test]
fn plantuml_kind_parsed() {
    assert_eq!(
        DiagramKind::from_info("plantuml"),
        Some(DiagramKind::PlantUml)
    );
}

#[test]
fn drawio_kind_parsed() {
    assert_eq!(DiagramKind::from_info("drawio"), Some(DiagramKind::DrawIo));
}

#[test]
fn unknown_info_is_none() {
    assert_eq!(DiagramKind::from_info("rust"), None);
    assert_eq!(DiagramKind::from_info(""), None);
}

#[test]
fn plantuml_validation_requires_delimiters() {
    let block_ok = DiagramBlock {
        kind: DiagramKind::PlantUml,
        source: "@startuml\nA -> B\n@enduml".to_string(),
    };
    let block_bad = DiagramBlock {
        kind: DiagramKind::PlantUml,
        source: "A -> B".to_string(),
    };
    assert!(block_ok.validate().is_ok());
    assert!(block_bad.validate().is_err());
}

#[test]
fn drawio_validation_requires_mxfile_or_mxgraphmodel() {
    let ok1 = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: "<mxfile><diagram></diagram></mxfile>".to_string(),
    };
    let ok2 = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: "<mxGraphModel><root/></mxGraphModel>".to_string(),
    };
    let bad = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: "compressed+base64data".to_string(),
    };
    assert!(ok1.validate().is_ok());
    assert!(ok2.validate().is_ok());
    assert!(bad.validate().is_err());
}

#[test]
fn noop_renderer_returns_ok_with_code_block() {
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: "graph TD; A-->B".to_string(),
    };
    let result = NoOpRenderer.render(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
    if let DiagramResult::Ok(html) = result {
        assert!(html.contains("mermaid"));
    }
}
