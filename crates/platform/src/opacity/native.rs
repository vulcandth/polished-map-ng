//! Borrowed native handles; no pointers or OS objects escape these calls.
#![allow(unsafe_code)]
use super::*;
use raw_window_handle::RawWindowHandle;

#[cfg(all(test, any(windows, target_os = "linux")))]
mod tests;

#[cfg(windows)]
pub(super) fn apply(
    owner: &(impl HasWindowHandle + HasDisplayHandle),
    opacity: WindowOpacity,
) -> Result<(), OpacityError> {
    use windows_sys::Win32::{
        Foundation::{GetLastError, SetLastError},
        UI::WindowsAndMessaging::{
            GWL_EXSTYLE, GetWindowLongPtrW, LWA_ALPHA, SetLayeredWindowAttributes,
            SetWindowLongPtrW, WS_EX_LAYERED,
        },
    };
    let handle = owner.window_handle()?;
    let RawWindowHandle::Win32(window) = handle.as_raw() else {
        return Err(OpacityError::Unsupported(
            "A Win32 window is required for window opacity",
        ));
    };
    let hwnd = window.hwnd.get() as _;
    // SAFETY: handle borrows the live owner for the entire call. These APIs use
    // no caller buffers. Zero is a valid previous style, so inspect LastError.
    unsafe {
        SetLastError(0);
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        if style == 0 && GetLastError() != 0 {
            return Err(OpacityError::Native(
                std::io::Error::last_os_error().to_string(),
            ));
        }
        let added_layer = style & WS_EX_LAYERED as isize == 0;
        if added_layer {
            SetLastError(0);
            let previous = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED as isize);
            if previous == 0 && GetLastError() != 0 {
                return Err(OpacityError::Native(
                    std::io::Error::last_os_error().to_string(),
                ));
            }
        }
        if SetLayeredWindowAttributes(hwnd, 0, opacity.alpha8(), LWA_ALPHA) == 0 {
            let mut error = std::io::Error::last_os_error().to_string();
            if added_layer {
                SetLastError(0);
                let previous = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style);
                if previous == 0 && GetLastError() != 0 {
                    error.push_str(&format!(
                        "; could not restore window style: {}",
                        std::io::Error::last_os_error()
                    ));
                }
            }
            return Err(OpacityError::Native(error));
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub(super) fn apply(
    owner: &(impl HasWindowHandle + HasDisplayHandle),
    opacity: WindowOpacity,
) -> Result<(), OpacityError> {
    let handle = owner.window_handle()?;
    let RawWindowHandle::AppKit(window) = handle.as_raw() else {
        return Err(OpacityError::Unsupported(
            "An AppKit window is required for window opacity",
        ));
    };
    let _main_thread = objc2::MainThreadMarker::new().ok_or(OpacityError::Unsupported(
        "Window opacity must be changed on the main UI thread",
    ))?;
    // SAFETY: raw-window-handle guarantees this is a live NSView, borrowed from
    // owner for this scope. MainThreadMarker verifies AppKit's thread contract.
    let view = unsafe { window.ns_view.cast::<objc2_app_kit::NSView>().as_ref() };
    let window = view.window().ok_or(OpacityError::Unsupported(
        "The view is not attached to a window",
    ))?;
    window.setAlphaValue(opacity.alpha());
    Ok(())
}

#[cfg(target_os = "linux")]
pub(super) fn apply(
    owner: &(impl HasWindowHandle + HasDisplayHandle),
    opacity: WindowOpacity,
) -> Result<(), OpacityError> {
    use raw_window_handle::RawDisplayHandle;
    use std::ffi::c_ulong;
    use x11_dl::xlib;

    let handle = owner.window_handle()?;
    if matches!(handle.as_raw(), RawWindowHandle::Wayland(_)) {
        return Err(OpacityError::WaylandProtocolRequired);
    }
    let display_handle = owner.display_handle()?;
    let (RawWindowHandle::Xlib(window), RawDisplayHandle::Xlib(display)) =
        (handle.as_raw(), display_handle.as_raw())
    else {
        return Err(OpacityError::Unsupported(
            "An Xlib window and display are required for window opacity",
        ));
    };
    let display = display
        .display
        .ok_or(OpacityError::Unsupported("The Xlib display is unavailable"))?;
    let xlib = xlib::Xlib::open().map_err(|error| OpacityError::Native(error.to_string()))?;
    let display = display.as_ptr().cast::<xlib::Display>();
    // XChangeProperty format=32 takes native unsigned longs, even on LP64.
    // Do not pass a u32 pointer: Xlib would read beyond it on 64-bit Linux.
    let value = c_ulong::from(opacity.alpha32());
    // SAFETY: both handles are borrowed from the same live owner. Xlib access
    // is serialized with its display lock, the atom string is NUL terminated,
    // and one native long remains alive until XChangeProperty copies it.
    unsafe {
        (xlib.XLockDisplay)(display);
        let atom = (xlib.XInternAtom)(display, c"_NET_WM_WINDOW_OPACITY".as_ptr(), 0);
        let result = if atom == 0 {
            Err(OpacityError::Native(
                "Could not create the X11 opacity atom".into(),
            ))
        } else {
            (xlib.XChangeProperty)(
                display,
                window.window,
                atom,
                xlib::XA_CARDINAL,
                32,
                xlib::PropModeReplace,
                (&value as *const c_ulong).cast(),
                1,
            );
            (xlib.XFlush)(display);
            Ok(())
        };
        (xlib.XUnlockDisplay)(display);
        result
    }
}
