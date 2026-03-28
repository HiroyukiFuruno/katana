#![deny(warnings)]
//! KatanA UI library.
//!
//! Exposes main editor components, allowing access for testing and external tools.

pub mod about_info;
pub mod app_state;
pub mod font_loader;
pub mod html_renderer;
pub(crate) mod http_cache_loader;
pub mod i18n;
pub mod icon;
pub use icon::*;
pub mod changelog;
pub mod diagram_controller;
pub mod preview_pane;
pub mod preview_pane_ui;
pub mod settings_window;
pub mod shell;
pub mod shell_logic;
pub mod shell_ui;
pub mod svg_loader;
pub mod theme_bridge;
pub mod widgets;
