//! Windows ChooseColorW bridge matching master palette-window.cpp.
#![allow(unsafe_code)]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::Controls::Dialogs::{
    CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW, ChooseColorW, CommDlgExtendedError,
};

pub(super) fn legacy_roaming_appdata() -> Option<std::path::PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::UI::Shell::{CSIDL_APPDATA, SHGFP_TYPE_CURRENT, SHGetFolderPathW};
    let mut buffer = [0u16; 260];
    // SAFETY: writable MAX_PATH buffer for this synchronous query; null handles
    // select the current user. No CSIDL_FLAG_CREATE, so this cannot create data.
    let result = unsafe {
        SHGetFolderPathW(
            std::ptr::null_mut(),
            CSIDL_APPDATA as i32,
            std::ptr::null_mut(),
            SHGFP_TYPE_CURRENT as u32,
            buffer.as_mut_ptr(),
        )
    };
    if result != 0 {
        return None;
    }
    let end = buffer.iter().position(|&c| c == 0)?;
    (end > 0).then(|| std::ffi::OsString::from_wide(&buffer[..end]).into())
}

pub(super) fn message_font_family() -> Option<String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, SystemParametersInfoW,
    };
    // SAFETY: NONCLIENTMETRICSW is a plain Win32 output struct; zero initializes
    // its integer/LOGFONT fields, then cbSize supplies the required ABI size.
    let mut metrics: NONCLIENTMETRICSW = unsafe { std::mem::zeroed() };
    metrics.cbSize = std::mem::size_of::<NONCLIENTMETRICSW>() as u32;
    // SAFETY: writable, aligned output remains alive throughout this synchronous
    // read-only system-settings query. No callback, owner or persistence flags.
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            (&mut metrics as *mut NONCLIENTMETRICSW).cast(),
            0,
        )
    };
    if ok == 0 {
        return None;
    }
    let name = &metrics.lfMessageFont.lfFaceName;
    let length = name.iter().position(|c| *c == 0)?;
    (length > 0)
        .then(|| String::from_utf16(&name[..length]).ok())
        .flatten()
}

/// Custom colors persist for this application session, without mutable globals.
#[derive(Default)]
pub struct ColorPicker {
    custom_colors: [u32; 16],
}
impl ColorPicker {
    /// Run the native modal chooser on the owner's UI thread. Cancel leaves the
    /// caller's color unchanged; genuine common-dialog failures return an error.
    pub fn choose(
        &mut self,
        owner: &impl HasWindowHandle,
        rgb: [u8; 3],
    ) -> Result<Option<[u8; 3]>, String> {
        let handle = owner.window_handle().map_err(|e| e.to_string())?;
        let RawWindowHandle::Win32(win) = handle.as_raw() else {
            return Err("A Windows window is required for the color chooser".into());
        };
        let mut options = CHOOSECOLORW {
            lStructSize: std::mem::size_of::<CHOOSECOLORW>() as u32,
            hwndOwner: win.hwnd.get() as _,
            rgbResult: u32::from(rgb[0]) | (u32::from(rgb[1]) << 8) | (u32::from(rgb[2]) << 16),
            lpCustColors: self.custom_colors.as_mut_ptr(),
            Flags: CC_FULLOPEN | CC_RGBINIT,
            ..Default::default()
        };
        // SAFETY: options has the correct ABI and size; the borrowed owner handle
        // and exclusively borrowed writable 16-COLORREF array remain valid during
        // this synchronous call. No hook/template flags or pointers are enabled.
        // The GUI invokes this on its UI thread with a live palette-window owner
        // (or eframe's root Frame for the embedded fallback).
        if unsafe { ChooseColorW(&mut options) } != 0 {
            let c = options.rgbResult;
            Ok(Some([c as u8, (c >> 8) as u8, (c >> 16) as u8]))
        } else {
            // SAFETY: Parameterless status query immediately after this thread's
            // common dialog returned, with no intervening common-dialog call.
            let error = unsafe { CommDlgExtendedError() };
            if error == 0 {
                Ok(None)
            } else {
                Err(format!("Windows color chooser failed (0x{error:08X})"))
            }
        }
    }
}
