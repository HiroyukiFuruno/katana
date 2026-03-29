use katana_core::markdown::svg_rasterize::RasterizedSvg;

// ─────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────
/// Display scale when converting diagram SVG to pixel images.
pub(crate) const DIAGRAM_SVG_DISPLAY_SCALE: f32 = 1.5;

/// Interval (in milliseconds) used to poll background render tasks.
pub(crate) const RENDER_POLL_INTERVAL_MS: u64 = 50;

/// Rendered sections held in the UI layer.
#[derive(Debug, Clone)]
pub enum RenderedSection {
    /// Markdown text rendered by egui_commonmark.
    Markdown(String),
    /// Rasterized diagram image.
    Image {
        svg_data: RasterizedSvg,
        alt: String,
        source_lines: usize,
    },
    /// A standalone local image handled by Katana (with viewer controls).
    LocalImage {
        path: std::path::PathBuf,
        alt: String,
        source_lines: usize,
    },
    /// Rendering error (holds source and message).
    Error {
        kind: String,
        _source: String,
        message: String,
        source_lines: usize,
    },
    /// Command line tool not found (path issues, etc.).
    CommandNotFound {
        tool_name: String,
        install_hint: String,
        _source: String,
        source_lines: usize,
    },
    /// Required tool is not installed — can be downloaded from the UI.
    NotInstalled {
        kind: String,
        download_url: String,
        install_path: std::path::PathBuf,
        source_lines: usize,
    },
    /// Placeholder during background rendering.
    Pending {
        kind: String,
        source: String,
        source_lines: usize,
    },
}

/// Download request returned by the preview pane.
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub dest: std::path::PathBuf,
}
