use crate::theme::builder::ThemePresetBuilder;
use crate::theme::preset::PresetColorData;
use crate::theme::types::{Rgb, ThemeMode};

pub(crate) const GRUVBOX_LIGHT: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Light,
    Rgb {
        r: 251,
        g: 241,
        b: 199,
    },
    Rgb {
        r: 60,
        g: 56,
        b: 54,
    },
    Rgb {
        r: 69,
        g: 133,
        b: 136,
    },
)
.with_panel_bg(Rgb {
    r: 235,
    g: 219,
    b: 178,
})
.with_code_bg(Rgb {
    r: 235,
    g: 219,
    b: 178,
})
.with_text_sec(Rgb {
    r: 130,
    g: 126,
    b: 124,
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
    r: 213,
    g: 196,
    b: 161,
})
.with_selection(Rgb {
    r: 213,
    g: 196,
    b: 161,
})
.build();