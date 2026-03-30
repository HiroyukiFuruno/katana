use crate::theme::preset::{PresetColorData, ThemePreset};
use crate::theme::presets::{
    ALABASTER, ANDROMEDA, AYU_LIGHT, CATPPUCCIN_LATTE, CATPPUCCIN_MOCHA, DRACULA, EVERFOREST_LIGHT,
    FLAT_UI_LIGHT, GITHUB_DARK, GITHUB_LIGHT, GRUVBOX_LIGHT, KATANA_DARK, KATANA_LIGHT,
    MATERIAL_DARK, MATERIAL_LIGHT, MINIMAL_LIGHT, MONOKAI, NIGHT_OWL, NORD, OCEANIC_NEXT, ONE_DARK,
    ONE_LIGHT, PALENIGHT, PAPERCOLOR_LIGHT, QUIET_LIGHT, ROSE_PINE, ROSE_PINE_DAWN,
    SOLARIZED_LIGHT, SYNTHWAVE_84, TOKYO_NIGHT,
};
use crate::theme::types::{ThemeColors, ThemeMode};

impl ThemeMode {
    // WHY: legacy theme string used for backward-compatible JSON persistence.
    pub fn to_theme_string(self) -> String {
        match self {
            ThemeMode::Dark => "dark".to_string(),
            ThemeMode::Light => "light".to_string(),
        }
    }
}

impl ThemePreset {
    // WHY: Extract the populated colour data.
    pub fn colors(self) -> ThemeColors {
        let (name, data) = Self::get_data(&self);
        data.to_theme_colors(name)
    }

    pub fn display_name(&self) -> &'static str {
        Self::get_data(self).0
    }

    pub fn builtins() -> Vec<Self> {
        PRESET_DATA.iter().map(|(p, _, _)| *p).collect()
    }

    fn get_data(&self) -> (&'static str, &'static PresetColorData) {
        PRESET_DATA
            .iter()
            .find(|(p, _, _)| p == self)
            .map(|(_, n, d)| (*n, *d))
            .unwrap()
    }
}

static PRESET_DATA: &[(ThemePreset, &str, &PresetColorData)] = &[
    (ThemePreset::KatanaDark, "KatanA Dark", &KATANA_DARK),
    (ThemePreset::Dracula, "Dracula", &DRACULA),
    (ThemePreset::GitHubDark, "GitHub Dark", &GITHUB_DARK),
    (ThemePreset::Nord, "Nord", &NORD),
    (ThemePreset::Monokai, "Monokai", &MONOKAI),
    (ThemePreset::OneDark, "One Dark", &ONE_DARK),
    (ThemePreset::TokyoNight, "Tokyo Night", &TOKYO_NIGHT),
    (
        ThemePreset::CatppuccinMocha,
        "Catppuccin Mocha",
        &CATPPUCCIN_MOCHA,
    ),
    (ThemePreset::MaterialDark, "Material Dark", &MATERIAL_DARK),
    (ThemePreset::NightOwl, "Night Owl", &NIGHT_OWL),
    (ThemePreset::RosePine, "Rosé Pine", &ROSE_PINE),
    (ThemePreset::Palenight, "Palenight", &PALENIGHT),
    (ThemePreset::SynthWave84, "SynthWave '84", &SYNTHWAVE_84),
    (ThemePreset::Andromeda, "Andromeda", &ANDROMEDA),
    (ThemePreset::OceanicNext, "Oceanic Next", &OCEANIC_NEXT),
    (ThemePreset::KatanaLight, "KatanA Light", &KATANA_LIGHT),
    (ThemePreset::GitHubLight, "GitHub Light", &GITHUB_LIGHT),
    (
        ThemePreset::SolarizedLight,
        "Solarized Light",
        &SOLARIZED_LIGHT,
    ),
    (ThemePreset::AyuLight, "Ayu Light", &AYU_LIGHT),
    (ThemePreset::GruvboxLight, "Gruvbox Light", &GRUVBOX_LIGHT),
    (ThemePreset::OneLight, "One Light", &ONE_LIGHT),
    (ThemePreset::RosePineDawn, "Rosé Pine Dawn", &ROSE_PINE_DAWN),
    (
        ThemePreset::CatppuccinLatte,
        "Catppuccin Latte",
        &CATPPUCCIN_LATTE,
    ),
    (
        ThemePreset::MaterialLight,
        "Material Light",
        &MATERIAL_LIGHT,
    ),
    (ThemePreset::QuietLight, "Quiet Light", &QUIET_LIGHT),
    (
        ThemePreset::PaperColorLight,
        "PaperColor Light",
        &PAPERCOLOR_LIGHT,
    ),
    (ThemePreset::MinimalLight, "Minimal Light", &MINIMAL_LIGHT),
    (ThemePreset::Alabaster, "Alabaster", &ALABASTER),
    (ThemePreset::FlatUILight, "Flat UI Light", &FLAT_UI_LIGHT),
    (
        ThemePreset::EverforestLight,
        "Everforest Light",
        &EVERFOREST_LIGHT,
    ),
];

impl PresetColorData {
    pub(crate) fn to_theme_colors(&self, name: &str) -> ThemeColors {
        ThemeColors {
            name: name.to_string(),
            mode: self.mode,
            system: self.system.clone(),
            code: self.code.clone(),
            preview: self.preview.clone(),
        }
    }
}