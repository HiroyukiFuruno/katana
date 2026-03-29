use super::DiagramColorPreset;
use std::sync::OnceLock;

pub fn get_dark_preset() -> &'static DiagramColorPreset {
    static DARK: OnceLock<DiagramColorPreset> = OnceLock::new();
    DARK.get_or_init(|| DiagramColorPreset {
        background: "transparent",
        text: "#E0E0E0",
        fill: "#2D2D2D",
        stroke: "#888888",
        arrow: "#AAAAAA",
        drawio_label_color: "#1A1A1A",
        mermaid_theme: "dark",
        plantuml_class_bg: "#2D2D2D",
        plantuml_note_bg: "#3A3A3A",
        plantuml_note_text: "#E0E0E0",
        syntax_theme_dark: "base16-ocean.dark",
        syntax_theme_light: "base16-ocean.light",
        preview_text: "#E0E0E0",
        proportional_font_candidates: super::default_proportional_fonts(),
        monospace_font_candidates: super::default_monospace_fonts(),
        emoji_font_candidates: super::default_emoji_fonts(),
        editor_font_size: DiagramColorPreset::DEFAULT_EDITOR_FONT_SIZE,
    })
}
