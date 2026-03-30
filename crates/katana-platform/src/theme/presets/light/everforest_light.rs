use crate::theme::builder::ThemePresetBuilder;
use crate::theme::preset::PresetColorData;
use crate::theme::types::{Rgb, ThemeMode};

pub(crate) const EVERFOREST_LIGHT: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Light,
    Rgb {
        r: 253,
        g: 246,
        b: 227,
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
    r: 244,
    g: 240,
    b: 217,
})
.with_code_bg(Rgb {
    r: 243,
    g: 236,
    b: 217,
})
.with_text_sec(Rgb {
    r: 147,
    g: 159,
    b: 145,
})
.with_success(Rgb {
    r: 141,
    g: 161,
    b: 1,
})
.with_warning(Rgb {
    r: 223,
    g: 160,
    b: 0,
})
.with_error(Rgb {
    r: 248,
    g: 85,
    b: 82,
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