use crate::theme::builder::ThemePresetBuilder;
use crate::theme::preset::PresetColorData;
use crate::theme::types::{Rgb, ThemeMode};

pub(crate) const NIGHT_OWL: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb { r: 1, g: 22, b: 39 },
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
.with_panel_bg(Rgb { r: 0, g: 17, b: 34 })
.with_code_bg(Rgb {
    r: 11,
    g: 32,
    b: 49,
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