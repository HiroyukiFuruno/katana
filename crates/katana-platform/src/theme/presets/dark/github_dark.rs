use crate::theme::types::*;

pub(crate) const GITHUB_DARK: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    system: SystemColors {
        background: Rgb {
            r: 13,
            g: 17,
            b: 23,
        },
        panel_background: Rgb {
            r: 22,
            g: 27,
            b: 34,
        },
        text: Rgb {
            r: 201,
            g: 209,
            b: 217,
        },
        text_secondary: Rgb {
            r: 131,
            g: 139,
            b: 147,
        },
        success_text: Rgb {
            r: 195,
            g: 232,
            b: 141,
        },
        error_text: Rgb {
            r: 240,
            g: 113,
            b: 120,
        },
        warning_text: Rgb {
            r: 255,
            g: 203,
            b: 107,
        },
        accent: Rgb {
            r: 88,
            g: 166,
            b: 255,
        },
        title_bar_text: Rgb {
            r: 201,
            g: 209,
            b: 217,
        },
        file_tree_text: Rgb {
            r: 131,
            g: 139,
            b: 147,
        },
        active_file_highlight: Rgba {
            r: 88,
            g: 166,
            b: 255,
            a: 30,
        },
        button_background: Rgba {
            r: 22,
            g: 27,
            b: 34,
            a: 255,
        },
        button_active_background: Rgba {
            r: 88,
            g: 166,
            b: 255,
            a: 80,
        },
        border: Rgb {
            r: 48,
            g: 54,
            b: 61,
        },
        selection: Rgb {
            r: 23,
            g: 74,
            b: 130,
        },
        splash_background: Rgb {
            r: 13,
            g: 17,
            b: 23,
        },
        splash_progress: Rgb {
            r: 88,
            g: 166,
            b: 255,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 22,
            g: 27,
            b: 34,
        },
        text: Rgb {
            r: 201,
            g: 209,
            b: 217,
        },
        line_number_text: Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        line_number_active_text: Rgb {
            r: 201,
            g: 209,
            b: 217,
        },
        current_line_background: Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 50,
        },
        hover_line_background: Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 50,
        },
        selection: Rgb {
            r: 23,
            g: 74,
            b: 130,
        },
    },
    preview: PreviewColors {
        background: Rgb {
            r: 13,
            g: 17,
            b: 23,
        },
        text: Rgb {
            r: 201,
            g: 209,
            b: 217,
        },
        warning_text: Rgb {
            r: 255,
            g: 203,
            b: 107,
        },
        border: Rgb {
            r: 48,
            g: 54,
            b: 61,
        },
        selection: Rgb {
            r: 23,
            g: 74,
            b: 130,
        },
        fullscreen_overlay: Rgba {
            r: 200,
            g: 200,
            b: 200,
            a: 200,
        },

        hover_line_background: Rgba {
            r: 255,
            g: 255,
            b: 255,
            a: 50,
        },
    },
};
