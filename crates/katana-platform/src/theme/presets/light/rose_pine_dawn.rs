use crate::theme::builder::ThemePresetBuilder;
use crate::theme::preset::PresetColorData;
use crate::theme::types::{Rgb, ThemeMode};

pub(crate) const ROSE_PINE_DAWN: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Light,
    Rgb {
        r: 250,
        g: 244,
        b: 237,
    },
    Rgb {
        r: 40,
        g: 40,
        b: 40,
    },
    Rgb {
        r: 0,
        g: 100,
        b: 200,
    },
)
.with_panel_bg(Rgb {
    r: 255,
    g: 250,
    b: 243,
})
.with_code_bg(Rgb {
    r: 240,
    g: 234,
    b: 227,
})
.with_text_sec(Rgb {
    r: 121,
    g: 117,
    b: 147,
})
.with_success(Rgb {
    r: 40,
    g: 105,
    b: 131,
})
.with_warning(Rgb {
    r: 215,
    g: 130,
    b: 126,
})
.with_error(Rgb {
    r: 180,
    g: 99,
    b: 122,
})
.with_border(Rgb {
    r: 200,
    g: 200,
    b: 200,
})
.with_selection(Rgb {
    r: 200,
    g: 220,
    b: 255,
})
.build();
