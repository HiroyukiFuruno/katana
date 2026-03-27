use crate::theme::types::*;

pub(crate) const ONE_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    system: SystemColors {
        background: Rgb {
            r: 250,
            g: 250,
            b: 250,
        },
        panel_background: Rgb {
            r: 240,
            g: 240,
            b: 241,
        },
        text: Rgb {
            r: 40,
            g: 40,
            b: 40,
        },
        text_secondary: Rgb {
            r: 160,
            g: 161,
            b: 167,
        },
        success_text: Rgb {
            r: 80,
            g: 161,
            b: 79,
        },
        error_text: Rgb {
            r: 228,
            g: 86,
            b: 73,
        },
        warning_text: Rgb {
            r: 193,
            g: 132,
            b: 1,
        },
        accent: Rgb {
            r: 0,
            g: 100,
            b: 200,
        },
        title_bar_text: Rgb {
            r: 56,
            g: 58,
            b: 66,
        },
        file_tree_text: Rgb {
            r: 56,
            g: 58,
            b: 66,
        },
        active_file_highlight: Rgba {
            r: 64,
            g: 120,
            b: 242,
            a: 30,
        },
        button_background: Rgba {
            r: 240,
            g: 240,
            b: 241,
            a: 255,
        },
        button_active_background: Rgba {
            r: 64,
            g: 120,
            b: 242,
            a: 70,
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
            r: 250,
            g: 250,
            b: 250,
        },
        splash_progress: Rgb {
            r: 64,
            g: 120,
            b: 242,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 240,
            g: 240,
            b: 240,
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
            r: 250,
            g: 250,
            b: 250,
        },
        text: Rgb {
            r: 40,
            g: 40,
            b: 40,
        },
        warning_text: Rgb {
            r: 193,
            g: 132,
            b: 1,
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
