use crate::theme::builder::ThemePresetBuilder;
use crate::theme::preset::PresetColorData;
use crate::theme::types::{Rgb, ThemeMode};

pub(crate) const AYU_LIGHT: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Light,
    Rgb {
        r: 250,
        g: 250,
        b: 250,
    },
    Rgb {
        r: 92,
        g: 101,
        b: 112,
    },
    Rgb {
        r: 255,
        g: 170,
        b: 51,
    },
)
.with_panel_bg(Rgb {
    r: 243,
    g: 244,
    b: 245,
})
.with_code_bg(Rgb {
    r: 242,
    g: 242,
    b: 242,
})
.with_text_sec(Rgb {
    r: 92,
    g: 103,
    b: 115,
})
.with_success(Rgb {
    r: 134,
    g: 179,
    b: 0,
})
.with_warning(Rgb {
    r: 242,
    g: 151,
    b: 24,
})
.with_error(Rgb {
    r: 240,
    g: 113,
    b: 120,
})
.with_border(Rgb {
    r: 218,
    g: 218,
    b: 218,
})
.with_selection(Rgb {
    r: 224,
    g: 224,
    b: 224,
})
.build();
