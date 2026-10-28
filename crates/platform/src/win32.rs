//! Windows ChooseColorW bridge matching master palette-window.cpp.
#![allow(unsafe_code)]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::Controls::Dialogs::{
    CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW, ChooseColorW, CommDlgExtendedError,
};

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
        // The GUI invokes this on eframe's UI thread with its current Frame.
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
