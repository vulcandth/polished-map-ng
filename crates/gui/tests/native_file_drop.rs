//! Exercise the exact vendored transfer helpers without winit's example/dev graph.
pub use egui_winit::winit::{dpi, event};

#[path = "../../../vendor/winit-0.30.13/src/file_drop.rs"]
mod file_drop;
