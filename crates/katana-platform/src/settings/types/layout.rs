use serde::{Deserialize, Serialize};

/// Split direction for editor/preview layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SplitDirection {
    /// Editor on left, preview on right.
    #[default]
    Horizontal,
    /// Editor on top, preview on bottom.
    Vertical,
}

/// Position of the Table of Contents panel in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TocPosition {
    /// Left side of the workspace.
    #[default]
    Left,
    /// Right side of the workspace.
    Right,
}

/// Pane order within the split view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PaneOrder {
    /// Editor first (left or top), preview second.
    #[default]
    EditorFirst,
    /// Preview first (left or top), editor second.
    PreviewFirst,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    #[serde(default)]
    pub split_direction: SplitDirection,
    #[serde(default)]
    pub pane_order: PaneOrder,
    #[serde(default = "super::super::defaults::default_true")]
    pub toc_visible: bool,
    #[serde(default)]
    pub toc_position: TocPosition,
}
