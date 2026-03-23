#[test]
fn test_sourcepos_bytes() {
    use comrak::{parse_document, Arena, Options};
    use comrak::nodes::NodeValue;
    let arena = Arena::new();
    let src = "Hello\nThis is an ![alt](test.png) text\n";
    let doc = parse_document(&arena, src, &Options::default());
    for node in doc.descendants() {
        if let NodeValue::Image(_) = node.data.borrow().value {
            let pos = node.data.borrow().sourcepos;
            let lines: Vec<&str> = src.lines().collect();
            let line = lines[pos.start.line - 1];
            assert_eq!(&line[pos.start.column - 1..pos.end.column], "![alt](test.png)");
        }
    }
}
