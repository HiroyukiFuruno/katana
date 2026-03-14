//! Diagram block detection and rendering adapter contracts.
//!
//! MVP supported input formats:
//! - `mermaid`  — raw Mermaid source text in a fenced `mermaid` block
//! - `plantuml` — raw PlantUML source with `@startuml`/`@enduml` delimiters
//! - `drawio`   — raw uncompressed XML containing `<mxfile>` or `<mxGraphModel>`

/// Supported diagram kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagramKind {
    Mermaid,
    PlantUml,
    DrawIo,
}

impl DiagramKind {
    /// Map a code-fence info string to a `DiagramKind`, or `None` if unsupported.
    pub fn from_info(info: &str) -> Option<Self> {
        match info.trim().to_ascii_lowercase().as_str() {
            "mermaid" => Some(Self::Mermaid),
            "plantuml" => Some(Self::PlantUml),
            "drawio" => Some(Self::DrawIo),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Mermaid => "Mermaid",
            Self::PlantUml => "PlantUML",
            Self::DrawIo => "Draw.io",
        }
    }
}

/// A parsed diagram block extracted from the Markdown source.
#[derive(Debug, Clone)]
pub struct DiagramBlock {
    pub kind: DiagramKind,
    /// Raw source inside the fence (content only, no delimiters).
    pub source: String,
}

impl DiagramBlock {
    /// Validate MVP input format constraints.
    pub fn validate(&self) -> Result<(), DiagramValidationError> {
        match self.kind {
            DiagramKind::Mermaid => {
                // Any non-empty source is accepted.
                if self.source.trim().is_empty() {
                    return Err(DiagramValidationError::EmptySource {
                        kind: self.kind.display_name(),
                    });
                }
            }
            DiagramKind::PlantUml => {
                let src = self.source.trim();
                if !src.contains("@startuml") || !src.contains("@enduml") {
                    return Err(DiagramValidationError::MissingDelimiters {
                        kind: "plantuml",
                        message: "PlantUML blocks must contain @startuml and @enduml".to_string(),
                    });
                }
            }
            DiagramKind::DrawIo => {
                let src = self.source.trim();
                if !src.contains("<mxfile") && !src.contains("<mxGraphModel") {
                    return Err(DiagramValidationError::UnsupportedEncoding {
                        kind: "drawio",
                        message: "Draw.io blocks must contain raw uncompressed XML with <mxfile> or <mxGraphModel>".to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Validation errors for diagram block input.
#[derive(Debug, thiserror::Error)]
pub enum DiagramValidationError {
    #[error("{kind} block has empty source")]
    EmptySource { kind: &'static str },

    #[error("{kind} block is missing required delimiters: {message}")]
    MissingDelimiters { kind: &'static str, message: String },

    #[error("{kind} block uses an unsupported encoding: {message}")]
    UnsupportedEncoding { kind: &'static str, message: String },
}

/// Result returned by a diagram renderer.
#[derive(Debug, Clone)]
pub enum DiagramResult {
    /// Successfully rendered HTML/SVG fragment.
    Ok(String),
    /// Rendering failed; preserve the original source for fallback display.
    Err { source: String, error: String },
}

/// Trait that all diagram renderer adapters must implement.
pub trait DiagramRenderer: Send + Sync {
    fn render(&self, block: &DiagramBlock) -> DiagramResult;
}

/// A no-op renderer: outputs the diagram source as a plain code block.
/// Used as the default until real renderers are integrated.
pub struct NoOpRenderer;

impl DiagramRenderer for NoOpRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        // Pass through as a code block — the diagram source is visible but not rendered.
        let html = format!(
            "<pre><code class=\"language-{kind}\">{source}</code></pre>",
            kind = match block.kind {
                DiagramKind::Mermaid => "mermaid",
                DiagramKind::PlantUml => "plantuml",
                DiagramKind::DrawIo => "drawio",
            },
            source = html_escape(&block.source),
        );
        DiagramResult::Ok(html)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
