//! Preview pane — native Markdown rendering with egui_commonmark
//! + rasterized image display of diagram blocks.
//!
//! Design considerations (MVP):
//! - The Markdown part is updated immediately on every text change (egui_commonmark).
//! - Diagrams involve sub-processes, so they are re-rendered only when
//!   the "🔄 Refresh" button is clicked or a document is selected.

use eframe::egui::{self, ScrollArea};
use egui_commonmark::CommonMarkCache;
use katana_core::markdown::diagram::DiagramKind;
use katana_core::{
    markdown::{
        diagram::{DiagramBlock, DiagramResult},
        drawio_renderer, mermaid_renderer, plantuml_renderer,
        svg_rasterize::{rasterize_svg, RasterizedSvg},
    },
    preview::{flatten_list_code_blocks, resolve_image_paths, split_into_sections, PreviewSection},
};

// ─────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────

/// Display scale when converting diagram SVG to pixel images.
const DIAGRAM_SVG_DISPLAY_SCALE: f32 = 1.5;

/// Interval (in milliseconds) used to poll background render tasks.
const RENDER_POLL_INTERVAL_MS: u64 = 50;

/// Rendered sections held in the UI layer.
#[derive(Debug, Clone)]
pub enum RenderedSection {
    /// Markdown text rendered by egui_commonmark.
    Markdown(String),
    /// Rasterized diagram image.
    Image {
        svg_data: RasterizedSvg,
        alt: String,
    },
    /// Rendering error (holds source and message).
    Error {
        kind: String,
        _source: String,
        message: String,
    },
    /// Command line tool not found (path issues, etc.).
    CommandNotFound {
        tool_name: String,
        install_hint: String,
        _source: String,
    },
    /// Required tool is not installed — can be downloaded from the UI.
    NotInstalled {
        kind: String,
        download_url: String,
        install_path: std::path::PathBuf,
    },
    /// Placeholder during background rendering.
    Pending { kind: String },
}

/// Download request returned by the preview pane.
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub dest: std::path::PathBuf,
}

#[derive(Default)]
pub struct PreviewPane {
    commonmark_cache: CommonMarkCache,
    pub sections: Vec<RenderedSection>,
    /// Channel for background rendering completion notifications.
    render_rx: Option<std::sync::mpsc::Receiver<(usize, RenderedSection)>>,
    /// Path to the currently previewed Markdown file (for resolving relative paths in render_html_fn).
    md_file_path: std::path::PathBuf,
}

impl PreviewPane {
    /// Immediately updates only text sections from the Markdown source (diagrams are preserved).
    pub fn update_markdown_sections(&mut self, source: &str, md_file_path: &std::path::Path) {
        self.md_file_path = md_file_path.to_path_buf();
        let resolved = resolve_image_paths(source, md_file_path);
        let flattened = flatten_list_code_blocks(&resolved);
        let raw = split_into_sections(&flattened);
        let mut new_sections = Vec::with_capacity(raw.len());
        let mut diagram_iter = self
            .sections
            .iter()
            .filter(|s| !matches!(s, RenderedSection::Markdown(_)));
        for section in &raw {
            match section {
                PreviewSection::Markdown(md) => {
                    new_sections.push(RenderedSection::Markdown(md.clone()));
                }
                PreviewSection::Diagram { kind, source } => {
                    // Reuse existing rendered image if available.
                    let reused =
                        diagram_iter
                            .next()
                            .cloned()
                            .unwrap_or_else(|| RenderedSection::Error {
                                kind: format!("{kind:?}"),
                                _source: source.clone(),
                                message: "🔄 Please refresh the preview".to_string(),
                            });
                    new_sections.push(reused);
                }
            }
        }
        self.sections = new_sections;
    }

    /// Completely re-renders all sections (including diagrams).
    ///
    /// Returns Markdown sections immediately. Diagrams are set to `Pending`
    /// and rendered in a background thread.
    pub fn full_render(&mut self, source: &str, md_file_path: &std::path::Path) {
        self.md_file_path = md_file_path.to_path_buf();
        let resolved = resolve_image_paths(source, md_file_path);
        let flattened = flatten_list_code_blocks(&resolved);
        let raw = split_into_sections(&flattened);
        // Cancel previous rendering.
        self.render_rx = None;

        let mut sections = Vec::with_capacity(raw.len());
        let mut jobs: Vec<(usize, DiagramKind, String)> = Vec::new();

        for (i, section) in raw.iter().enumerate() {
            match section {
                PreviewSection::Markdown(md) => {
                    sections.push(RenderedSection::Markdown(md.clone()));
                }
                PreviewSection::Diagram { kind, source: src } => {
                    sections.push(RenderedSection::Pending {
                        kind: format!("{kind:?}"),
                    });
                    jobs.push((i, kind.clone(), src.clone()));
                }
            }
        }
        self.sections = sections;

        if jobs.is_empty() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.render_rx = Some(rx);
        std::thread::spawn(move || {
            for (index, kind, src) in jobs {
                let section = render_diagram(&kind, &src);
                if tx.send((index, section)).is_err() {
                    break; // Receiver was dropped.
                }
            }
        });
    }

    /// Renders the preview pane content (including ScrollArea).
    /// Used when scroll sync is not needed, such as in PreviewOnly mode.
    /// Returns `Some(DownloadRequest)` if the download button is pressed.
    #[allow(dead_code)]
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        // Poll for background rendering completion.
        self.poll_renders(ui.ctx());

        let mut request: Option<DownloadRequest> = None;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                request = self.render_sections(ui);
            });
        request
    }

    /// Renders only the preview content without a ScrollArea.
    /// Used when you want to control the outer ScrollArea (e.g. for scroll sync).
    pub fn show_content(&mut self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        self.poll_renders(ui.ctx());
        self.render_sections(ui)
    }

    /// Internal method to sequentially render sections.
    /// Actual UI rendering is delegated to preview_pane_ui::render_sections.
    fn render_sections(&mut self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        crate::preview_pane_ui::render_sections(
            ui,
            &mut self.commonmark_cache,
            &self.sections,
            &self.md_file_path,
        )
    }

    /// Polls for background rendering completion and updates sections with received results.
    fn poll_renders(&mut self, ctx: &egui::Context) {
        let still_pending = if let Some(rx) = &self.render_rx {
            let mut updated = false;
            while let Ok((idx, section)) = rx.try_recv() {
                if let Some(slot) = self.sections.get_mut(idx) {
                    *slot = section;
                    updated = true;
                }
            }
            if updated {
                ctx.request_repaint();
            }
            self.sections
                .iter()
                .any(|s| matches!(s, RenderedSection::Pending { .. }))
        } else {
            false
        };
        if still_pending {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            self.render_rx = None;
        }
    }

    /// For testing: Block and wait on the background thread until there are no Pending sections.
    /// Available in release builds too (harmless no-op when no background render is running).
    pub fn wait_for_renders(&mut self) {
        while let Some(rx) = &self.render_rx {
            while let Ok((idx, section)) = rx.try_recv() {
                if let Some(slot) = self.sections.get_mut(idx) {
                    *slot = section;
                }
            }
            if self
                .sections
                .iter()
                .any(|s| matches!(s, RenderedSection::Pending { .. }))
            {
                std::thread::sleep(std::time::Duration::from_millis(RENDER_POLL_INTERVAL_MS));
            } else {
                self.render_rx = None;
                break;
            }
        }
    }
}

/// Renders a `PreviewSection` into a `RenderedSection`.
/// Converts diagram blocks via the renderer and attempts SVG rasterization.
fn render_diagram(kind: &DiagramKind, source: &str) -> RenderedSection {
    let block = DiagramBlock {
        kind: kind.clone(),
        source: source.to_string(),
    };
    let result = dispatch_renderer(&block);
    map_diagram_result(kind, source, result)
}

/// Pure function converting a `DiagramResult` into a `RenderedSection`. Exposed for testing.
pub(crate) fn map_diagram_result(
    kind: &DiagramKind,
    source: &str,
    result: DiagramResult,
) -> RenderedSection {
    match result {
        DiagramResult::Ok(html) => try_rasterize(kind, source, &html),
        DiagramResult::OkPng(bytes) => decode_png_to_section(kind, source, bytes),
        DiagramResult::Err { source, error } => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source,
            message: error,
        },
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            source,
        } => RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            _source: source,
        },
        DiagramResult::NotInstalled {
            kind: k,
            download_url,
            install_path,
        } => RenderedSection::NotInstalled {
            kind: k,
            download_url,
            install_path,
        },
    }
}

/// Delegates to the appropriate renderer per diagram kind.
fn dispatch_renderer(block: &DiagramBlock) -> DiagramResult {
    match block.kind {
        DiagramKind::Mermaid => mermaid_renderer::render_mermaid(block),
        DiagramKind::PlantUml => plantuml_renderer::render_plantuml(block),
        DiagramKind::DrawIo => drawio_renderer::render_drawio(block),
    }
}

/// Extracts SVG from an HTML fragment and rasterizes it.
fn try_rasterize(kind: &DiagramKind, source: &str, html: &str) -> RenderedSection {
    let Some(svg) = extract_svg(html) else {
        return RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: "Failed to extract SVG".to_string(),
        };
    };
    match rasterize_svg(svg, DIAGRAM_SVG_DISPLAY_SCALE) {
        Ok(img) => RenderedSection::Image {
            svg_data: img,
            alt: format!("{kind:?} diagram"),
        },
        Err(e) => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: e.to_string(),
        },
    }
}

/// Extracts `<svg...>...</svg>` from an HTML fragment.
pub fn extract_svg(html: &str) -> Option<&str> {
    let start = html.find("<svg")?;
    let end = html.rfind("</svg>")? + "</svg>".len();
    Some(&html[start..end])
}

/// Converts PNG bytes to `RenderedSection::Image`.
///
/// Decodes mmdc PNG output using the `image` crate to get an RGBA pixel buffer.
/// This completely avoids resvg's lack of support for `<foreignObject>`.
fn decode_png_to_section(kind: &DiagramKind, source: &str, bytes: Vec<u8>) -> RenderedSection {
    match decode_png_rgba(&bytes) {
        Ok(rasterized) => RenderedSection::Image {
            svg_data: rasterized,
            alt: format!("{kind:?} diagram"),
        },
        Err(e) => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: format!("PNG decode failed: {e}"),
        },
    }
}

/// Converts PNG bytes to RGBA pixels.
pub fn decode_png_rgba(bytes: &[u8]) -> Result<RasterizedSvg, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(RasterizedSvg {
        width,
        height,
        rgba: rgba.into_raw(),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    // ── Markdown image parsing (test-only utilities) ──

    /// Byte length of the badge image prefix `[![`.
    const BADGE_PREFIX_LEN: usize = "[![".len();

    /// Parsed Markdown image reference.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MdImage {
        src: String,
        alt: String,
        /// Number of characters consumed from the input string.
        consumed: usize,
    }

    /// Finds the byte offset of the next image pattern (`![` or `[![`).
    fn find_next_image(s: &str) -> Option<usize> {
        let pos = s.find("![")?;
        if pos > 0 && s.as_bytes()[pos - 1] == b'[' {
            Some(pos - 1)
        } else {
            Some(pos)
        }
    }

    /// Parses a Markdown image at the start of the given string.
    fn parse_md_image(s: &str) -> Option<MdImage> {
        if let Some(rest) = s.strip_prefix("[![") {
            let alt_end = rest.find(']')?;
            let alt = &rest[..alt_end];
            let after_alt = &rest[alt_end + 1..];
            let inner_src = after_alt.strip_prefix('(')?;
            let src_end = inner_src.find(')')?;
            let src = &inner_src[..src_end];
            let after_inner = &inner_src[src_end + 1..];
            let after_bracket = after_inner.strip_prefix("](")?;
            let link_end = after_bracket.find(')')?;
            let total = BADGE_PREFIX_LEN + alt_end + 1 + 1 + src_end + 1 + 2 + link_end + 1;
            return Some(MdImage {
                src: src.to_string(),
                alt: alt.to_string(),
                consumed: total,
            });
        }

        let rest = s.strip_prefix("![")?;
        let close_bracket = rest.find("](")?;
        let alt = &rest[..close_bracket];
        let after = &rest[close_bracket + 2..];
        let close_paren = after.find(')')?;
        let src = &after[..close_paren];
        if src.is_empty() {
            return None;
        }
        let total = 2 + close_bracket + 2 + close_paren + 1;
        Some(MdImage {
            src: src.to_string(),
            alt: alt.to_string(),
            consumed: total,
        })
    }

    /// Alternative to `matches!` macro. Avoids uncovered lines issue caused by
    /// subregions (`^0`) generated by LLVM in the else branch of `matches!`.
    macro_rules! assert_variant {
        ($expr:expr, $pat:pat) => {
            let val = &$expr;
            assert!(
                if let $pat = val { true } else { false },
                "expected {}, got {:?}",
                stringify!($pat),
                val
            );
        };
    }
    // render_diagram: Maps DrawIo result to RenderedSection
    #[test]
    fn render_diagram_drawio_returns_ok_section() {
        let xml = r#"<mxGraphModel><root><mxCell id="0"/><mxCell id="1" parent="0"/></root></mxGraphModel>"#;
        let section = render_diagram(&DiagramKind::DrawIo, xml);
        assert_variant!(
            section,
            RenderedSection::Image { .. } | RenderedSection::Error { .. }
        );
    }

    // dispatch_renderer: DrawIo branch
    #[test]
    fn dispatch_renderer_drawio_returns_result() {
        let block = DiagramBlock {
            kind: DiagramKind::DrawIo,
            source: r#"<mxGraphModel><root><mxCell id="0"/></root></mxGraphModel>"#.to_string(),
        };
        let result = dispatch_renderer(&block);
        assert_variant!(result, DiagramResult::Ok(_) | DiagramResult::Err { .. });
    }

    // dispatch_renderer: Mermaid branch
    #[test]
    fn dispatch_renderer_mermaid_when_no_mmdc_returns_command_not_found() {
        let block = DiagramBlock {
            kind: DiagramKind::Mermaid,
            source: "graph TD; A-->B".to_string(),
        };
        let result = dispatch_renderer(&block);
        assert_variant!(
            result,
            DiagramResult::CommandNotFound { .. }
                | DiagramResult::OkPng(_)
                | DiagramResult::Err { .. }
        );
    }

    // dispatch_renderer: PlantUml branch
    #[test]
    fn dispatch_renderer_plantuml_when_no_jar_returns_not_installed() {
        std::env::set_var("PLANTUML_JAR", "/nonexistent/plantuml.jar");
        let block = DiagramBlock {
            kind: DiagramKind::PlantUml,
            source: "@startuml\nA->B\n@enduml".to_string(),
        };
        let result = dispatch_renderer(&block);
        std::env::remove_var("PLANTUML_JAR");
        assert_variant!(result, DiagramResult::NotInstalled { .. });
    }

    // try_rasterize: SVG extraction failure case
    #[test]
    fn try_rasterize_returns_error_when_no_svg_in_html() {
        let kind = DiagramKind::DrawIo;
        let section = try_rasterize(&kind, "source", "<div>no svg here</div>");
        assert_variant!(section, RenderedSection::Error { .. });
    }

    // try_rasterize: Success with valid SVG
    #[test]
    fn try_rasterize_returns_image_for_valid_svg() {
        let kind = DiagramKind::DrawIo;
        let html = r#"<div class="diagram"><svg width="10" height="10"><rect fill="white" width="10" height="10"/></svg></div>"#;
        let section = try_rasterize(&kind, "source", html);
        assert_variant!(
            section,
            RenderedSection::Image { .. } | RenderedSection::Error { .. }
        );
    }

    // decode_png_to_section: Valid PNG
    #[test]
    fn decode_png_to_section_returns_image_for_valid_png() {
        use image::{ImageBuffer, Rgba};
        let mut buf = Vec::new();
        let img = ImageBuffer::from_pixel(2, 2, Rgba([100u8, 150, 200, 255]));
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        let section = decode_png_to_section(&DiagramKind::DrawIo, "source", buf);
        assert_variant!(section, RenderedSection::Image { .. });
    }

    // decode_png_to_section: Invalid data
    #[test]
    fn decode_png_to_section_returns_error_for_invalid_data() {
        let section = decode_png_to_section(&DiagramKind::DrawIo, "source", b"not png".to_vec());
        assert_variant!(section, RenderedSection::Error { .. });
    }

    // map_diagram_result: Exhaustive test for all variants
    #[test]
    fn map_diagram_result_ok_delegates_to_try_rasterize() {
        let section = map_diagram_result(
            &DiagramKind::DrawIo,
            "src",
            DiagramResult::Ok("<svg width=\"10\" height=\"10\"></svg>".to_string()),
        );
        assert_variant!(
            section,
            RenderedSection::Image { .. } | RenderedSection::Error { .. }
        );
    }

    #[test]
    fn map_diagram_result_ok_png_delegates_to_decode() {
        use image::{ImageBuffer, Rgba};
        let mut buf = Vec::new();
        let img = ImageBuffer::from_pixel(2, 2, Rgba([0u8, 0, 0, 255]));
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        let section = map_diagram_result(&DiagramKind::Mermaid, "src", DiagramResult::OkPng(buf));
        assert_variant!(section, RenderedSection::Image { .. });
    }

    #[test]
    fn map_diagram_result_err_maps_to_error_section() {
        let section = map_diagram_result(
            &DiagramKind::DrawIo,
            "src",
            DiagramResult::Err {
                source: "src".to_string(),
                error: "render failed".to_string(),
            },
        );
        assert_variant!(section, RenderedSection::Error { .. });
    }

    #[test]
    fn map_diagram_result_command_not_found_maps_to_section() {
        let section = map_diagram_result(
            &DiagramKind::Mermaid,
            "src",
            DiagramResult::CommandNotFound {
                tool_name: "mmdc".to_string(),
                install_hint: "npm install".to_string(),
                source: "src".to_string(),
            },
        );
        assert_variant!(section, RenderedSection::CommandNotFound { .. });
    }

    #[test]
    fn map_diagram_result_not_installed_maps_to_section() {
        let section = map_diagram_result(
            &DiagramKind::PlantUml,
            "src",
            DiagramResult::NotInstalled {
                kind: "PlantUML".to_string(),
                download_url: "https://example.com".to_string(),
                install_path: std::path::PathBuf::from("/tmp/plantuml.jar"),
            },
        );
        assert_variant!(section, RenderedSection::NotInstalled { .. });
    }

    // render_diagram_mermaid: Integration test (independent of mmdc presence)
    #[test]
    fn render_diagram_mermaid_produces_valid_section() {
        let section = render_diagram(&DiagramKind::Mermaid, "graph TD; A-->B");
        // CommandNotFound if mmdc is absent, Image if present
        assert!(!matches!(section, RenderedSection::Pending { .. }));
    }

    // poll_renders: Receives results from the background thread and updates sections (L200-206)
    #[test]
    fn poll_renders_receives_background_result_and_updates_section() {
        use std::sync::mpsc;
        let mut pane = PreviewPane::default();

        // Set Pending section
        pane.sections = vec![RenderedSection::Pending {
            kind: "DrawIo".to_string(),
        }];

        // Create an mpsc channel and set it to render_rx
        let (tx, rx) = mpsc::channel();
        pane.render_rx = Some(rx);

        // Send a result from the background thread
        tx.send((0, RenderedSection::Markdown("# Result".to_string())))
            .unwrap();
        // Drop tx so the receiver becomes Disconnected
        drop(tx);

        // egui Context is required to call poll_renders
        let ctx = egui::Context::default();
        pane.poll_renders(&ctx);

        // The section has been updated
        assert_variant!(pane.sections[0], RenderedSection::Markdown(_));
        // render_rx is None (Pending removed)
        assert!(pane.render_rx.is_none());
    }

    // wait_for_renders: Wait until Pending sections are gone (L224-242)
    #[test]
    fn wait_for_renders_blocks_until_all_rendered() {
        use std::sync::mpsc;
        let mut pane = PreviewPane::default();

        pane.sections = vec![RenderedSection::Pending {
            kind: "DrawIo".to_string(),
        }];

        let (tx, rx) = mpsc::channel();
        pane.render_rx = Some(rx);

        // Send in another thread
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let _ = tx.send((0, RenderedSection::Markdown("# Done".to_string())));
        });

        pane.wait_for_renders();

        // No longer Pending after completion
        assert!(pane.render_rx.is_none());
        assert_variant!(pane.sections[0], RenderedSection::Markdown(_));
    }

    // poll_renders: Do nothing without render_rx (L211-213)
    #[test]
    fn poll_renders_without_rx_does_nothing() {
        let mut pane = PreviewPane::default();
        // render_rx remains None
        let ctx = egui::Context::default();
        pane.poll_renders(&ctx);
        // OK as long as no crash occurs
        assert!(pane.render_rx.is_none());
    }

    // full_render: Starts thread and generates Pending section (L140-149)
    #[test]
    fn full_render_with_diagram_creates_pending_section_then_renders() {
        let mut pane = PreviewPane::default();
        // Content containing a DrawIo diagram -> evaluates to Pending
        let source = "# Title\n```drawio\n<mxGraphModel><root></root></mxGraphModel>\n```";
        pane.full_render(source, std::path::Path::new("/tmp/test.md"));

        // render_rx is set (because there is a diagram)
        assert!(pane.render_rx.is_some());

        // Wait and confirm that there are no crashes
        pane.wait_for_renders();
        assert!(pane.render_rx.is_none());
    }

    // ── parse_md_image / find_next_image unit tests ──

    #[test]
    fn parse_md_image_simple_image() {
        let input = "![alt text](path/to/image.png)";
        let img = parse_md_image(input).unwrap();
        assert_eq!(img.src, "path/to/image.png");
        assert_eq!(img.alt, "alt text");
        assert_eq!(img.consumed, input.len());
    }

    #[test]
    fn parse_md_image_simple_with_file_uri() {
        let input = "![icon](file:///tmp/icon.png)";
        let img = parse_md_image(input).unwrap();
        assert_eq!(img.src, "file:///tmp/icon.png");
        assert_eq!(img.alt, "icon");
        assert_eq!(img.consumed, input.len());
    }

    #[test]
    fn parse_md_image_badge_pattern() {
        let input = "[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)";
        let img = parse_md_image(input).unwrap();
        assert_eq!(img.src, "https://img.shields.io/badge/License-MIT-blue.svg");
        assert_eq!(img.alt, "License: MIT");
        assert_eq!(img.consumed, input.len());
    }

    #[test]
    fn parse_md_image_badge_with_url_link() {
        let input =
            "[![CI](https://github.com/org/repo/badge.svg)](https://github.com/org/repo/actions)";
        let img = parse_md_image(input).unwrap();
        assert_eq!(img.src, "https://github.com/org/repo/badge.svg");
        assert_eq!(img.alt, "CI");
        assert_eq!(img.consumed, input.len());
    }

    #[test]
    fn parse_md_image_consumed_allows_continuation() {
        let input = "[![A](https://a.svg)](link1) ![B](https://b.png)";
        let first = parse_md_image(input).unwrap();
        assert_eq!(first.alt, "A");
        let remainder = &input[first.consumed..];
        let trimmed = remainder.trim_start();
        let second = parse_md_image(trimmed).unwrap();
        assert_eq!(second.alt, "B");
        assert_eq!(second.src, "https://b.png");
    }

    #[test]
    fn parse_md_image_empty_src_returns_none() {
        assert!(parse_md_image("![alt]()").is_none());
    }

    #[test]
    fn parse_md_image_plain_text_returns_none() {
        assert!(parse_md_image("just plain text").is_none());
    }

    #[test]
    fn parse_md_image_incomplete_badge_returns_none() {
        // Missing closing of outer link
        assert!(parse_md_image("[![alt](src)](").is_none());
    }

    #[test]
    fn find_next_image_simple() {
        assert_eq!(find_next_image("hello ![img](src)"), Some(6));
    }

    #[test]
    fn find_next_image_badge() {
        assert_eq!(find_next_image("[![badge](url)](link)"), Some(0));
    }

    #[test]
    fn find_next_image_badge_before_simple() {
        // Badge starts at 0, simple at 1 — should return 0
        assert_eq!(
            find_next_image("[![badge](url)](link) ![img](src)"),
            Some(0)
        );
    }

    #[test]
    fn find_next_image_no_image() {
        assert_eq!(find_next_image("no images here"), None);
    }

    #[test]
    fn find_next_image_with_preceding_text() {
        assert_eq!(
            find_next_image("text before [![badge](url)](link)"),
            Some(12)
        );
    }
}
