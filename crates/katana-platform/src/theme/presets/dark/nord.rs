use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::{PresetColorData, Rgb, ThemeMode};

pub(crate) const NORD: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb {
        r: 46,
        g: 52,
        b: 64,
    },
    Rgb {
        r: 216,
        g: 222,
        b: 233,
    },
    Rgb {
        r: 136,
        g: 192,
        b: 208,
    },
)
.with_panel_bg(Rgb {
    r: 59,
    g: 66,
    b: 82,
})
.with_code_bg(Rgb {
    r: 59,
    g: 66,
    b: 82,
})
.with_text_sec(Rgb {
    r: 146,
    g: 152,
    b: 163,
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
    r: 67,
    g: 76,
    b: 94,
})
.with_selection(Rgb {
    r: 67,
    g: 76,
    b: 94,
})
.build();
