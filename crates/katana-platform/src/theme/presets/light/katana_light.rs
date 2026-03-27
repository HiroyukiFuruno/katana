use crate::theme::types::*;

pub(crate) const KATANA_LIGHT: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    system: SystemColors {
        background: Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
        panel_background: Rgb {
            r: 243,
            g: 243,
            b: 243,
        },
        text: Rgb {
            r: 36,
            g: 36,
            b: 36,
        },
        text_secondary: Rgb {
            r: 106,
            g: 106,
            b: 106,
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
            g: 120,
            b: 212,
        },
        title_bar_text: Rgb {
            r: 36,
            g: 36,
            b: 36,
        },
        file_tree_text: Rgb {
            r: 106,
            g: 106,
            b: 106,
        },
        active_file_highlight: Rgba {
            r: 0,
            g: 120,
            b: 212,
            a: 40,
        },
        button_background: Rgba {
            r: 243,
            g: 243,
            b: 243,
            a: 255,
        },
        button_active_background: Rgba {
            r: 0,
            g: 120,
            b: 212,
            a: 100,
        },
        border: Rgb {
            r: 220,
            g: 220,
            b: 220,
        },
        selection: Rgb {
            r: 173,
            g: 214,
            b: 255,
        },
        splash_background: Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
        splash_progress: Rgb {
            r: 0,
            g: 120,
            b: 212,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 243,
            g: 243,
            b: 243,
        },
        text: Rgb {
            r: 36,
            g: 36,
            b: 36,
        },
        line_number_text: Rgb {
            r: 160,
            g: 160,
            b: 160,
        },
        line_number_active_text: Rgb {
            r: 36,
            g: 36,
            b: 36,
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
            r: 173,
            g: 214,
            b: 255,
        },
    },
    preview: PreviewColors {
        background: Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
        text: Rgb {
            r: 36,
            g: 36,
            b: 36,
        },
        warning_text: Rgb {
            r: 223,
            g: 142,
            b: 29,
        },
        border: Rgb {
            r: 220,
            g: 220,
            b: 220,
        },
        selection: Rgb {
            r: 173,
            g: 214,
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
