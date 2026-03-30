#![deny(
    warnings,
    clippy::all,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::wildcard_imports,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_or_default,
    clippy::match_wild_err_arm,
    clippy::let_and_return,
    clippy::manual_ok_err
)]
#![warn(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::missing_errors_doc,
    missing_docs
)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::expect_used,
        clippy::indexing_slicing
    )
)]

pub mod ai;
pub mod document;
pub mod emoji;
pub mod html;
pub mod markdown;
pub mod plugin;
pub mod preview;
pub mod update;
pub mod workspace;

pub use document::Document;
pub use workspace::Workspace;
