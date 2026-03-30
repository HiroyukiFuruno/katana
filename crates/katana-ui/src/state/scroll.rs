#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum ScrollSource {
    #[default]
    Neither,
    Editor,
    Preview,
}

pub struct ScrollState {
    pub fraction: f32,
    pub source: ScrollSource,
    pub editor_max: f32,
    pub preview_max: f32,
    pub active_editor_line: Option<usize>,
    pub scroll_to_line: Option<usize>,
    pub hovered_preview_lines: Vec<std::ops::Range<usize>>,
    pub sync_override: Option<bool>,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            fraction: 0.0,
            source: ScrollSource::Neither,
            editor_max: 0.0,
            preview_max: 0.0,
            active_editor_line: None,
            scroll_to_line: None,
            hovered_preview_lines: Vec::new(),
            sync_override: None,
        }
    }
}