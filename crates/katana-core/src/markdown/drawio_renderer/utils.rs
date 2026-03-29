use xmltree::Element;

pub const CANVAS_MIN_WIDTH: f64 = 400.0;

pub const CANVAS_MIN_HEIGHT: f64 = 300.0;

pub const CANVAS_EDGE_MARGIN: f64 = 20.0;

pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

pub fn attr_f64(el: &Element, name: &str) -> f64 {
    el.attributes
        .get(name)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

pub fn extract_style_value<'a>(style: &'a str, key: &str) -> Option<&'a str> {
    style.split(';').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k.trim() == key).then_some(v.trim())
    })
}

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
