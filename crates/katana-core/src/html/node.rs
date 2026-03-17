//! HTML element node model with display mode classification.

use std::path::{Path, PathBuf};

// ───────────────────── Display Mode ─────────────────────

/// Whether an HTML element creates line breaks (block) or flows inline.
///
/// Corresponds to the CSS `display` property for standard HTML elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Block elements (`<div>`, `<p>`, `<h1>`–`<h6>`) generate line breaks
    /// before and after.
    Block,
    /// Inline elements (`<a>`, `<img>`, `<span>`, `<em>`, `<strong>`)
    /// flow horizontally without line breaks.
    Inline,
}

// ───────────────────── Text Alignment ─────────────────────

/// Horizontal text alignment for block elements (e.g. `<p align="center">`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

// ───────────────────── Link Target ─────────────────────

/// Classification of a link destination.
///
/// Determined at parse time from the `href` attribute or markdown link URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkTarget {
    /// External URL (`http://` or `https://`).
    External(String),
    /// Internal file link (relative path resolved to absolute).
    InternalFile(PathBuf),
    /// Same-document anchor (`#section-id`).
    Anchor(String),
}

impl LinkTarget {
    /// Classifies a link href into the appropriate target variant.
    ///
    /// - URLs starting with `http://` or `https://` → `External`
    /// - URLs starting with `#` → `Anchor`
    /// - Everything else → `InternalFile` (resolved against `base_dir`)
    pub fn resolve(href: &str, base_dir: &Path) -> Self {
        if href.starts_with("http://") || href.starts_with("https://") {
            Self::External(href.to_string())
        } else if let Some(anchor) = href.strip_prefix('#') {
            Self::Anchor(anchor.to_string())
        } else {
            Self::InternalFile(base_dir.join(href))
        }
    }
}

// ───────────────────── Link Action ─────────────────────

/// How a link should be opened when clicked.
///
/// New variants (e.g. `OpenInNewTab`, `OpenInSplitView`) can be added here
/// as navigation features are implemented. The compiler's exhaustive match
/// checking ensures all call sites handle the new variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkAction {
    /// Open in the system's default web browser.
    OpenInBrowser(String),
    /// Navigate within the current editor tab (supports history back/forward).
    NavigateCurrentTab(PathBuf),
}

impl LinkTarget {
    /// Returns the default action for this link target type.
    pub fn default_action(&self) -> LinkAction {
        match self {
            Self::External(url) => LinkAction::OpenInBrowser(url.clone()),
            Self::InternalFile(path) => LinkAction::NavigateCurrentTab(path.clone()),
            Self::Anchor(anchor) => {
                LinkAction::NavigateCurrentTab(PathBuf::from(format!("#{anchor}")))
            }
        }
    }

    /// Returns the text to display in tooltip.
    pub fn tooltip_text(&self) -> String {
        match self {
            Self::External(url) => url.clone(),
            Self::InternalFile(path) => path.to_string_lossy().into_owned(),
            Self::Anchor(anchor) => format!("#{anchor}"),
        }
    }
}

// ───────────────────── HTML Node ─────────────────────

/// A parsed HTML/Markdown element in the preview tree.
///
/// This enum represents the subset of HTML elements that can appear within
/// Markdown content. It is UI-independent and lives in `katana-core`.
#[derive(Debug, Clone, PartialEq)]
pub enum HtmlNode {
    /// Plain text content.
    Text(String),

    /// Image element (`<img>` or `![alt](src)`).
    Image {
        /// Image source URL or file path.
        src: String,
        /// Alt text for accessibility.
        alt: String,
    },

    /// Link element (`<a>` or `[text](url)`).
    Link {
        /// Classified link destination.
        target: LinkTarget,
        /// Child nodes (text, images, etc.) rendered inside the link.
        children: Vec<HtmlNode>,
    },

    /// Heading element (`<h1>`–`<h6>` or `# heading`).
    Heading {
        /// Heading level (1–6).
        level: u8,
        /// Horizontal alignment (from `align` attribute).
        align: Option<TextAlign>,
        /// Child nodes rendered inside the heading.
        children: Vec<HtmlNode>,
    },

    /// Paragraph element (`<p>`) with optional alignment.
    Paragraph {
        /// Horizontal alignment (from `align` attribute).
        align: Option<TextAlign>,
        /// Child nodes rendered inside the paragraph.
        children: Vec<HtmlNode>,
    },

    /// Line break (`<br>`).
    LineBreak,

    /// Emphasis (`<em>` or `*text*`).
    Emphasis(Vec<HtmlNode>),

    /// Strong emphasis (`<strong>` or `**text**`).
    Strong(Vec<HtmlNode>),
}

impl HtmlNode {
    /// Returns whether this node is a block or inline element.
    pub fn display_mode(&self) -> DisplayMode {
        match self {
            Self::Heading { .. } | Self::Paragraph { .. } => DisplayMode::Block,
            Self::Text(_)
            | Self::Image { .. }
            | Self::Link { .. }
            | Self::LineBreak
            | Self::Emphasis(_)
            | Self::Strong(_) => DisplayMode::Inline,
        }
    }

    /// Shorthand for `self.display_mode() == DisplayMode::Inline`.
    pub fn is_inline(&self) -> bool {
        self.display_mode() == DisplayMode::Inline
    }

    /// Shorthand for `self.display_mode() == DisplayMode::Block`.
    pub fn is_block(&self) -> bool {
        self.display_mode() == DisplayMode::Block
    }
}

// ───────────────────── Tests ─────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // ── DisplayMode tests ──

    #[test]
    fn text_is_inline() {
        let node = HtmlNode::Text("hello".into());
        assert_eq!(node.display_mode(), DisplayMode::Inline);
        assert!(node.is_inline());
        assert!(!node.is_block());
    }

    #[test]
    fn image_is_inline() {
        let node = HtmlNode::Image {
            src: "icon.png".into(),
            alt: "icon".into(),
        };
        assert_eq!(node.display_mode(), DisplayMode::Inline);
    }

    #[test]
    fn link_is_inline() {
        let node = HtmlNode::Link {
            target: LinkTarget::External("https://example.com".into()),
            children: vec![HtmlNode::Text("click".into())],
        };
        assert_eq!(node.display_mode(), DisplayMode::Inline);
    }

    #[test]
    fn linebreak_is_inline() {
        assert_eq!(HtmlNode::LineBreak.display_mode(), DisplayMode::Inline);
    }

    #[test]
    fn emphasis_is_inline() {
        let node = HtmlNode::Emphasis(vec![HtmlNode::Text("em".into())]);
        assert_eq!(node.display_mode(), DisplayMode::Inline);
    }

    #[test]
    fn strong_is_inline() {
        let node = HtmlNode::Strong(vec![HtmlNode::Text("bold".into())]);
        assert_eq!(node.display_mode(), DisplayMode::Inline);
    }

    #[test]
    fn heading_is_block() {
        let node = HtmlNode::Heading {
            level: 1,
            align: None,
            children: vec![HtmlNode::Text("title".into())],
        };
        assert_eq!(node.display_mode(), DisplayMode::Block);
        assert!(node.is_block());
        assert!(!node.is_inline());
    }

    #[test]
    fn paragraph_is_block() {
        let node = HtmlNode::Paragraph {
            align: None,
            children: vec![HtmlNode::Text("text".into())],
        };
        assert_eq!(node.display_mode(), DisplayMode::Block);
    }

    // ── LinkTarget tests ──

    #[test]
    fn tooltip_text_external() {
        let target = LinkTarget::External("https://example.com".into());
        assert_eq!(target.tooltip_text(), "https://example.com");
    }

    #[test]
    fn tooltip_text_internal() {
        let target = LinkTarget::InternalFile(PathBuf::from("/path/to/file.md"));
        assert_eq!(target.tooltip_text(), "/path/to/file.md");
    }

    #[test]
    fn tooltip_text_anchor() {
        let target = LinkTarget::Anchor("section-2".into());
        assert_eq!(target.tooltip_text(), "#section-2");
    }

    #[test]
    fn resolve_external_https() {
        let target = LinkTarget::resolve("https://github.com/org/repo", Path::new("/project"));
        assert_eq!(
            target,
            LinkTarget::External("https://github.com/org/repo".into())
        );
        assert_eq!(
            target.default_action(),
            LinkAction::OpenInBrowser("https://github.com/org/repo".into())
        );
    }

    #[test]
    fn resolve_external_http() {
        let target = LinkTarget::resolve("http://example.com", Path::new("/project"));
        assert_eq!(target, LinkTarget::External("http://example.com".into()));
        assert_eq!(
            target.default_action(),
            LinkAction::OpenInBrowser("http://example.com".into())
        );
    }

    #[test]
    fn resolve_internal_file() {
        let target = LinkTarget::resolve("README.ja.md", Path::new("/project"));
        assert_eq!(
            target,
            LinkTarget::InternalFile(PathBuf::from("/project/README.ja.md"))
        );
        assert_eq!(
            target.default_action(),
            LinkAction::NavigateCurrentTab(PathBuf::from("/project/README.ja.md"))
        );
    }

    #[test]
    fn resolve_anchor() {
        let target = LinkTarget::resolve("#installation", Path::new("/project"));
        assert_eq!(target, LinkTarget::Anchor("installation".into()));
        assert_eq!(
            target.default_action(),
            LinkAction::NavigateCurrentTab(PathBuf::from("#installation"))
        );
    }

    #[test]
    fn resolve_relative_path_with_subdirectory() {
        let target = LinkTarget::resolve("docs/guide.md", Path::new("/project"));
        assert_eq!(
            target,
            LinkTarget::InternalFile(PathBuf::from("/project/docs/guide.md"))
        );
    }

    #[test]
    fn resolve_license_file() {
        let target = LinkTarget::resolve("LICENSE", Path::new("/project"));
        assert_eq!(
            target,
            LinkTarget::InternalFile(PathBuf::from("/project/LICENSE"))
        );
        assert_eq!(
            target.default_action(),
            LinkAction::NavigateCurrentTab(PathBuf::from("/project/LICENSE"))
        );
    }
}
