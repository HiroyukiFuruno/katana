use crate::theme::types::*;

pub(crate) const CATPPUCCIN_LATTE: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    system: SystemColors {
        background: Rgb {
            r: 239,
            g: 241,
            b: 245,
        },
        panel_background: Rgb {
            r: 230,
            g: 233,
            b: 239,
        },
        text: Rgb {
            r: 40,
            g: 40,
            b: 40,
        },
        text_secondary: Rgb {
            r: 108,
            g: 111,
            b: 133,
        },
        success_text: Rgb {
            r: 64,
            g: 160,
            b: 43,
        },
        error_text: Rgb {
            r: 210,
            g: 15,
            b: 57,
        },
        warning_text: Rgb {
            r: 223,
            g: 142,
            b: 29,
        },
        accent: Rgb {
            r: 0,
            g: 100,
            b: 200,
        },
        title_bar_text: Rgb {
            r: 76,
            g: 79,
            b: 105,
        },
        file_tree_text: Rgb {
            r: 76,
            g: 79,
            b: 105,
        },
        active_file_highlight: Rgba {
            r: 30,
            g: 102,
            b: 245,
            a: 30,
        },
        button_background: Rgba {
            r: 230,
            g: 233,
            b: 239,
            a: 255,
        },
        button_active_background: Rgba {
            r: 30,
            g: 102,
            b: 245,
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
            r: 239,
            g: 241,
            b: 245,
        },
        splash_progress: Rgb {
            r: 30,
            g: 102,
            b: 245,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 229,
            g: 231,
            b: 235,
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
            r: 239,
            g: 241,
            b: 245,
        },
        text: Rgb {
            r: 40,
            g: 40,
            b: 40,
        },
        warning_text: Rgb {
            r: 223,
            g: 142,
            b: 29,
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
