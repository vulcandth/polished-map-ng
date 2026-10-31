//! Narrow platform interop for native dialogs, window opacity and script editing.
//! LGPL-3.0-only. GUI, project and core crates retain their unsafe-code prohibition.
mod legacy_home;
mod opacity;
mod script;
pub use legacy_home::legacy_macos_home;
pub use opacity::{OpacityError, WindowOpacity, set_window_opacity};
pub use script::{ScriptOpenError, ScriptOpenRequest};
#[cfg(windows)]
mod script_windows;
#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::ColorPicker;

/// Windows' configured message-box font family, as used by upstream themes.cpp.
/// Other platforms use the reference's ordered family lists instead.
pub fn message_font_family() -> Option<String> {
    #[cfg(windows)]
    {
        win32::message_font_family()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Current Windows roaming profile, using the same redirected-folder lookup as
/// pinned FLTK. Read-only; does not create folders. Other platforms return None.
pub fn legacy_roaming_appdata() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        win32::legacy_roaming_appdata()
    }
    #[cfg(not(windows))]
    {
        None
    }
}
