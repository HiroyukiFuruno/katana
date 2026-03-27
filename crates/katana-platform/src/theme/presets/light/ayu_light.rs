use crate::theme::types::*;

pub(crate) const AYU_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    system: SystemColors {
        background: Rgb {
            r: 250,
            g: 250,
            b: 250,
        },
        panel_background: Rgb {
            r: 243,
            g: 244,
            b: 245,
        },
        text: Rgb {
            r: 92,
            g: 101,
            b: 112,
        },
        text_secondary: Rgb {
            r: 92,
            g: 103,
            b: 115,
        },
        success_text: Rgb {
            r: 134,
            g: 179,
            b: 0,
        },
        error_text: Rgb {
            r: 240,
            g: 113,
            b: 120,
        },
        warning_text: Rgb {
            r: 242,
            g: 151,
            b: 24,
        },
        accent: Rgb {
            r: 255,
            g: 170,
            b: 51,
        },
        title_bar_text: Rgb {
            r: 26,
            g: 26,
            b: 25,
        },
        file_tree_text: Rgb {
            r: 57,
            g: 58,
            b: 52,
        },
        active_file_highlight: Rgba {
            r: 255,
            g: 153,
            b: 64,
            a: 40,
        },
        button_background: Rgba {
            r: 243,
            g: 244,
            b: 245,
            a: 255,
        },
        button_active_background: Rgba {
            r: 255,
            g: 153,
            b: 64,
            a: 80,
        },
        border: Rgb {
            r: 218,
            g: 218,
            b: 218,
        },
        selection: Rgb {
            r: 224,
            g: 224,
            b: 224,
        },
        splash_background: Rgb {
            r: 250,
            g: 250,
            b: 250,
        },
        splash_progress: Rgb {
            r: 255,
            g: 153,
            b: 64,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 242,
            g: 242,
            b: 242,
        },
        text: Rgb {
            r: 92,
            g: 101,
            b: 112,
        },
        line_number_text: Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        line_number_active_text: Rgb {
            r: 92,
            g: 101,
            b: 112,
        },
        current_line_background: Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 15,
        },
        hover_line_background: Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 10,
        },
        selection: Rgb {
            r: 224,
            g: 224,
            b: 224,
        },
    },
    preview: PreviewColors {
        background: Rgb {
            r: 250,
            g: 250,
            b: 250,
        },
        text: Rgb {
            r: 92,
            g: 101,
            b: 112,
        },
        warning_text: Rgb {
            r: 242,
            g: 151,
            b: 24,
        },
        border: Rgb {
            r: 218,
            g: 218,
            b: 218,
        },
        selection: Rgb {
            r: 224,
            g: 224,
            b: 224,
        },
        fullscreen_overlay: Rgba {
            r: 200,
            g: 200,
            b: 200,
            a: 200,
        },

        hover_line_background: Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 20,
        },
    },
};
