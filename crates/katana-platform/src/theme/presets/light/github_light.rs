use crate::theme::types::*;

pub(crate) const GITHUB_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    system: SystemColors {
        background: Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
        panel_background: Rgb {
            r: 246,
            g: 248,
            b: 250,
        },
        text: Rgb {
            r: 31,
            g: 35,
            b: 40,
        },
        text_secondary: Rgb {
            r: 87,
            g: 96,
            b: 106,
        },
        success_text: Rgb {
            r: 26,
            g: 127,
            b: 55,
        },
        error_text: Rgb {
            r: 207,
            g: 34,
            b: 46,
        },
        warning_text: Rgb {
            r: 191,
            g: 135,
            b: 0,
        },
        accent: Rgb {
            r: 9,
            g: 105,
            b: 218,
        },
        title_bar_text: Rgb {
            r: 36,
            g: 41,
            b: 47,
        },
        file_tree_text: Rgb {
            r: 36,
            g: 41,
            b: 47,
        },
        active_file_highlight: Rgba {
            r: 9,
            g: 105,
            b: 218,
            a: 25,
        },
        button_background: Rgba {
            r: 246,
            g: 248,
            b: 250,
            a: 255,
        },
        button_active_background: Rgba {
            r: 9,
            g: 105,
            b: 218,
            a: 50,
        },
        border: Rgb {
            r: 216,
            g: 222,
            b: 228,
        },
        selection: Rgb {
            r: 218,
            g: 234,
            b: 247,
        },
        splash_background: Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
        splash_progress: Rgb {
            r: 9,
            g: 105,
            b: 218,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 246,
            g: 248,
            b: 250,
        },
        text: Rgb {
            r: 31,
            g: 35,
            b: 40,
        },
        line_number_text: Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        line_number_active_text: Rgb {
            r: 31,
            g: 35,
            b: 40,
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
            r: 218,
            g: 234,
            b: 247,
        },
    },
    preview: PreviewColors {
        background: Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
        text: Rgb {
            r: 31,
            g: 35,
            b: 40,
        },
        warning_text: Rgb {
            r: 191,
            g: 135,
            b: 0,
        },
        border: Rgb {
            r: 216,
            g: 222,
            b: 228,
        },
        selection: Rgb {
            r: 218,
            g: 234,
            b: 247,
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
