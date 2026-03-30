use katana_core::markdown::svg_rasterize::RasterizedSvg;

pub(crate) const DIAGRAM_SVG_DISPLAY_SCALE: f32 = 1.5;

pub(crate) const RENDER_POLL_INTERVAL_MS: u64 = 50;

#[derive(Debug, Clone)]
pub enum RenderedSection {
    Markdown(String),
    Image {
        svg_data: RasterizedSvg,
        alt: String,
        source_lines: usize,
    },
    LocalImage {
        path: std::path::PathBuf,
        alt: String,
        source_lines: usize,
    },
    Error {
        kind: String,
        _source: String,
        message: String,
        source_lines: usize,
    },
    CommandNotFound {
        tool_name: String,
        install_hint: String,
        _source: String,
        source_lines: usize,
    },
    NotInstalled {
        kind: String,
        download_url: String,
        install_path: std::path::PathBuf,
        source_lines: usize,
    },
    Pending {
        kind: String,
        source: String,
        source_lines: usize,
    },
}

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub dest: std::path::PathBuf,
}