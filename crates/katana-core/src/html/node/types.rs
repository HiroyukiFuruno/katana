use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Block,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkTarget {
    External(String),
    InternalFile(PathBuf),
    Anchor(String),
}

/* WHY: New variants (e.g. `OpenInNewTab`, `OpenInSplitView`) can be added here
as navigation features are implemented. The compiler's exhaustive match
checking ensures all call sites handle the new variant. */
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkAction {
    OpenInBrowser(String),
    NavigateCurrentTab(PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub enum HtmlNode {
    Text(String),
    Image {
        src: String,
        alt: String,
    },
    Link {
        target: LinkTarget,
        children: Vec<HtmlNode>,
    },
    Heading {
        level: u8,
        align: Option<TextAlign>,
        children: Vec<HtmlNode>,
    },
    Paragraph {
        align: Option<TextAlign>,
        children: Vec<HtmlNode>,
    },
    LineBreak,
    Emphasis(Vec<HtmlNode>),
    Strong(Vec<HtmlNode>),
}
