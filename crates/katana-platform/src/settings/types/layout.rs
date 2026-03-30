use serde::{Deserialize, Serialize};

// WHY: Split direction for editor/preview layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SplitDirection {
    // WHY: Editor on left, preview on right.
    #[default]
    Horizontal,
    // WHY: Editor on top, preview on bottom.
    Vertical,
}

// WHY: Position of the Table of Contents panel in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TocPosition {
    // WHY: Left side of the workspace.
    #[default]
    Left,
    // WHY: Right side of the workspace.
    Right,
}

// WHY: Pane order within the split view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PaneOrder {
    // WHY: Editor first (left or top), preview second.
    #[default]
    EditorFirst,
    // WHY: Preview first (left or top), editor second.
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

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            split_direction: Default::default(),
            pane_order: Default::default(),
            toc_visible: true,
            toc_position: Default::default(),
        }
    }
}