//! HTML element model for structured rendering.
//!
//! Provides a UI-independent representation of HTML elements found in Markdown,
//! enabling proper inline/block display mode classification, link resolution,
//! and testable parsing without egui dependencies.

mod node;
mod parser;

pub use node::{DisplayMode, HtmlNode, LinkAction, LinkTarget, TextAlign};
pub use parser::HtmlParser;
