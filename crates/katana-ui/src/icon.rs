#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Dot,
    ChevronLeft,
    ChevronRight,
    Refresh,
    Close,  // 'x'
    Remove, // '×' (U+00D7)
    ExternalLink,
    TriangleDown,
    TriangleLeft,
    TriangleRight,
    Search,
    Plus,
    Minus,
    Toc,
    // Viewer overlay controls
    PanUp,
    PanDown,
    PanLeft,
    PanRight,
    ZoomIn,
    ZoomOut,
    ResetView,
    Fullscreen,
    CloseModal,
}

impl Icon {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Dot => "●",
            Self::ChevronLeft => "‹",
            Self::ChevronRight => "›",
            Self::Refresh => "🔄",
            Self::Close => "x",
            Self::Remove => "×",
            Self::ExternalLink => "↗",
            Self::TriangleDown => "\u{25BC}", // ▼
            Self::TriangleLeft => "◀",
            Self::TriangleRight => "▶",
            Self::Search => "🔍",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Toc => "☰",
            Self::PanUp => "∧",
            Self::PanDown => "∨",
            Self::PanLeft => "<",
            Self::PanRight => ">",
            Self::ZoomIn => "⊕",
            Self::ZoomOut => "⊖",
            Self::ResetView => "↻",
            Self::Fullscreen => "⛶",
            Self::CloseModal => "✗",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_as_str() {
        assert_eq!(Icon::Dot.as_str(), "●");
        assert_eq!(Icon::ChevronLeft.as_str(), "‹");
        assert_eq!(Icon::ChevronRight.as_str(), "›");
        assert_eq!(Icon::Refresh.as_str(), "🔄");
        assert_eq!(Icon::Close.as_str(), "x");
        assert_eq!(Icon::Remove.as_str(), "×");
        assert_eq!(Icon::ExternalLink.as_str(), "↗");
        assert_eq!(Icon::TriangleDown.as_str(), "\u{25BC}");
        assert_eq!(Icon::TriangleLeft.as_str(), "◀");
        assert_eq!(Icon::TriangleRight.as_str(), "▶");
        assert_eq!(Icon::Search.as_str(), "🔍");
        assert_eq!(Icon::Plus.as_str(), "+");
        assert_eq!(Icon::Minus.as_str(), "-");
        assert_eq!(Icon::Toc.as_str(), "☰");
        assert_eq!(Icon::PanUp.as_str(), "∧");
        assert_eq!(Icon::PanDown.as_str(), "∨");
        assert_eq!(Icon::PanLeft.as_str(), "<");
        assert_eq!(Icon::PanRight.as_str(), ">");
        assert_eq!(Icon::ZoomIn.as_str(), "⊕");
        assert_eq!(Icon::ZoomOut.as_str(), "⊖");
        assert_eq!(Icon::ResetView.as_str(), "↻");
        assert_eq!(Icon::Fullscreen.as_str(), "⛶");
        assert_eq!(Icon::CloseModal.as_str(), "✗");
    }
}
