use crate::theme::builder::ThemePresetBuilder;
use crate::theme::preset::PresetColorData;
use crate::theme::types::{Rgb, ThemeMode};

pub(crate) const DRACULA: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb {
        r: 40,
        g: 42,
        b: 54,
    },
    Rgb {
        r: 248,
        g: 248,
        b: 242,
    },
    Rgb {
        r: 139,
        g: 233,
        b: 253,
    },
)
.with_panel_bg(Rgb {
    r: 44,
    g: 44,
    b: 58,
})
.with_code_bg(Rgb {
    r: 50,
    g: 52,
    b: 66,
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
    r: 68,
    g: 71,
    b: 90,
})
.with_selection(Rgb {
    r: 68,
    g: 71,
    b: 90,
})
.build();
