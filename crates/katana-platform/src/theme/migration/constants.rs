use crate::theme::types::{Rgb, Rgba};

pub(crate) const ERROR_R: u8 = 0xFF;
pub(crate) const ERROR_G: u8 = 0x55;
pub(crate) const ERROR_B: u8 = 0x55;

/* WHY: ── Migration fallback color constants ──────────────────────────
Default values used when converting from legacy format to the new 3-layer structure.
Defined in dark/light pairs for use in the "is_dark" branch. */
pub(crate) const LEGACY_SUCCESS_DARK: Rgb = Rgb {
    r: 80,
    g: 200,
    b: 80,
};
pub(crate) const LEGACY_SUCCESS_LIGHT: Rgb = Rgb {
    r: 20,
    g: 160,
    b: 20,
};

pub(crate) const LEGACY_BUTTON_BG_DARK: Rgba = Rgba {
    r: 80,
    g: 80,
    b: 80,
    a: 50,
};
pub(crate) const LEGACY_BUTTON_BG_LIGHT: Rgba = Rgba {
    r: 160,
    g: 160,
    b: 160,
    a: 50,
};

pub(crate) const LEGACY_BUTTON_ACTIVE_DARK: Rgba = Rgba {
    r: 120,
    g: 120,
    b: 120,
    a: 100,
};
pub(crate) const LEGACY_BUTTON_ACTIVE_LIGHT: Rgba = Rgba {
    r: 200,
    g: 200,
    b: 200,
    a: 100,
};

pub(crate) const LEGACY_LINE_NUMBER_DARK: Rgb = Rgb {
    r: 100,
    g: 100,
    b: 100,
};
pub(crate) const LEGACY_LINE_NUMBER_LIGHT: Rgb = Rgb {
    r: 160,
    g: 160,
    b: 160,
};

pub(crate) const LEGACY_CURRENT_LINE_DARK: Rgba = Rgba {
    r: 255,
    g: 255,
    b: 255,
    a: 15,
};
pub(crate) const LEGACY_CURRENT_LINE_LIGHT: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 15,
};

pub(crate) const LEGACY_HOVER_LINE_DARK: Rgba = Rgba {
    r: 255,
    g: 255,
    b: 255,
    a: 10,
};
pub(crate) const LEGACY_HOVER_LINE_LIGHT: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 10,
};

pub(crate) const LEGACY_DEFAULT_WARNING: Rgb = Rgb {
    r: 255,
    g: 140,
    b: 0,
};
pub(crate) const LEGACY_DEFAULT_BORDER: Rgb = Rgb {
    r: 60,
    g: 60,
    b: 60,
};
pub(crate) const LEGACY_DEFAULT_SELECTION: Rgb = Rgb {
    r: 80,
    g: 80,
    b: 80,
};
pub(crate) const LEGACY_DEFAULT_CODE_BG: Rgb = Rgb {
    r: 30,
    g: 30,
    b: 30,
};
pub(crate) const LEGACY_DEFAULT_PREVIEW_BG: Rgb = Rgb {
    r: 35,
    g: 35,
    b: 35,
};