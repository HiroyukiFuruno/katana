use crate::theme::types::*;

pub(crate) const ANDROMEDA: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    system: SystemColors {
        background: Rgb {
            r: 35,
            g: 38,
            b: 46,
        },
        panel_background: Rgb {
            r: 30,
            g: 33,
            b: 41,
        },
        text: Rgb {
            r: 212,
            g: 212,
            b: 212,
        },
        text_secondary: Rgb {
            r: 142,
            g: 142,
            b: 142,
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
            r: 100,
            g: 150,
            b: 255,
        },
        title_bar_text: Rgb {
            r: 212,
            g: 212,
            b: 212,
        },
        file_tree_text: Rgb {
            r: 142,
            g: 142,
            b: 142,
        },
        active_file_highlight: Rgba {
            r: 100,
            g: 150,
            b: 255,
            a: 30,
        },
        button_background: Rgba {
            r: 30,
            g: 33,
            b: 41,
            a: 255,
        },
        button_active_background: Rgba {
            r: 100,
            g: 150,
            b: 255,
            a: 80,
        },
        border: Rgb {
            r: 60,
            g: 60,
            b: 60,
        },
        selection: Rgb {
            r: 80,
            g: 80,
            b: 100,
        },
        splash_background: Rgb {
            r: 35,
            g: 38,
            b: 46,
        },
        splash_progress: Rgb {
            r: 100,
            g: 150,
            b: 255,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 45,
            g: 48,
            b: 56,
        },
        text: Rgb {
            r: 212,
            g: 212,
            b: 212,
        },
        line_number_text: Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        line_number_active_text: Rgb {
            r: 212,
            g: 212,
            b: 212,
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
            r: 80,
            g: 80,
            b: 100,
        },
    },
    preview: PreviewColors {
        background: Rgb {
            r: 35,
            g: 38,
            b: 46,
        },
        text: Rgb {
            r: 212,
            g: 212,
            b: 212,
        },
        warning_text: Rgb {
            r: 255,
            g: 203,
            b: 107,
        },
        border: Rgb {
            r: 60,
            g: 60,
            b: 60,
        },
        selection: Rgb {
            r: 80,
            g: 80,
            b: 100,
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
