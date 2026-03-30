macro_rules! define_icons {
    ( $( $(#[$meta:meta])* $variant:ident => $file:literal ),+ $(,)? ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Icon {
            $( $(#[$meta])* $variant, )+
        }

        impl Icon {
            pub const fn name(&self) -> &'static str {
                match self {
                    $( Self::$variant => $file, )+
                }
            }

            pub fn svg_bytes(&self) -> &'static [u8] {
                match self {
                    $( Self::$variant => include_bytes!(
                        concat!("../../../assets/icons/", $file, ".svg")
                    ), )+
                }
            }
        }

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
    Close           => "close",
    Remove          => "remove",
    ExternalLink    => "external_link",
    TriangleDown    => "triangle_down",
    TriangleLeft    => "triangle_left",
    TriangleRight   => "triangle_right",
    Search          => "search",
    Plus            => "plus",
    Minus           => "minus",
    Toc             => "toc",
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
    SplitVertical   => "split_vertical",
    SplitHorizontal => "split_horizontal",
    Preview         => "preview",
    Document        => "document",
    FolderOpen      => "folder_open",
    FolderClosed    => "folder_closed",
    Copy            => "copy",
    ExpandAll       => "expand_all",
    CollapseAll     => "collapse_all",
    Github          => "github",
    Heart           => "heart",
    Bug             => "bug",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconSize {
    Small,
    Medium,
    Large,
}

impl IconSize {
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
    pub fn uri(&self) -> String {
        format!("bytes://icon/{}.svg", self.name())
    }

    pub fn image(&self, size: IconSize) -> egui::Image<'static> {
        egui::Image::new(self.uri()).fit_to_exact_size(size.to_vec2())
    }

    pub fn ui_image(&self, ui: &egui::Ui, size: IconSize) -> egui::Image<'static> {
        self.image(size).tint(ui.visuals().text_color())
    }
}

pub struct IconRegistry;

impl IconRegistry {
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
        assert_eq!(ALL_ICONS.len(), 41);
    }

    #[test]
    fn icon_registry_install_registers_all_icons() {
        let ctx = egui::Context::default();
        IconRegistry::install(&ctx);
    }

    #[test]
    fn icon_bytes_survive_forget_all_images_after_re_install() {
        let ctx = egui::Context::default();
        crate::svg_loader::install_image_loaders(&ctx);
        IconRegistry::install(&ctx);

        ctx.forget_all_images();

        IconRegistry::install(&ctx);

        let result = ctx.try_load_bytes(&Icon::Refresh.uri());
        assert!(
            result.is_ok(),
            "Icon bytes must be available after forget_all_images + re-install"
        );
    }
}