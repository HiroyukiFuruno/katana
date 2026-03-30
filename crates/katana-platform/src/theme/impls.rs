use crate::theme::presets::{
    ALABASTER, ANDROMEDA, AYU_LIGHT, CATPPUCCIN_LATTE, CATPPUCCIN_MOCHA, DRACULA, EVERFOREST_LIGHT,
    FLAT_UI_LIGHT, GITHUB_DARK, GITHUB_LIGHT, GRUVBOX_LIGHT, KATANA_DARK, KATANA_LIGHT,
    MATERIAL_DARK, MATERIAL_LIGHT, MINIMAL_LIGHT, MONOKAI, NIGHT_OWL, NORD, OCEANIC_NEXT, ONE_DARK,
    ONE_LIGHT, PALENIGHT, PAPERCOLOR_LIGHT, QUIET_LIGHT, ROSE_PINE, ROSE_PINE_DAWN,
    SOLARIZED_LIGHT, SYNTHWAVE_84, TOKYO_NIGHT,
};
use crate::theme::types::{PresetColorData, ThemeColors, ThemeMode, ThemePreset};

impl ThemeMode {
    /// Returns the legacy theme string used for backward-compatible JSON persistence.
    pub fn to_theme_string(self) -> String {
        match self {
            ThemeMode::Dark => "dark".to_string(),
            ThemeMode::Light => "light".to_string(),
        }
    }
}

impl ThemePreset {
    /// Extract the populated colour data.
    pub fn colors(self) -> ThemeColors {
        let (name, data) = match self {
            Self::KatanaDark => ("KatanA Dark", &KATANA_DARK),
            Self::Dracula => ("Dracula", &DRACULA),
            Self::GitHubDark => ("GitHub Dark", &GITHUB_DARK),
            Self::Nord => ("Nord", &NORD),
            Self::Monokai => ("Monokai", &MONOKAI),
            Self::OneDark => ("One Dark", &ONE_DARK),
            Self::TokyoNight => ("Tokyo Night", &TOKYO_NIGHT),
            Self::CatppuccinMocha => ("Catppuccin Mocha", &CATPPUCCIN_MOCHA),
            Self::MaterialDark => ("Material Dark", &MATERIAL_DARK),
            Self::NightOwl => ("Night Owl", &NIGHT_OWL),
            Self::RosePine => ("Rosé Pine", &ROSE_PINE),
            Self::Palenight => ("Palenight", &PALENIGHT),
            Self::SynthWave84 => ("SynthWave '84", &SYNTHWAVE_84),
            Self::Andromeda => ("Andromeda", &ANDROMEDA),
            Self::OceanicNext => ("Oceanic Next", &OCEANIC_NEXT),
            Self::KatanaLight => ("KatanA Light", &KATANA_LIGHT),
            Self::GitHubLight => ("GitHub Light", &GITHUB_LIGHT),
            Self::SolarizedLight => ("Solarized Light", &SOLARIZED_LIGHT),
            Self::AyuLight => ("Ayu Light", &AYU_LIGHT),
            Self::GruvboxLight => ("Gruvbox Light", &GRUVBOX_LIGHT),
            Self::OneLight => ("One Light", &ONE_LIGHT),
            Self::RosePineDawn => ("Rosé Pine Dawn", &ROSE_PINE_DAWN),
            Self::CatppuccinLatte => ("Catppuccin Latte", &CATPPUCCIN_LATTE),
            Self::MaterialLight => ("Material Light", &MATERIAL_LIGHT),
            Self::QuietLight => ("Quiet Light", &QUIET_LIGHT),
            Self::PaperColorLight => ("PaperColor Light", &PAPERCOLOR_LIGHT),
            Self::MinimalLight => ("Minimal Light", &MINIMAL_LIGHT),
            Self::Alabaster => ("Alabaster", &ALABASTER),
            Self::FlatUILight => ("Flat UI Light", &FLAT_UI_LIGHT),
            Self::EverforestLight => ("Everforest Light", &EVERFOREST_LIGHT),
        };
        data.to_theme_colors(name)
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::KatanaDark => "KatanA Dark",
            Self::Dracula => "Dracula",
            Self::GitHubDark => "GitHub Dark",
            Self::Nord => "Nord",
            Self::Monokai => "Monokai",
            Self::OneDark => "One Dark",
            Self::TokyoNight => "Tokyo Night",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::MaterialDark => "Material Dark",
            Self::NightOwl => "Night Owl",
            Self::RosePine => "Rosé Pine",
            Self::Palenight => "Palenight",
            Self::SynthWave84 => "SynthWave '84",
            Self::Andromeda => "Andromeda",
            Self::OceanicNext => "Oceanic Next",
            Self::KatanaLight => "KatanA Light",
            Self::GitHubLight => "GitHub Light",
            Self::SolarizedLight => "Solarized Light",
            Self::AyuLight => "Ayu Light",
            Self::GruvboxLight => "Gruvbox Light",
            Self::OneLight => "One Light",
            Self::RosePineDawn => "Rosé Pine Dawn",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::MaterialLight => "Material Light",
            Self::QuietLight => "Quiet Light",
            Self::PaperColorLight => "PaperColor Light",
            Self::MinimalLight => "Minimal Light",
            Self::Alabaster => "Alabaster",
            Self::FlatUILight => "Flat UI Light",
            Self::EverforestLight => "Everforest Light",
        }
    }

    pub fn builtins() -> Vec<Self> {
        vec![
            Self::KatanaDark,
            Self::Dracula,
            Self::GitHubDark,
            Self::Nord,
            Self::Monokai,
            Self::OneDark,
            Self::TokyoNight,
            Self::CatppuccinMocha,
            Self::MaterialDark,
            Self::NightOwl,
            Self::RosePine,
            Self::Palenight,
            Self::SynthWave84,
            Self::Andromeda,
            Self::OceanicNext,
            Self::KatanaLight,
            Self::GitHubLight,
            Self::SolarizedLight,
            Self::AyuLight,
            Self::GruvboxLight,
            Self::OneLight,
            Self::RosePineDawn,
            Self::CatppuccinLatte,
            Self::MaterialLight,
            Self::QuietLight,
            Self::PaperColorLight,
            Self::MinimalLight,
            Self::Alabaster,
            Self::FlatUILight,
            Self::EverforestLight,
        ]
    }
}

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
