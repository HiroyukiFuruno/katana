use crate::theme::builder::ThemePresetBuilder;
use crate::theme::preset::PresetColorData;
use crate::theme::types::{Rgb, ThemeMode};

pub(crate) const ROSE_PINE: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb {
        r: 25,
        g: 23,
        b: 36,
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
    r: 31,
    g: 29,
    b: 46,
})
.with_code_bg(Rgb {
    r: 35,
    g: 33,
    b: 46,
})
.with_text_sec(Rgb {
    r: 144,
    g: 140,
    b: 170,
})
.with_success(Rgb {
    r: 156,
    g: 207,
    b: 216,
})
.with_warning(Rgb {
    r: 246,
    g: 193,
    b: 119,
})
.with_error(Rgb {
    r: 235,
    g: 111,
    b: 146,
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