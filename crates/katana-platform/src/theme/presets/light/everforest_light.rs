use crate::theme::types::*;

pub(crate) const EVERFOREST_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    system: SystemColors {
        background: Rgb {
            r: 253,
            g: 246,
            b: 227,
        },
        panel_background: Rgb {
            r: 244,
            g: 240,
            b: 217,
        },
        text: Rgb {
            r: 40,
            g: 40,
            b: 40,
        },
        text_secondary: Rgb {
            r: 147,
            g: 159,
            b: 145,
        },
        success_text: Rgb {
            r: 141,
            g: 161,
            b: 1,
        },
        error_text: Rgb {
            r: 248,
            g: 85,
            b: 82,
        },
        warning_text: Rgb {
            r: 223,
            g: 160,
            b: 0,
        },
        accent: Rgb {
            r: 0,
            g: 100,
            b: 200,
        },
        title_bar_text: Rgb {
            r: 92,
            g: 106,
            b: 114,
        },
        file_tree_text: Rgb {
            r: 92,
            g: 106,
            b: 114,
        },
        active_file_highlight: Rgba {
            r: 127,
            g: 187,
            b: 179,
            a: 40,
        },
        button_background: Rgba {
            r: 244,
            g: 240,
            b: 217,
            a: 255,
        },
        button_active_background: Rgba {
            r: 127,
            g: 187,
            b: 179,
            a: 80,
        },
        border: Rgb {
            r: 200,
            g: 200,
            b: 200,
        },
        selection: Rgb {
            r: 200,
            g: 220,
            b: 255,
        },
        splash_background: Rgb {
            r: 253,
            g: 246,
            b: 227,
        },
        splash_progress: Rgb {
            r: 127,
            g: 187,
            b: 179,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 243,
            g: 236,
            b: 217,
        },
        text: Rgb {
            r: 40,
            g: 40,
            b: 40,
        },
        line_number_text: Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        line_number_active_text: Rgb {
            r: 40,
            g: 40,
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
            r: 200,
            g: 220,
            b: 255,
        },
    },
    preview: PreviewColors {
        background: Rgb {
            r: 253,
            g: 246,
            b: 227,
        },
        text: Rgb {
            r: 40,
            g: 40,
            b: 40,
        },
        warning_text: Rgb {
            r: 223,
            g: 160,
            b: 0,
        },
        border: Rgb {
            r: 200,
            g: 200,
            b: 200,
        },
        selection: Rgb {
            r: 200,
            g: 220,
            b: 255,
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
