/* WHY: Mermaid CLI (`mmdc`) subprocess renderer.
Calls the system-installed `mmdc`, converts Mermaid source to SVG and returns it.
MVP constraints:
- Only works if `mmdc` is on the system PATH.
- Alternative binary path can be specified via the `MERMAID_MMDC` environment variable.
- Input is raw Mermaid source (excluding code fence markers). */

pub mod render;
pub mod resolve;

pub use render::*;
pub use resolve::*;
