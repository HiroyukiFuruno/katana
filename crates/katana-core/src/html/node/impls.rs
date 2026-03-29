use std::path::{Path, PathBuf};

use super::types::{DisplayMode, HtmlNode, LinkAction, LinkTarget};

impl LinkTarget {
    pub fn resolve(href: &str, base_dir: &Path) -> Self {
        if href.starts_with("http://") || href.starts_with("https://") {
            Self::External(href.to_string())
        } else if let Some(anchor) = href.strip_prefix('#') {
            Self::Anchor(anchor.to_string())
        } else {
            Self::InternalFile(base_dir.join(href))
        }
    }

    pub fn default_action(&self) -> LinkAction {
        match self {
            Self::External(url) => LinkAction::OpenInBrowser(url.clone()),
            Self::InternalFile(path) => LinkAction::NavigateCurrentTab(path.clone()),
            Self::Anchor(anchor) => {
                LinkAction::NavigateCurrentTab(PathBuf::from(format!("#{anchor}")))
            }
        }
    }

    pub fn tooltip_text(&self) -> String {
        match self {
            Self::External(url) => url.clone(),
            Self::InternalFile(path) => path.to_string_lossy().into_owned(),
            Self::Anchor(anchor) => format!("#{anchor}"),
        }
    }
}

impl HtmlNode {
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

    pub fn is_inline(&self) -> bool {
        self.display_mode() == DisplayMode::Inline
    }

    pub fn is_block(&self) -> bool {
        self.display_mode() == DisplayMode::Block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
