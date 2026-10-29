//! Narrow platform interop for native dialogs, window opacity and script editing.
//! LGPL-3.0-only. GUI, project and core crates retain their unsafe-code prohibition.
mod opacity;
mod script;
pub use opacity::{OpacityError, WindowOpacity, set_window_opacity};
pub use script::{ScriptOpenError, ScriptOpenRequest};
#[cfg(windows)]
mod script_windows;
#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::ColorPicker;
