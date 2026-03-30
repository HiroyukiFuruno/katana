/* WHY: Settings type definitions.

All settings-related structs, enums, and constants are consolidated here. */

pub mod app;
pub mod behavior;
pub mod export;
pub mod font;
pub mod layout;
pub mod performance;
pub mod theme;
pub mod update;
pub mod workspace;

pub use app::*;
pub use behavior::*;
pub use export::*;
pub use font::*;
pub use layout::*;
pub use performance::*;
pub use theme::*;
pub use update::*;
pub use workspace::*;