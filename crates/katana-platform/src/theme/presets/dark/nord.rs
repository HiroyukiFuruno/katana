use crate::theme::types::*;

pub(crate) const NORD: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    system: SystemColors {
        background: Rgb {
            r: 46,
            g: 52,
            b: 64,
        },
        panel_background: Rgb {
            r: 59,
            g: 66,
            b: 82,
        },
        text: Rgb {
            r: 216,
            g: 222,
            b: 233,
        },
        text_secondary: Rgb {
            r: 146,
            g: 152,
            b: 163,
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
            r: 136,
            g: 192,
            b: 208,
        },
        title_bar_text: Rgb {
            r: 216,
            g: 222,
            b: 233,
        },
        file_tree_text: Rgb {
            r: 146,
            g: 152,
            b: 163,
        },
        active_file_highlight: Rgba {
            r: 136,
            g: 192,
            b: 208,
            a: 30,
        },
        button_background: Rgba {
            r: 59,
            g: 66,
            b: 82,
            a: 255,
        },
        button_active_background: Rgba {
            r: 136,
            g: 192,
            b: 208,
            a: 80,
        },
        border: Rgb {
            r: 67,
            g: 76,
            b: 94,
        },
        selection: Rgb {
            r: 67,
            g: 76,
            b: 94,
        },
        splash_background: Rgb {
            r: 46,
            g: 52,
            b: 64,
        },
        splash_progress: Rgb {
            r: 136,
            g: 192,
            b: 208,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 59,
            g: 66,
            b: 82,
        },
        text: Rgb {
            r: 216,
            g: 222,
            b: 233,
        },
        line_number_text: Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        line_number_active_text: Rgb {
            r: 216,
            g: 222,
            b: 233,
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
            r: 67,
            g: 76,
            b: 94,
        },
    },
    preview: PreviewColors {
        background: Rgb {
            r: 46,
            g: 52,
            b: 64,
        },
        text: Rgb {
            r: 216,
            g: 222,
            b: 233,
        },
        warning_text: Rgb {
            r: 255,
            g: 203,
            b: 107,
        },
        border: Rgb {
            r: 67,
            g: 76,
            b: 94,
        },
        selection: Rgb {
            r: 67,
            g: 76,
            b: 94,
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
