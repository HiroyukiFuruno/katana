use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::*;

pub(crate) const KATANA_LIGHT: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Light,
    Rgb {
        r: 255,
        g: 255,
        b: 255,
    },
    Rgb {
        r: 36,
        g: 36,
        b: 36,
    },
    Rgb {
        r: 0,
        g: 120,
        b: 212,
    },
)
.with_panel_bg(Rgb {
    r: 243,
    g: 243,
    b: 243,
})
.with_code_bg(Rgb {
    r: 243,
    g: 243,
    b: 243,
})
.with_text_sec(Rgb {
    r: 106,
    g: 106,
    b: 106,
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
    r: 220,
    g: 220,
    b: 220,
})
.with_selection(Rgb {
    r: 173,
    g: 214,
    b: 255,
})
.build();
