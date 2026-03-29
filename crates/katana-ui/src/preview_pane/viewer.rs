use eframe::egui;

// ─────────────────────────────────────────────
// Viewer Controls — State management for pan/zoom on images and diagrams
// ─────────────────────────────────────────────

/// Zoom increment per button click.
pub(crate) const VIEWER_ZOOM_STEP: f32 = 0.25;
/// Minimum zoom level (25%).
pub(crate) const VIEWER_ZOOM_MIN: f32 = 0.25;
/// Maximum zoom level (400%).
pub(crate) const VIEWER_ZOOM_MAX: f32 = 4.0;
/// Pan offset (in logical pixels) per button click.
pub(crate) const VIEWER_PAN_STEP: f32 = 50.0;

/// Per-image/diagram viewer state (zoom level and pan offset).
#[derive(Clone, PartialEq)]
pub struct ViewerState {
    /// Current zoom factor (1.0 = 100%).
    pub zoom: f32,
    /// Current pan offset in logical pixels.
    pub pan: egui::Vec2,
    /// Cached texture handle to avoid re-uploading the image to the GPU every frame.
    pub texture: Option<egui::TextureHandle>,
}

impl std::fmt::Debug for ViewerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewerState")
            .field("zoom", &self.zoom)
            .field("pan", &self.pan)
            .field("texture", &self.texture.as_ref().map(|t| t.id()))
            .finish()
    }
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            texture: None,
        }
    }
}

impl ViewerState {
    /// Zoom in by one step, clamped to `VIEWER_ZOOM_MAX`.
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + VIEWER_ZOOM_STEP).min(VIEWER_ZOOM_MAX);
    }

    /// Zoom out by one step, clamped to `VIEWER_ZOOM_MIN`.
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - VIEWER_ZOOM_STEP).max(VIEWER_ZOOM_MIN);
    }

    /// Pan by the given delta (in logical pixels).
    pub fn pan_by(&mut self, delta: egui::Vec2) {
        self.pan += delta;
    }

    /// Pan up by one step.
    pub fn pan_up(&mut self) {
        self.pan_by(egui::vec2(0.0, -VIEWER_PAN_STEP));
    }

    /// Pan down by one step.
    pub fn pan_down(&mut self) {
        self.pan_by(egui::vec2(0.0, VIEWER_PAN_STEP));
    }

    /// Pan left by one step.
    pub fn pan_left(&mut self) {
        self.pan_by(egui::vec2(-VIEWER_PAN_STEP, 0.0));
    }

    /// Pan right by one step.
    pub fn pan_right(&mut self) {
        self.pan_by(egui::vec2(VIEWER_PAN_STEP, 0.0));
    }

    /// Reset zoom and pan to defaults.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
