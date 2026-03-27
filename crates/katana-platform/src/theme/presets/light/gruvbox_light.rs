use crate::theme::types::*;

pub(crate) const GRUVBOX_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    system: SystemColors {
        background: Rgb {
            r: 251,
            g: 241,
            b: 199,
        },
        panel_background: Rgb {
            r: 235,
            g: 219,
            b: 178,
        },
        text: Rgb {
            r: 60,
            g: 56,
            b: 54,
        },
        text_secondary: Rgb {
            r: 130,
            g: 126,
            b: 124,
        },
        success_text: Rgb {
            r: 64,
            g: 160,
            b: 43,
        },
        error_text: Rgb {
            r: 210,
            g: 15,
            b: 57,
        },
        warning_text: Rgb {
            r: 223,
            g: 142,
            b: 29,
        },
        accent: Rgb {
            r: 69,
            g: 133,
            b: 136,
        },
        title_bar_text: Rgb {
            r: 60,
            g: 56,
            b: 54,
        },
        file_tree_text: Rgb {
            r: 130,
            g: 126,
            b: 124,
        },
        active_file_highlight: Rgba {
            r: 69,
            g: 133,
            b: 136,
            a: 40,
        },
        button_background: Rgba {
            r: 235,
            g: 219,
            b: 178,
            a: 255,
        },
        button_active_background: Rgba {
            r: 69,
            g: 133,
            b: 136,
            a: 100,
        },
        border: Rgb {
            r: 213,
            g: 196,
            b: 161,
        },
        selection: Rgb {
            r: 213,
            g: 196,
            b: 161,
        },
        splash_background: Rgb {
            r: 251,
            g: 241,
            b: 199,
        },
        splash_progress: Rgb {
            r: 69,
            g: 133,
            b: 136,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 235,
            g: 219,
            b: 178,
        },
        text: Rgb {
            r: 60,
            g: 56,
            b: 54,
        },
        line_number_text: Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        line_number_active_text: Rgb {
            r: 60,
            g: 56,
            b: 54,
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
            r: 213,
            g: 196,
            b: 161,
        },
    },
    preview: PreviewColors {
        background: Rgb {
            r: 251,
            g: 241,
            b: 199,
        },
        text: Rgb {
            r: 60,
            g: 56,
            b: 54,
        },
        warning_text: Rgb {
            r: 223,
            g: 142,
            b: 29,
        },
        border: Rgb {
            r: 213,
            g: 196,
            b: 161,
        },
        selection: Rgb {
            r: 213,
            g: 196,
            b: 161,
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
