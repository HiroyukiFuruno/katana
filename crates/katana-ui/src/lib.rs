#![deny(warnings, clippy::all)]
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]

pub mod about_info;
pub mod app;
pub mod app_state;
pub mod font_loader;
pub mod html_renderer;
pub(crate) mod http_cache_loader;
pub mod i18n;
pub mod icon;
pub use icon::*;
pub mod changelog;
pub mod diagram_controller;
pub mod native_menu;
pub mod preview_pane;
pub mod settings;
pub mod shell;
pub mod shell_logic;
pub mod shell_ui;
pub mod svg_loader;
pub mod theme_bridge;
pub mod widgets;

pub mod state;
pub mod views;