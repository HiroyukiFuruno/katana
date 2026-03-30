use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::{PresetColorData, Rgb, ThemeMode};

pub(crate) const GITHUB_LIGHT: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Light,
    Rgb {
        r: 255,
        g: 255,
        b: 255,
    },
    Rgb {
        r: 31,
        g: 35,
        b: 40,
    },
    Rgb {
        r: 9,
        g: 105,
        b: 218,
    },
)
.with_panel_bg(Rgb {
    r: 246,
    g: 248,
    b: 250,
})
.with_code_bg(Rgb {
    r: 246,
    g: 248,
    b: 250,
})
.with_text_sec(Rgb {
    r: 87,
    g: 96,
    b: 106,
})
.with_success(Rgb {
    r: 26,
    g: 127,
    b: 55,
})
.with_warning(Rgb {
    r: 191,
    g: 135,
    b: 0,
})
.with_error(Rgb {
    r: 207,
    g: 34,
    b: 46,
})
.with_border(Rgb {
    r: 216,
    g: 222,
    b: 228,
})
.with_selection(Rgb {
    r: 218,
    g: 234,
    b: 247,
})
.build();
