use crate::theme::types::*;

pub(crate) const ROSE_PINE: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    system: SystemColors {
        background: Rgb {
            r: 25,
            g: 23,
            b: 36,
        },
        panel_background: Rgb {
            r: 31,
            g: 29,
            b: 46,
        },
        text: Rgb {
            r: 212,
            g: 212,
            b: 212,
        },
        text_secondary: Rgb {
            r: 144,
            g: 140,
            b: 170,
        },
        success_text: Rgb {
            r: 156,
            g: 207,
            b: 216,
        },
        error_text: Rgb {
            r: 235,
            g: 111,
            b: 146,
        },
        warning_text: Rgb {
            r: 246,
            g: 193,
            b: 119,
        },
        accent: Rgb {
            r: 100,
            g: 150,
            b: 255,
        },
        title_bar_text: Rgb {
            r: 224,
            g: 222,
            b: 244,
        },
        file_tree_text: Rgb {
            r: 224,
            g: 222,
            b: 244,
        },
        active_file_highlight: Rgba {
            r: 49,
            g: 116,
            b: 143,
            a: 60,
        },
        button_background: Rgba {
            r: 38,
            g: 35,
            b: 58,
            a: 255,
        },
        button_active_background: Rgba {
            r: 49,
            g: 116,
            b: 143,
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
            r: 25,
            g: 23,
            b: 36,
        },
        splash_progress: Rgb {
            r: 196,
            g: 167,
            b: 231,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 35,
            g: 33,
            b: 46,
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
            r: 25,
            g: 23,
            b: 36,
        },
        text: Rgb {
            r: 212,
            g: 212,
            b: 212,
        },
        warning_text: Rgb {
            r: 246,
            g: 193,
            b: 119,
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
