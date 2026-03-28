use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::*;

pub(crate) const SOLARIZED_LIGHT: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Light,
    Rgb {
        r: 253,
        g: 246,
        b: 227,
    },
    Rgb {
        r: 101,
        g: 123,
        b: 131,
    },
    Rgb {
        r: 38,
        g: 139,
        b: 210,
    },
)
.with_panel_bg(Rgb {
    r: 238,
    g: 232,
    b: 213,
})
.with_code_bg(Rgb {
    r: 238,
    g: 232,
    b: 213,
})
.with_text_sec(Rgb {
    r: 88,
    g: 110,
    b: 117,
})
.with_success(Rgb {
    r: 133,
    g: 153,
    b: 0,
})
.with_warning(Rgb {
    r: 181,
    g: 137,
    b: 0,
})
.with_error(Rgb {
    r: 220,
    g: 50,
    b: 47,
})
.with_border(Rgb {
    r: 238,
    g: 232,
    b: 213,
})
.with_selection(Rgb {
    r: 238,
    g: 232,
    b: 213,
})
.build();
