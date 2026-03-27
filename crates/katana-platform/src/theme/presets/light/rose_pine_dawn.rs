use crate::theme::types::*;

pub(crate) const ROSE_PINE_DAWN: PresetColorData = PresetColorData {
    mode: ThemeMode::Light,
    system: SystemColors {
        background: Rgb {
            r: 250,
            g: 244,
            b: 237,
        },
        panel_background: Rgb {
            r: 255,
            g: 250,
            b: 243,
        },
        text: Rgb {
            r: 40,
            g: 40,
            b: 40,
        },
        text_secondary: Rgb {
            r: 121,
            g: 117,
            b: 147,
        },
        success_text: Rgb {
            r: 40,
            g: 105,
            b: 131,
        },
        error_text: Rgb {
            r: 180,
            g: 99,
            b: 122,
        },
        warning_text: Rgb {
            r: 215,
            g: 130,
            b: 126,
        },
        accent: Rgb {
            r: 0,
            g: 100,
            b: 200,
        },
        title_bar_text: Rgb {
            r: 87,
            g: 82,
            b: 121,
        },
        file_tree_text: Rgb {
            r: 87,
            g: 82,
            b: 121,
        },
        active_file_highlight: Rgba {
            r: 86,
            g: 148,
            b: 159,
            a: 40,
        },
        button_background: Rgba {
            r: 255,
            g: 250,
            b: 243,
            a: 255,
        },
        button_active_background: Rgba {
            r: 86,
            g: 148,
            b: 159,
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
            r: 250,
            g: 244,
            b: 237,
        },
        splash_progress: Rgb {
            r: 144,
            g: 122,
            b: 169,
        },
    },
    code: CodeColors {
        background: Rgb {
            r: 240,
            g: 234,
            b: 227,
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
            g: 244,
            b: 237,
        },
        text: Rgb {
            r: 40,
            g: 40,
            b: 40,
        },
        warning_text: Rgb {
            r: 215,
            g: 130,
            b: 126,
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
