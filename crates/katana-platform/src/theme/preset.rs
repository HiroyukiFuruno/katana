use serde::{Deserialize, Serialize};

use crate::theme::types::{CodeColors, PreviewColors, SystemColors, ThemeMode};

/// WHY: Built-in theme presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemePreset {
    #[default]
    KatanaDark,
    Dracula,
    GitHubDark,
    Nord,
    Monokai,
    OneDark,
    TokyoNight,
    CatppuccinMocha,
    MaterialDark,
    NightOwl,
    RosePine,
    Palenight,
    SynthWave84,
    Andromeda,
    OceanicNext,
    KatanaLight,
    GitHubLight,
    SolarizedLight,
    AyuLight,
    GruvboxLight,
    OneLight,
    RosePineDawn,
    CatppuccinLatte,
    MaterialLight,
    QuietLight,
    PaperColorLight,
    MinimalLight,
    Alabaster,
    FlatUILight,
    EverforestLight,
}

pub(crate) struct PresetColorData {
    pub mode: ThemeMode,
    pub system: SystemColors,
    pub code: CodeColors,
    pub preview: PreviewColors,
}
