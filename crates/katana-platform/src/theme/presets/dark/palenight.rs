use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::*;

pub(crate) const PALENIGHT: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb {
        r: 41,
        g: 45,
        b: 62,
    },
    Rgb {
        r: 212,
        g: 212,
        b: 212,
    },
    Rgb {
        r: 100,
        g: 150,
        b: 255,
    },
)
.with_panel_bg(Rgb {
    r: 27,
    g: 30,
    b: 43,
})
.with_code_bg(Rgb {
    r: 51,
    g: 55,
    b: 72,
})
.with_text_sec(Rgb {
    r: 103,
    g: 110,
    b: 149,
})
.with_success(Rgb {
    r: 195,
    g: 232,
    b: 141,
})
.with_warning(Rgb {
    r: 255,
    g: 203,
    b: 107,
})
.with_error(Rgb {
    r: 240,
    g: 113,
    b: 120,
})
.with_border(Rgb {
    r: 60,
    g: 60,
    b: 60,
})
.with_selection(Rgb {
    r: 80,
    g: 80,
    b: 100,
})
.build();
