//! Test the exact adapter tracker without the vendored crate's example/dev graph.
extern crate self as winit;
pub use egui_winit::winit::{dpi, event};

#[path = "../../../vendor/eframe-0.37.0/src/native/normal_geometry.rs"]
mod normal_geometry;
