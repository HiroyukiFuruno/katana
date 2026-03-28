use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::*;

pub(crate) const TOKYO_NIGHT: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb {
        r: 31,
        g: 35,
        b: 53,
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
    r: 26,
    g: 27,
    b: 38,
})
.with_code_bg(Rgb {
    r: 36,
    g: 37,
    b: 48,
})
.with_text_sec(Rgb {
    r: 86,
    g: 95,
    b: 137,
})
.with_success(Rgb {
    r: 158,
    g: 206,
    b: 106,
})
.with_warning(Rgb {
    r: 224,
    g: 175,
    b: 104,
})
.with_error(Rgb {
    r: 219,
    g: 75,
    b: 75,
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
