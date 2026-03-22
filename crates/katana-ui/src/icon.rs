/// Generates the Icon enum, name(), svg_bytes(), and ALL_ICONS from a single
/// declaration table, eliminating path duplication.
macro_rules! define_icons {
    ( $( $(#[$meta:meta])* $variant:ident => $file:literal ),+ $(,)? ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Icon {
            $( $(#[$meta])* $variant, )+
        }

        impl Icon {
            /// Returns the lowercase name used in SVG asset paths.
            pub const fn name(&self) -> &'static str {
                match self {
                    $( Self::$variant => $file, )+
                }
            }

            /// Returns raw SVG bytes embedded at compile time.
            pub fn svg_bytes(&self) -> &'static [u8] {
                match self {
                    $( Self::$variant => include_bytes!(
                        concat!("../../../assets/icons/", $file, ".svg")
                    ), )+
                }
            }
        }

        /// All icon variants for iteration.
        pub const ALL_ICONS: &[Icon] = &[
            $( Icon::$variant, )+
        ];
    };
}

define_icons! {
    Dot             => "dot",
    ChevronLeft     => "chevron_left",
    ChevronRight    => "chevron_right",
    Refresh         => "refresh",
    /// 'x'
    Close           => "close",
    /// '×' (U+00D7)
    Remove          => "remove",
    ExternalLink    => "external_link",
    TriangleDown    => "triangle_down",
    TriangleLeft    => "triangle_left",
    TriangleRight   => "triangle_right",
    Search          => "search",
    Plus            => "plus",
    Minus           => "minus",
    Toc             => "toc",
    // Viewer overlay controls
    PanUp           => "pan_up",
    PanDown         => "pan_down",
    PanLeft         => "pan_left",
    PanRight        => "pan_right",
    ZoomIn          => "zoom_in",
    ZoomOut         => "zoom_out",
    ResetView       => "reset_view",
    Fullscreen      => "fullscreen",
    CloseModal      => "close_modal",
    Info            => "info",
    Success         => "success",
    Warning         => "warning",
    Error           => "error",
    Export          => "export",
    Filter          => "filter",
    // File tree & layout icons
    SplitVertical   => "split_vertical",
    SplitHorizontal => "split_horizontal",
    Preview         => "preview",
    Document        => "document",
    FolderOpen      => "folder_open",
    FolderClosed    => "folder_closed",
    Copy            => "copy",
    ExpandAll       => "expand_all",
    CollapseAll     => "collapse_all",
}

/// Predefined icon sizes for consistent rendering across the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconSize {
    /// 12×12 — compact controls (tab close, small toolbar buttons).
    Small,
    /// 16×16 — standard toolbar and sidebar icons.
    Medium,
    /// 20×20 — prominent actions and diagram controls.
    Large,
}

impl IconSize {
    /// Returns the pixel dimensions as an `egui::Vec2`.
    const SMALL: f32 = 12.0;
    const MEDIUM: f32 = 16.0;
    const LARGE: f32 = 20.0;

    pub const fn to_vec2(self) -> egui::Vec2 {
        match self {
            Self::Small => egui::vec2(Self::SMALL, Self::SMALL),
            Self::Medium => egui::vec2(Self::MEDIUM, Self::MEDIUM),
            Self::Large => egui::vec2(Self::LARGE, Self::LARGE),
        }
    }
}

impl Icon {
    /// Returns the `bytes://` URI used by the egui image loader.
    pub fn uri(&self) -> String {
        format!("bytes://icon/{}.svg", self.name())
    }

    /// Returns an `egui::Image` sized and tinted for the given parameters.
    /// Use `ui_image()` for automatic theme color tinting.
    pub fn image(&self, size: IconSize) -> egui::Image<'static> {
        egui::Image::new(self.uri()).fit_to_exact_size(size.to_vec2())
    }

    /// Returns an `egui::Image` tinted with the current UI text color.
    /// Preferred method for most icon usages.
    pub fn ui_image(&self, ui: &egui::Ui, size: IconSize) -> egui::Image<'static> {
        self.image(size).tint(ui.visuals().text_color())
    }
}

/// Registers all icon SVG bytes with the egui context for lazy loading.
pub struct IconRegistry;

impl IconRegistry {
    /// Registers all icon SVGs with the given context via `include_bytes`.
    pub fn install(ctx: &egui::Context) {
        for icon in ALL_ICONS {
            ctx.include_bytes(icon.uri(), icon.svg_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_size_to_vec2_returns_correct_dimensions() {
        assert_eq!(
            IconSize::Small.to_vec2(),
            egui::vec2(IconSize::SMALL, IconSize::SMALL)
        );
        assert_eq!(
            IconSize::Medium.to_vec2(),
            egui::vec2(IconSize::MEDIUM, IconSize::MEDIUM)
        );
        assert_eq!(
            IconSize::Large.to_vec2(),
            egui::vec2(IconSize::LARGE, IconSize::LARGE)
        );
    }

    #[test]
    fn icon_name_returns_snake_case_identifier() {
        assert_eq!(Icon::Refresh.name(), "refresh");
        assert_eq!(Icon::ChevronLeft.name(), "chevron_left");
        assert_eq!(Icon::ExternalLink.name(), "external_link");
        assert_eq!(Icon::ZoomIn.name(), "zoom_in");
    }

    #[test]
    fn icon_uri_follows_bytes_scheme() {
        assert_eq!(Icon::Refresh.uri(), "bytes://icon/refresh.svg");
        assert_eq!(Icon::ChevronLeft.uri(), "bytes://icon/chevron_left.svg");
    }

    #[test]
    fn all_icons_have_valid_svg_bytes() {
        for icon in ALL_ICONS {
            let bytes = icon.svg_bytes();
            let svg_str = std::str::from_utf8(bytes)
                .unwrap_or_else(|_| panic!("icon {:?} has invalid UTF-8", icon));
            assert!(
                svg_str.contains("<svg"),
                "icon {:?} SVG bytes must contain <svg tag",
                icon
            );
        }
    }

    #[test]
    fn all_icons_list_covers_every_variant() {
        assert_eq!(ALL_ICONS.len(), 38);
    }

    #[test]
    fn icon_registry_install_registers_all_icons() {
        let ctx = egui::Context::default();
        IconRegistry::install(&ctx);
        // egui does not expose a public API to query registered bytes,
        // but install should not panic for any icon.
    }

    #[test]
    fn icon_bytes_survive_forget_all_images_after_re_install() {
        let ctx = egui::Context::default();
        crate::svg_loader::install_image_loaders(&ctx);
        IconRegistry::install(&ctx);

        // Simulate RefreshDiagrams: forget_all_images clears byte registrations
        ctx.forget_all_images();

        // Re-register icon bytes (the fix)
        IconRegistry::install(&ctx);

        // Verify the SVG loader can still load an icon after re-install
        let result = ctx.try_load_bytes(&Icon::Refresh.uri());
        assert!(
            result.is_ok(),
            "Icon bytes must be available after forget_all_images + re-install"
        );
    }
}
