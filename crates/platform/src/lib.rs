//! Narrow platform interop for native dialogs and external event script editing.
//! LGPL-3.0-only. GUI, project and core crates retain their unsafe-code prohibition.
mod script;
pub use script::{ScriptOpenError, ScriptOpenRequest};
#[cfg(windows)]
mod script_windows;
#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::ColorPicker;
