pub mod comment_style;
pub mod error_first;
pub mod lazy_code;
pub mod magic_numbers;
pub mod performance;
pub mod prohibited_attrs;
pub mod prohibited_types;

pub use comment_style::lint_comment_style;
pub use error_first::lint_error_first;
pub use lazy_code::lint_lazy_code;
pub use magic_numbers::lint_magic_numbers;
pub use performance::lint_performance;
pub use prohibited_attrs::lint_prohibited_attributes;
pub use prohibited_types::lint_prohibited_types;
