use eframe::egui;


pub(crate) const VIEWER_ZOOM_STEP: f32 = 0.25;
pub(crate) const VIEWER_ZOOM_MIN: f32 = 0.25;
pub(crate) const VIEWER_ZOOM_MAX: f32 = 4.0;
pub(crate) const VIEWER_PAN_STEP: f32 = 50.0;

#[derive(Clone, PartialEq)]
pub struct ViewerState {
    pub zoom: f32,
    pub pan: egui::Vec2,
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
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + VIEWER_ZOOM_STEP).min(VIEWER_ZOOM_MAX);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - VIEWER_ZOOM_STEP).max(VIEWER_ZOOM_MIN);
    }

    pub fn pan_by(&mut self, delta: egui::Vec2) {
        self.pan += delta;
    }

    pub fn pan_up(&mut self) {
        self.pan_by(egui::vec2(0.0, -VIEWER_PAN_STEP));
    }

    pub fn pan_down(&mut self) {
        self.pan_by(egui::vec2(0.0, VIEWER_PAN_STEP));
    }

    pub fn pan_left(&mut self) {
        self.pan_by(egui::vec2(-VIEWER_PAN_STEP, 0.0));
    }

    pub fn pan_right(&mut self) {
        self.pan_by(egui::vec2(VIEWER_PAN_STEP, 0.0));
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}