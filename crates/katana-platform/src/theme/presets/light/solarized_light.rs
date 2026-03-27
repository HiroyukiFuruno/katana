use crate::theme::types::*;

pub(crate) const SOLARIZED_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    system: SystemColors {
        background: Rgb {
            r: 253,
            g: 246,
            b: 227,
        },
        panel_background: Rgb {
            r: 238,
            g: 232,
            b: 213,
        },
        text: Rgb {
            r: 101,
            g: 123,
            b: 131,
        },
        text_secondary: Rgb {
            r: 88,
            g: 110,
            b: 117,
        },
        success_text: Rgb {
            r: 133,
            g: 153,
            b: 0,
        },
        error_text: Rgb {
            r: 220,
            g: 50,
            b: 47,
        },
        warning_text: Rgb {
            r: 181,
            g: 137,
            b: 0,
        },
        accent: Rgb {
            r: 38,
            g: 139,
            b: 210,
        },
        title_bar_text: Rgb { r: 7, g: 54, b: 66 },
        file_tree_text: Rgb {
            r: 101,
            g: 123,
            b: 131,
        },
        active_file_highlight: Rgba {
            r: 38,
            g: 139,
            b: 210,
            a: 40,
        },
        button_background: Rgba {
            r: 238,
            g: 232,
            b: 213,
            a: 255,
        },
        button_active_background: Rgba {
            r: 38,
            g: 139,
            b: 210,
            a: 80,
        },
        border: Rgb {
            r: 238,
            g: 232,
            b: 213,
        },
        selection: Rgb {
            r: 238,
            g: 232,
            b: 213,
        },
        splash_background: Rgb {
            r: 253,
            g: 246,
            b: 227,
        },
        splash_progress: Rgb {
            r: 38,
            g: 139,
            b: 210,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 238,
            g: 232,
            b: 213,
        },
        text: Rgb {
            r: 101,
            g: 123,
            b: 131,
        },
        line_number_text: Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        line_number_active_text: Rgb {
            r: 101,
            g: 123,
            b: 131,
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
            r: 238,
            g: 232,
            b: 213,
        },
    },
    preview: PreviewColors {
        background: Rgb {
            r: 253,
            g: 246,
            b: 227,
        },
        text: Rgb {
            r: 101,
            g: 123,
            b: 131,
        },
        warning_text: Rgb {
            r: 181,
            g: 137,
            b: 0,
        },
        border: Rgb {
            r: 238,
            g: 232,
            b: 213,
        },
        selection: Rgb {
            r: 238,
            g: 232,
            b: 213,
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
