use super::types::*;
use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::{
    drawio_renderer, mermaid_renderer, plantuml_renderer,
    svg_rasterize::{rasterize_svg, RasterizedSvg},
};

//
//
//
// ─────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────

/// Renders a `PreviewSection` into a `RenderedSection`.
/// Converts diagram blocks via the renderer and attempts SVG rasterization.
#[cfg(test)]
pub fn render_diagram(kind: &DiagramKind, source: &str, source_lines: usize) -> RenderedSection {
    let block = DiagramBlock {
        kind: kind.clone(),
        source: source.to_string(),
    };
    let result = dispatch_renderer(&block);
    map_diagram_result(kind, source, result, source_lines)
}

/// Generates a cache key based on the file path, diagram kind, and source text.
pub fn get_cache_key(md_file_path: &std::path::Path, kind: &DiagramKind, source: &str) -> String {
    use katana_core::markdown::color_preset::DiagramColorPreset;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    md_file_path.hash(&mut hasher);
    kind.display_name().hash(&mut hasher);
    source.hash(&mut hasher);
    DiagramColorPreset::is_dark_mode().hash(&mut hasher);
    format!("diagram_{:x}", hasher.finish())
}

/// Pure function converting a `DiagramResult` into a `RenderedSection`. Exposed for testing.
pub fn map_diagram_result(
    kind: &DiagramKind,
    source: &str,
    result: DiagramResult,
    source_lines: usize,
) -> RenderedSection {
    match result {
        DiagramResult::Ok(html) => try_rasterize(kind, source, &html, source_lines),
        DiagramResult::OkPng(bytes) => decode_png_to_section(kind, source, bytes, source_lines),
        DiagramResult::Err { source, error } => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source,
            message: error,
            source_lines,
        },
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            source,
        } => RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            _source: source,
            source_lines,
        },
        DiagramResult::NotInstalled {
            kind: k,
            download_url,
            install_path,
        } => RenderedSection::NotInstalled {
            kind: k,
            download_url,
            install_path,
            source_lines,
        },
    }
}

/// Delegates to the appropriate renderer per diagram kind.
pub(crate) fn dispatch_renderer(block: &DiagramBlock) -> DiagramResult {
    match block.kind {
        DiagramKind::Mermaid => mermaid_renderer::render_mermaid(block),
        DiagramKind::PlantUml => plantuml_renderer::render_plantuml(block),
        DiagramKind::DrawIo => drawio_renderer::render_drawio(block),
    }
}

/// Extracts SVG from an HTML fragment and rasterizes it.
pub fn try_rasterize(
    kind: &DiagramKind,
    source: &str,
    html: &str,
    source_lines: usize,
) -> RenderedSection {
    let Some(svg) = extract_svg(html) else {
        return RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: "Failed to extract SVG".to_string(),
            source_lines,
        };
    };
    match rasterize_svg(svg, DIAGRAM_SVG_DISPLAY_SCALE) {
        Ok(img) => RenderedSection::Image {
            svg_data: img,
            alt: format!("{kind:?} diagram"),
            source_lines,
        },
        Err(e) => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: e.to_string(),
            source_lines,
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
pub fn decode_png_to_section(
    kind: &DiagramKind,
    source: &str,
    bytes: Vec<u8>,
    source_lines: usize,
) -> RenderedSection {
    match decode_png_rgba(&bytes) {
        Ok(rasterized) => RenderedSection::Image {
            svg_data: rasterized,
            alt: format!("{kind:?} diagram"),
            source_lines,
        },
        Err(e) => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: format!("PNG decode failed: {e}"),
            source_lines,
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
