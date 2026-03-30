use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::{PresetColorData, Rgb, ThemeMode};

pub(crate) const GITHUB_DARK: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb {
        r: 13,
        g: 17,
        b: 23,
    },
    Rgb {
        r: 201,
        g: 209,
        b: 217,
    },
    Rgb {
        r: 88,
        g: 166,
        b: 255,
    },
)
.with_panel_bg(Rgb {
    r: 22,
    g: 27,
    b: 34,
})
.with_code_bg(Rgb {
    r: 22,
    g: 27,
    b: 34,
})
.with_text_sec(Rgb {
    r: 131,
    g: 139,
    b: 147,
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
    r: 48,
    g: 54,
    b: 61,
})
.with_selection(Rgb {
    r: 23,
    g: 74,
    b: 130,
})
.build();
