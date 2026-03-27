use crate::theme::types::*;

pub(crate) const MONOKAI: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    system: SystemColors {
        background: Rgb {
            r: 39,
            g: 40,
            b: 34,
        },
        panel_background: Rgb {
            r: 49,
            g: 50,
            b: 44,
        },
        text: Rgb {
            r: 248,
            g: 248,
            b: 242,
        },
        text_secondary: Rgb {
            r: 178,
            g: 178,
            b: 172,
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
            r: 102,
            g: 217,
            b: 239,
        },
        title_bar_text: Rgb {
            r: 248,
            g: 248,
            b: 242,
        },
        file_tree_text: Rgb {
            r: 178,
            g: 178,
            b: 172,
        },
        active_file_highlight: Rgba {
            r: 102,
            g: 217,
            b: 239,
            a: 30,
        },
        button_background: Rgba {
            r: 49,
            g: 50,
            b: 44,
            a: 255,
        },
        button_active_background: Rgba {
            r: 102,
            g: 217,
            b: 239,
            a: 80,
        },
        border: Rgb {
            r: 73,
            g: 72,
            b: 62,
        },
        selection: Rgb {
            r: 73,
            g: 72,
            b: 62,
        },
        splash_background: Rgb {
            r: 39,
            g: 40,
            b: 34,
        },
        splash_progress: Rgb {
            r: 102,
            g: 217,
            b: 239,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 49,
            g: 50,
            b: 44,
        },
        text: Rgb {
            r: 248,
            g: 248,
            b: 242,
        },
        line_number_text: Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        line_number_active_text: Rgb {
            r: 248,
            g: 248,
            b: 242,
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
            r: 73,
            g: 72,
            b: 62,
        },
    },
    preview: PreviewColors {
        background: Rgb {
            r: 39,
            g: 40,
            b: 34,
        },
        text: Rgb {
            r: 248,
            g: 248,
            b: 242,
        },
        warning_text: Rgb {
            r: 255,
            g: 203,
            b: 107,
        },
        border: Rgb {
            r: 73,
            g: 72,
            b: 62,
        },
        selection: Rgb {
            r: 73,
            g: 72,
            b: 62,
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
