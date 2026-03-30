use crate::theme::builder::ThemePresetBuilder;
use crate::theme::preset::PresetColorData;
use crate::theme::types::{Rgb, ThemeMode};

pub(crate) const PAPERCOLOR_LIGHT: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Light,
    Rgb {
        r: 238,
        g: 238,
        b: 238,
    },
    Rgb {
        r: 40,
        g: 40,
        b: 40,
    },
    Rgb {
        r: 0,
        g: 100,
        b: 200,
    },
)
.with_panel_bg(Rgb {
    r: 243,
    g: 243,
    b: 243,
})
.with_code_bg(Rgb {
    r: 228,
    g: 228,
    b: 228,
})
.with_text_sec(Rgb {
    r: 110,
    g: 110,
    b: 110,
})
.with_success(Rgb {
    r: 64,
    g: 160,
    b: 43,
})
.with_warning(Rgb {
    r: 223,
    g: 142,
    b: 29,
})
.with_error(Rgb {
    r: 210,
    g: 15,
    b: 57,
})
.with_border(Rgb {
    r: 200,
    g: 200,
    b: 200,
})
.with_selection(Rgb {
    r: 200,
    g: 220,
    b: 255,
})
.build();