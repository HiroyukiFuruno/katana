use crate::theme::types::*;

pub(crate) const ALABASTER: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    system: SystemColors {
        background: Rgb {
            r: 247,
            g: 247,
            b: 247,
        },
        panel_background: Rgb {
            r: 252,
            g: 252,
            b: 252,
        },
        text: Rgb {
            r: 40,
            g: 40,
            b: 40,
        },
        text_secondary: Rgb {
            r: 110,
            g: 110,
            b: 110,
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
            r: 40,
            g: 40,
            b: 40,
        },
        file_tree_text: Rgb {
            r: 110,
            g: 110,
            b: 110,
        },
        active_file_highlight: Rgba {
            r: 0,
            g: 100,
            b: 200,
            a: 40,
        },
        button_background: Rgba {
            r: 252,
            g: 252,
            b: 252,
            a: 255,
        },
        button_active_background: Rgba {
            r: 0,
            g: 100,
            b: 200,
            a: 100,
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
            r: 247,
            g: 247,
            b: 247,
        },
        splash_progress: Rgb {
            r: 0,
            g: 100,
            b: 200,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 237,
            g: 237,
            b: 237,
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
            r: 247,
            g: 247,
            b: 247,
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
