//! Narrow platform interop for the original application's native dialogs.
//! LGPL-3.0-only. GUI, project and core crates retain their unsafe-code prohibition.
#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::ColorPicker;
