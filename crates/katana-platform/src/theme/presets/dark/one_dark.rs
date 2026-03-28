use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::{PresetColorData, Rgb, ThemeMode};

pub(crate) const ONE_DARK: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb {
        r: 40,
        g: 44,
        b: 52,
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
    r: 35,
    g: 39,
    b: 47,
})
.with_code_bg(Rgb {
    r: 50,
    g: 54,
    b: 62,
})
.with_text_sec(Rgb {
    r: 142,
    g: 142,
    b: 142,
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
