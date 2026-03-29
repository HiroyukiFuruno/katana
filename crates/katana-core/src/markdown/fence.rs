use super::diagram::{DiagramBlock, DiagramKind, DiagramRenderer, DiagramResult};

pub const FENCE_OPEN_LEN: usize = 3;

pub const FENCE_CLOSE_LEN: usize = 4;

pub struct FenceBlock {
    pub info: String,
    pub content: String,
    pub raw: String,
}

pub fn extract_fence_block(s: &str) -> Option<(FenceBlock, &str)> {
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

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn fallback_html(source: &str, error: &str) -> String {
    format!(
        r#"<div class="katana-diagram-error"><p class="katana-diagram-error-label">⚠ Diagram render failed: {e}</p><pre><code>{s}</code></pre></div>"#,
        e = html_escape(error),
        s = html_escape(source),
    )
}

pub fn render_diagram_block<R: DiagramRenderer>(
    block: &FenceBlock,
    renderer: &R,
) -> Option<String> {
    let kind = DiagramKind::from_info(&block.info)?;
    let diagram = DiagramBlock {
        kind,
        source: block.content.clone(),
    };
    Some(match renderer.render(&diagram) {
        DiagramResult::Ok(html) => html,
        // WHY: OkPng: embed as a base64 data URI so the diagram shows in exported HTML.
        DiagramResult::OkPng(bytes) => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            format!(
                r#"<div class="katana-diagram mermaid"><img src="data:image/png;base64,{b64}" style="max-width:100%" /></div>"#
            )
        }
        DiagramResult::Err { source, error } => fallback_html(&source, &error),
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            ..
        } => fallback_html("", &format!("{tool_name} not found. {install_hint}")),
        /* WHY: In the core layer, "NotInstalled" is displayed as fallback text.
        The appropriate download UI is rendered in the UI layer by `RenderedSection::NotInstalled`. */
        DiagramResult::NotInstalled { kind, .. } => {
            fallback_html("", &format!("{kind} is not installed"))
        }
    })
}

pub fn process_fence<R: DiagramRenderer>(output: &mut String, remaining: &mut &str, renderer: &R) {
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

pub fn transform_diagram_blocks<R: DiagramRenderer>(source: &str, renderer: &R) -> String {
    let mut output = String::with_capacity(source.len());
    let mut remaining = source;
    loop {
        let fence_offset = if remaining.starts_with("```") {
            Some(0)
        } else {
            remaining.find("\n```").map(|pos| pos + 1)
        };
        let Some(offset) = fence_offset else {
            break;
        };
        output.push_str(&remaining[..offset]);
        remaining = &remaining[offset..];
        process_fence(&mut output, &mut remaining, renderer);
    }
    output.push_str(remaining);
    output
}

// WHY: ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::diagram::NoOpRenderer;

    // WHY: extract_fence_block: strip_prefix("```") is None (input doesn't start with a fence)
    #[test]
    fn extract_fence_block_no_fence_prefix() {
        assert!(extract_fence_block("not a fence").is_none());
    }

    // WHY: extract_fence_block: has "```" but no newline (find('\n') in info_end is None)
    #[test]
    fn extract_fence_block_no_newline_after_info() {
        assert!(extract_fence_block("```mermaid").is_none());
    }

    // WHY: extract_fence_block: has an opening fence but no closing fence
    #[test]
    fn extract_fence_block_no_closing_fence() {
        assert!(extract_fence_block("```mermaid\ngraph TD; A-->B").is_none());
    }

    // WHY: extract_fence_block: valid fence block
    #[test]
    fn extract_fence_block_valid() {
        let result = extract_fence_block("```mermaid\ngraph TD; A-->B\n```\nrest");
        assert!(result.is_some());
        let (block, rest) = result.unwrap();
        assert_eq!(block.info, "mermaid");
        assert_eq!(block.content, "graph TD; A-->B");
        assert_eq!(rest, "rest");
    }

    // WHY: transform_diagram_blocks: diagram fence at the very start of input
    #[test]
    fn transform_handles_fence_at_start_of_input() {
        let source = "```mermaid\ngraph TD; A-->B\n```\nAfter";
        let result = transform_diagram_blocks(source, &NoOpRenderer);
        /* WHY: NoOpRenderer returns empty string for OkPng, so diagram is replaced with nothing.
        But the key assertion is that the "After" text is preserved and no panic occurs. */
        assert!(result.contains("After"));
    }

    struct PngTestRenderer;
    impl DiagramRenderer for PngTestRenderer {
        fn render(&self, _block: &DiagramBlock) -> DiagramResult {
            DiagramResult::OkPng(vec![0x89, 0x50, 0x4E, 0x47]) // PNG magic bytes
        }
    }

    // WHY: render_diagram_block: OkPng should be embedded as base64 <img> tag
    #[test]
    fn render_diagram_block_okpng_embeds_base64_img() {
        let block = FenceBlock {
            info: "mermaid".to_string(),
            content: "graph TD; A-->B".to_string(),
            raw: "```mermaid\ngraph TD; A-->B\n```".to_string(),
        };
        let result = render_diagram_block(&block, &PngTestRenderer);
        let html = result.expect("mermaid blocks should produce Some");
        assert!(
            html.contains("data:image/png;base64,"),
            "OkPng should embed a base64 data URI, got: {html}"
        );
        assert!(
            html.contains("<img"),
            "OkPng should produce an <img> tag, got: {html}"
        );
    }

    // WHY: transform_diagram_blocks with PngTestRenderer produces <img> in output
    #[test]
    fn transform_with_png_renderer_embeds_base64_in_output() {
        let source = "# Hello\n\n```mermaid\ngraph TD; A-->B\n```\n\nAfter";
        let result = transform_diagram_blocks(source, &PngTestRenderer);
        assert!(
            result.contains("data:image/png;base64,"),
            "Output should contain base64 image data"
        );
        assert!(result.contains("After"), "Text after diagram preserved");
    }
}
