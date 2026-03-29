//! Shared color palette and alpha constants for theme builders.

use crate::theme::types::Rgb;

pub(crate) const DEFAULT_ACTIVE_FILE_HIGHLIGHT_ALPHA: u8 = 30;
pub(crate) const DEFAULT_BUTTON_ACTIVE_ALPHA: u8 = 80;
pub(crate) const DEFAULT_CODE_CURRENT_LINE_DARK_ALPHA: u8 = 50;
pub(crate) const DEFAULT_CODE_CURRENT_LINE_LIGHT_ALPHA: u8 = 15;
pub(crate) const DEFAULT_HOVER_LINE_HIGHLIGHT_ALPHA: u8 = 25;

pub(crate) const DEFAULT_TEXT_SECONDARY_DARKEN: u8 = 70;
pub(crate) const DEFAULT_TEXT_SECONDARY_LIGHTEN: u8 = 50;
pub(crate) const DEFAULT_BORDER_DARKEN: u8 = 25;
pub(crate) const DEFAULT_BORDER_LIGHTEN: u8 = 25;
pub(crate) const DEFAULT_SELECTION_DARKEN: u8 = 15;
pub(crate) const DEFAULT_SELECTION_LIGHTEN: u8 = 45;
pub(crate) const DEFAULT_PANEL_BG_DARKEN_DARK: u8 = 5;
pub(crate) const DEFAULT_PANEL_BG_DARKEN_LIGHT: u8 = 10;
pub(crate) const DEFAULT_CODE_BG_LIGHTEN_DARK: u8 = 5;
pub(crate) const DEFAULT_CODE_BG_DARKEN_LIGHT: u8 = 5;
pub(crate) const DEFAULT_BUTTON_BACKGROUND_ALPHA: u8 = 255;

pub(crate) const DEFAULT_SUCCESS_DARK: Rgb = Rgb {
    r: 153,
    g: 204,
    b: 153,
};
pub(crate) const DEFAULT_SUCCESS_LIGHT: Rgb = Rgb {
    r: 60,
    g: 130,
    b: 60,
};
pub(crate) const DEFAULT_WARNING_DARK: Rgb = Rgb {
    r: 255,
    g: 204,
    b: 102,
};
pub(crate) const DEFAULT_WARNING_LIGHT: Rgb = Rgb {
    r: 160,
    g: 110,
    b: 20,
};
pub(crate) const DEFAULT_ERROR_DARK: Rgb = Rgb {
    r: 242,
    g: 119,
    b: 122,
};
pub(crate) const DEFAULT_ERROR_LIGHT: Rgb = Rgb {
    r: 180,
    g: 40,
    b: 40,
};
