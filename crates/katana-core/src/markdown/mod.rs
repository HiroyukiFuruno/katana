//! Markdown parsing pipeline using `comrak`.

use comrak::{markdown_to_html, ComrakOptions};

pub mod diagram;
pub mod drawio_renderer;
pub mod mermaid_renderer;
pub mod plantuml_renderer;
pub mod svg_rasterize;

pub use diagram::NoOpRenderer;
use diagram::{DiagramBlock, DiagramKind, DiagramRenderer, DiagramResult};

/// Byte length of the fence block start delimiter "```".
const FENCE_OPEN_LEN: usize = 3;

/// Byte length of the fence block end delimiter "\n```".
const FENCE_CLOSE_LEN: usize = 4;

/// Production renderer: delegates each diagram block type to the actual subprocess / XML parser.
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

/// Renders Markdown to HTML using the production `KatanaRenderer`.
pub fn render_with_katana_renderer(source: &str) -> Result<RenderOutput, MarkdownError> {
    render(source, &KatanaRenderer)
}

/// The result of rendering a Markdown buffer.
#[derive(Debug, Clone)]
pub struct RenderOutput {
    pub html: String,
}

/// Errors that may arise during Markdown rendering.
#[derive(Debug, thiserror::Error)]
pub enum MarkdownError {
    #[error("Rendering failed: {0}")]
    RenderFailed(String),
}

/// Build default `comrak` options with GFM extensions enabled.
fn gfm_options() -> ComrakOptions<'static> {
    let mut opts = ComrakOptions::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    // Required to output custom HTML (markup after diagram block conversion) as-is.
    opts.render.unsafe_ = true;
    opts
}

/// Render Markdown to HTML, routing diagram fences through `renderer`.
pub fn render<R: DiagramRenderer>(
    source: &str,
    renderer: &R,
) -> Result<RenderOutput, MarkdownError> {
    let transformed = transform_diagram_blocks(source, renderer);
    let html = markdown_to_html(&transformed, &gfm_options());
    Ok(RenderOutput { html })
}

/// Convenience render using the no-op diagram renderer.
pub fn render_basic(source: &str) -> Result<RenderOutput, MarkdownError> {
    render(source, &diagram::NoOpRenderer)
}

// ── Fence transformation ─────────────────────────────────────────────────────

struct FenceBlock {
    info: String,
    content: String,
    raw: String,
}

/// Extract one complete fenced block from the start of `s`.
fn extract_fence_block(s: &str) -> Option<(FenceBlock, &str)> {
    let body = s.strip_prefix("```")?;
    let info_end = body.find('\n')?;
    let info = body[..info_end].trim().to_string();
    let after_info = &body[info_end + 1..];
    let close = after_info.find("\n```")?;
    let content = after_info[..close].to_string();
    let raw = format!("```{info}\n{content}\n```");
    let rest = after_info[close + FENCE_CLOSE_LEN..]
        .strip_prefix('\n')
        .unwrap_or(&after_info[close + FENCE_CLOSE_LEN..]);
    Some((FenceBlock { info, content, raw }, rest))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn fallback_html(source: &str, error: &str) -> String {
    format!(
        r#"<div class="katana-diagram-error"><p class="katana-diagram-error-label">⚠ Diagram render failed: {e}</p><pre><code>{s}</code></pre></div>"#,
        e = html_escape(error),
        s = html_escape(source),
    )
}

fn render_diagram_block<R: DiagramRenderer>(block: &FenceBlock, renderer: &R) -> Option<String> {
    let kind = DiagramKind::from_info(&block.info)?;
    let diagram = DiagramBlock {
        kind,
        source: block.content.clone(),
    };
    Some(match renderer.render(&diagram) {
        DiagramResult::Ok(html) => html,
        // OkPng is directly converted to RGBA in the UI layer. Only a placeholder in the core layer.
        DiagramResult::OkPng(_) => String::new(),
        DiagramResult::Err { source, error } => fallback_html(&source, &error),
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            ..
        } => fallback_html("", &format!("{tool_name} not found. {install_hint}")),
        // In the core layer, "NotInstalled" is displayed as fallback HTML.
        // The appropriate download UI is rendered in the UI layer by `RenderedSection::NotInstalled`.
        DiagramResult::NotInstalled { kind, .. } => {
            fallback_html("", &format!("{kind} is not installed"))
        }
    })
}

fn process_fence<R: DiagramRenderer>(output: &mut String, remaining: &mut &str, renderer: &R) {
    let Some((block, after)) = extract_fence_block(remaining) else {
        output.push_str("```");
        *remaining = &remaining[FENCE_OPEN_LEN..];
        return;
    };
    if let Some(html) = render_diagram_block(&block, renderer) {
        output.push_str(&html);
    } else {
        output.push_str(&block.raw);
    }
    *remaining = after;
}

/// Walk code fences in `source`, replace diagram blocks with rendered HTML.
fn transform_diagram_blocks<R: DiagramRenderer>(source: &str, renderer: &R) -> String {
    let mut output = String::with_capacity(source.len());
    let mut remaining = source;
    while let Some(fence_start) = remaining.find("\n```") {
        output.push_str(&remaining[..fence_start + 1]);
        remaining = &remaining[fence_start + 1..];
        process_fence(&mut output, &mut remaining, renderer);
    }
    output.push_str(remaining);
    output
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // extract_fence_block: strip_prefix("```") is None (input doesn't start with a fence)
    #[test]
    fn extract_fence_block_no_fence_prefix() {
        assert!(extract_fence_block("not a fence").is_none());
    }

    // extract_fence_block: has "```" but no newline (find('\n') in info_end is None)
    #[test]
    fn extract_fence_block_no_newline_after_info() {
        assert!(extract_fence_block("```mermaid").is_none());
    }

    // extract_fence_block: has an opening fence but no closing fence
    #[test]
    fn extract_fence_block_no_closing_fence() {
        assert!(extract_fence_block("```mermaid\ngraph TD; A-->B").is_none());
    }

    // extract_fence_block: valid fence block
    #[test]
    fn extract_fence_block_valid() {
        let result = extract_fence_block("```mermaid\ngraph TD; A-->B\n```\nrest");
        assert!(result.is_some());
        let (block, rest) = result.unwrap();
        assert_eq!(block.info, "mermaid");
        assert_eq!(block.content, "graph TD; A-->B");
        assert_eq!(rest, "rest");
    }
}
