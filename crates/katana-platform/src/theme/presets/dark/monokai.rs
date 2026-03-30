use crate::theme::builder::ThemePresetBuilder;
use crate::theme::preset::PresetColorData;
use crate::theme::types::{Rgb, ThemeMode};

pub(crate) const MONOKAI: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb {
        r: 39,
        g: 40,
        b: 34,
    },
    Rgb {
        r: 248,
        g: 248,
        b: 242,
    },
    Rgb {
        r: 102,
        g: 217,
        b: 239,
    },
)
.with_panel_bg(Rgb {
    r: 49,
    g: 50,
    b: 44,
})
.with_code_bg(Rgb {
    r: 49,
    g: 50,
    b: 44,
})
.with_text_sec(Rgb {
    r: 178,
    g: 178,
    b: 172,
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
    r: 73,
    g: 72,
    b: 62,
})
.with_selection(Rgb {
    r: 73,
    g: 72,
    b: 62,
})
.build();