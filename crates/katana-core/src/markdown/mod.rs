/* WHY: Markdown parsing pipeline using `comrak`. */

pub mod color_preset;
pub mod diagram;
pub mod drawio_renderer;
pub mod export;
pub mod fence;
pub mod mermaid_renderer;
pub mod outline;
pub mod plantuml_renderer;
pub mod render;
pub mod svg_rasterize;

// WHY: Re-exports
pub use diagram::{DiagramBlock, DiagramKind, DiagramRenderer, DiagramResult, NoOpRenderer};
pub use export::{HtmlExporter, ImageExporter, PdfExporter};
pub use render::*;
