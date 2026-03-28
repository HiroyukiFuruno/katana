use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::*;

pub(crate) const KATANA_DARK: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb {
        r: 30,
        g: 30,
        b: 30,
    },
    Rgb {
        r: 212,
        g: 212,
        b: 212,
    },
    Rgb {
        r: 86,
        g: 156,
        b: 214,
    },
)
.with_panel_bg(Rgb {
    r: 37,
    g: 37,
    b: 38,
})
.with_code_bg(Rgb {
    r: 40,
    g: 40,
    b: 40,
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
    r: 38,
    g: 79,
    b: 120,
})
.build();
