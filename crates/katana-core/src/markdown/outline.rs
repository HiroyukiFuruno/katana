use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, Options};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineItem {
    pub level: u8,
    pub text: String,
    pub index: usize,
}

fn extract_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    for child in node.children() {
        extract_text_from_node(child, &mut text);
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_text_from_node<'a>(node: &'a AstNode<'a>, out: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Text(ref text) => out.push_str(text),
        NodeValue::Code(ref code) => out.push_str(&code.literal),
        NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
        _ => {}
    }
    for child in node.children() {
        extract_text_from_node(child, out);
    }
}

pub fn extract_outline(source: &str) -> Vec<OutlineItem> {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.front_matter_delimiter = Some("---".to_string());

    let root = parse_document(&arena, source, &options);
    let mut outline = Vec::new();
    let mut index = 0;

    for node in root.descendants() {
        if let NodeValue::Heading(ref heading) = node.data.borrow().value {
            let text = extract_text(node);
            outline.push(OutlineItem {
                level: heading.level,
                text,
                index,
            });
            index += 1;
        }
    }

    outline
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_outline_with_line_breaks() {
        let md = "Heading\nwith softbreak\n=======\n\nHeading  \nwith linebreak\n-------";
        let outline = extract_outline(md);
        assert_eq!(outline.len(), 2);
        assert_eq!(outline[0].text, "Heading with softbreak");
        assert_eq!(outline[1].text, "Heading with linebreak");
    }

    #[test]
    fn test_extract_outline_with_complex_formatting() {
        let md = r#"
# Heading with `code`
Some text
## Heading with [link](http://example.com)
More text
### **Bold** and *italic*
"#;
        let outline = extract_outline(md);
        assert_eq!(outline.len(), 3);
        assert_eq!(outline[0].text, "Heading with code");
        assert_eq!(outline[1].text, "Heading with link");
        assert_eq!(outline[2].text, "Bold and italic");
    }

    #[test]
    fn test_extract_outline() {
        let md = r#"
# Heading 1
Some text.
## Heading 2
More text.
### Heading 3
"#;
        let outline = extract_outline(md);
        assert_eq!(outline.len(), 3);
        assert_eq!(
            outline[0],
            OutlineItem {
                level: 1,
                text: "Heading 1".to_string(),
                index: 0,
            }
        );
        assert_eq!(
            outline[1],
            OutlineItem {
                level: 2,
                text: "Heading 2".to_string(),
                index: 1,
            }
        );
        assert_eq!(
            outline[2],
            OutlineItem {
                level: 3,
                text: "Heading 3".to_string(),
                index: 2,
            }
        );
    }
}
