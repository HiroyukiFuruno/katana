use crate::theme::builder::ThemePresetBuilder;
use crate::theme::types::*;

pub(crate) const SYNTHWAVE_84: PresetColorData = ThemePresetBuilder::new(
    ThemeMode::Dark,
    Rgb {
        r: 43,
        g: 33,
        b: 58,
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
    r: 36,
    g: 27,
    b: 47,
})
.with_code_bg(Rgb {
    r: 48,
    g: 45,
    b: 63,
})
.with_text_sec(Rgb {
    r: 132,
    g: 139,
    b: 189,
})
.with_success(Rgb {
    r: 114,
    g: 241,
    b: 184,
})
.with_warning(Rgb {
    r: 254,
    g: 222,
    b: 93,
})
.with_error(Rgb {
    r: 254,
    g: 68,
    b: 80,
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
