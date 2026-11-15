//! Native map editor and deterministic egui embedding surface.
//!
//! Desktop callers normally use [`NativeStartup::discover_unified`] and
//! [`NativeStartup::run`]. [`Editor::new_unified`] embeds the same editor in an
//! existing eframe application. The fixed-format constructors remain available
//! for existing launchers; they share the editor implementation.
//!
//! # Contributor entry points
//!
//! `editor` owns session state and coordinates feature modules. Its `startup`
//! module builds sessions, `frame` drives input/drawing, `documents` dispatches
//! storage-specific operations, and `widgets` contains reusable controls.
//! [`geometry`] holds testable layout calculations without creating a window.
//! Editing and persistence belong in `polished_map_core` and
//! `polished_map_project`, respectively.
//!
//! LGPL-3.0-only; upstream adaptations retain attribution in their modules.

#![deny(missing_docs)]

mod editor;
pub mod geometry;

#[cfg(feature = "native-tests")]
pub use editor::run_native_preferences_probe;
pub use editor::{Editor, NativeStartup, WINDOW_TITLE, application_icon};
