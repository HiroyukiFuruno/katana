use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramKind {
    Mermaid,
    PlantUml,
    DrawIo,
}

impl DiagramKind {
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

#[derive(Debug, Clone)]
pub struct DiagramBlock {
    pub kind: DiagramKind,
    pub source: String,
}

impl DiagramBlock {
    pub fn validate(&self) -> Result<(), DiagramValidationError> {
        match self.kind {
            DiagramKind::Mermaid => {
                // WHY: Any non-empty source is accepted.
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

#[derive(Debug, thiserror::Error)]
pub enum DiagramValidationError {
    #[error("{kind} block has empty source")]
    EmptySource { kind: &'static str },

    #[error("{kind} block is missing required delimiters: {message}")]
    MissingDelimiters { kind: &'static str, message: String },

    #[error("{kind} block uses an unsupported encoding: {message}")]
    UnsupportedEncoding { kind: &'static str, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagramResult {
    Ok(String),
    OkPng(Vec<u8>),
    Err {
        source: String,
        error: String,
    },
    CommandNotFound {
        tool_name: String,
        install_hint: String,
        source: String,
    },
    NotInstalled {
        kind: String,
        download_url: String,
        install_path: std::path::PathBuf,
    },
}

pub trait DiagramRenderer: Send + Sync {
    fn render(&self, block: &DiagramBlock) -> DiagramResult;
}

pub struct NoOpRenderer;

impl DiagramRenderer for NoOpRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        // WHY: Pass through as a code block — the diagram source is visible but not rendered.
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
