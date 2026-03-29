use comrak::{markdown_to_html, ComrakOptions};

use super::diagram::{self, DiagramBlock, DiagramKind, DiagramRenderer, DiagramResult};
use super::drawio_renderer;
use super::mermaid_renderer;
use super::plantuml_renderer;

use super::fence::transform_diagram_blocks;

#[derive(Debug, Default)]
pub struct KatanaRenderer;

impl DiagramRenderer for KatanaRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        match block.kind {
            DiagramKind::Mermaid => mermaid_renderer::render_mermaid(block),
            DiagramKind::PlantUml => plantuml_renderer::render_plantuml(block),
            DiagramKind::DrawIo => drawio_renderer::render_drawio(block),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderOutput {
    pub html: String,
}

#[derive(Debug, thiserror::Error)]
pub enum MarkdownError {
    #[error("Rendering failed: {0}")]
    RenderFailed(String),
    #[error("Export failed: {0}")]
    ExportFailed(String),
}

pub fn gfm_options() -> ComrakOptions<'static> {
    let mut opts = ComrakOptions::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    // WHY: Required to output custom HTML (markup after diagram block conversion) as-is.
    opts.render.unsafe_ = true;
    opts
}

pub fn render_with_katana_renderer(source: &str) -> Result<RenderOutput, MarkdownError> {
    render(source, &KatanaRenderer)
}

pub fn render<R: DiagramRenderer>(
    source: &str,
    renderer: &R,
) -> Result<RenderOutput, MarkdownError> {
    let transformed = transform_diagram_blocks(source, renderer);
    let html = markdown_to_html(&transformed, &gfm_options());
    Ok(RenderOutput { html })
}

pub fn render_basic(source: &str) -> Result<RenderOutput, MarkdownError> {
    render(source, &diagram::NoOpRenderer)
}
