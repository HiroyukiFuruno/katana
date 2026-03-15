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
    /// Successfully rendered as PNG bytes (e.g. mmdc output).
    OkPng(Vec<u8>),
    /// Rendering failed; preserve the original source for fallback display.
    Err { source: String, error: String },
    /// Required command line tool is not found.
    CommandNotFound {
        tool_name: String,
        install_hint: String,
        source: String,
    },
    /// Required runtime tool is not installed (supports auto-download).
    NotInstalled {
        /// Display name (e.g., "PlantUML").
        kind: String,
        /// Download URL.
        download_url: String,
        /// Installation path.
        install_path: std::path::PathBuf,
    },
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
