use crate::theme::types::*;

pub(crate) const TOKYO_NIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Dark,
    system: SystemColors {
        background: Rgb {
            r: 31,
            g: 35,
            b: 53,
        },
        panel_background: Rgb {
            r: 26,
            g: 27,
            b: 38,
        },
        text: Rgb {
            r: 212,
            g: 212,
            b: 212,
        },
        text_secondary: Rgb {
            r: 86,
            g: 95,
            b: 137,
        },
        success_text: Rgb {
            r: 158,
            g: 206,
            b: 106,
        },
        error_text: Rgb {
            r: 219,
            g: 75,
            b: 75,
        },
        warning_text: Rgb {
            r: 224,
            g: 175,
            b: 104,
        },
        accent: Rgb {
            r: 100,
            g: 150,
            b: 255,
        },
        title_bar_text: Rgb {
            r: 192,
            g: 202,
            b: 245,
        },
        file_tree_text: Rgb {
            r: 169,
            g: 177,
            b: 214,
        },
        active_file_highlight: Rgba {
            r: 122,
            g: 162,
            b: 247,
            a: 40,
        },
        button_background: Rgba {
            r: 65,
            g: 72,
            b: 104,
            a: 255,
        },
        button_active_background: Rgba {
            r: 122,
            g: 162,
            b: 247,
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
            r: 26,
            g: 27,
            b: 38,
        },
        splash_progress: Rgb {
            r: 122,
            g: 162,
            b: 247,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 36,
            g: 37,
            b: 48,
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
            r: 26,
            g: 27,
            b: 38,
        },
        text: Rgb {
            r: 212,
            g: 212,
            b: 212,
        },
        warning_text: Rgb {
            r: 224,
            g: 175,
            b: 104,
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
