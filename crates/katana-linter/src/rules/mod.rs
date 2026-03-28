#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::expect_used,
    clippy::indexing_slicing
)]

pub mod coding;
pub mod domains;
pub mod structure;

pub use coding::{
    lint_comment_style, lint_error_first, lint_lazy_code, lint_magic_numbers, lint_performance,
    lint_prohibited_attributes, lint_prohibited_types,
};

pub use structure::{lint_file_length, lint_function_length, lint_nesting_depth, lint_pub_free_fn};

pub use domains::font_normalization::lint_font_normalization;
