use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::*;

pub(crate) const ONE_LIGHT: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Light,
    Rgb {
        r: 250,
        g: 250,
        b: 250,
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
    r: 240,
    g: 240,
    b: 241,
})
.with_code_bg(Rgb {
    r: 240,
    g: 240,
    b: 240,
})
.with_text_sec(Rgb {
    r: 160,
    g: 161,
    b: 167,
})
.with_success(Rgb {
    r: 80,
    g: 161,
    b: 79,
})
.with_warning(Rgb {
    r: 193,
    g: 132,
    b: 1,
})
.with_error(Rgb {
    r: 228,
    g: 86,
    b: 73,
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
