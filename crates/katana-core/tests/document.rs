use katana_core::Document;

#[test]
fn new_document_is_clean() {
    let doc = Document::new("/tmp/test.md", "# Hello");
    assert!(!doc.is_dirty);
    assert_eq!(doc.buffer, "# Hello");
}

#[test]
fn update_buffer_marks_dirty() {
    let mut doc = Document::new("/tmp/test.md", "# Hello");
    doc.update_buffer("# Hello World");
    assert!(doc.is_dirty);
    assert_eq!(doc.buffer, "# Hello World");
}

#[test]
fn mark_clean_clears_dirty_flag() {
    let mut doc = Document::new("/tmp/test.md", "# Hello");
    doc.update_buffer("# Changed");
    doc.mark_clean();
    assert!(!doc.is_dirty);
}

#[test]
fn update_with_same_content_does_not_dirty() {
    let mut doc = Document::new("/tmp/test.md", "# Hello");
    doc.update_buffer("# Hello");
    assert!(!doc.is_dirty);
}

#[test]
fn file_name_returns_basename() {
    let doc = Document::new("/workspace/spec.md", "");
    assert_eq!(doc.file_name(), Some("spec.md"));
}
