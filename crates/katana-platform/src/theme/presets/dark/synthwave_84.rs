use crate::theme::types::*;

pub(crate) const SYNTHWAVE_84: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    system: SystemColors {
        background: Rgb {
            r: 43,
            g: 33,
            b: 58,
        },
        panel_background: Rgb {
            r: 36,
            g: 27,
            b: 47,
        },
        text: Rgb {
            r: 212,
            g: 212,
            b: 212,
        },
        text_secondary: Rgb {
            r: 132,
            g: 139,
            b: 189,
        },
        success_text: Rgb {
            r: 114,
            g: 241,
            b: 184,
        },
        error_text: Rgb {
            r: 254,
            g: 68,
            b: 80,
        },
        warning_text: Rgb {
            r: 254,
            g: 222,
            b: 93,
        },
        accent: Rgb {
            r: 100,
            g: 150,
            b: 255,
        },
        title_bar_text: Rgb {
            r: 249,
            g: 42,
            b: 173,
        },
        file_tree_text: Rgb {
            r: 182,
            g: 177,
            b: 177,
        },
        active_file_highlight: Rgba {
            r: 249,
            g: 42,
            b: 173,
            a: 50,
        },
        button_background: Rgba {
            r: 52,
            g: 41,
            b: 79,
            a: 255,
        },
        button_active_background: Rgba {
            r: 249,
            g: 42,
            b: 173,
            a: 100,
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
            r: 43,
            g: 33,
            b: 58,
        },
        splash_progress: Rgb {
            r: 54,
            g: 249,
            b: 246,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 48,
            g: 45,
            b: 63,
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
            r: 38,
            g: 35,
            b: 53,
        },
        text: Rgb {
            r: 212,
            g: 212,
            b: 212,
        },
        warning_text: Rgb {
            r: 254,
            g: 222,
            b: 93,
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
