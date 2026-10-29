//! These create only unmapped/hidden test windows; never borrow a user's window.
use super::*;
use raw_window_handle::{DisplayHandle, HandleError, RawDisplayHandle, WindowHandle};

#[cfg(windows)]
#[test]
fn native_windows_opacity_round_trips_on_a_hidden_window_and_preserves_style() {
    use raw_window_handle::{Win32WindowHandle, WindowsDisplayHandle};
    use std::{num::NonZeroIsize, ptr};
    use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};

    struct HiddenWindow(HWND);
    impl Drop for HiddenWindow {
        fn drop(&mut self) {
            // SAFETY: this scope owns the window and remains on its creating thread.
            unsafe {
                DestroyWindow(self.0);
            }
        }
    }
    impl HasWindowHandle for HiddenWindow {
        fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
            let window = Win32WindowHandle::new(NonZeroIsize::new(self.0 as isize).unwrap());
            // SAFETY: self owns the live HWND until after this borrow expires.
            unsafe { Ok(WindowHandle::borrow_raw(RawWindowHandle::Win32(window))) }
        }
    }
    impl HasDisplayHandle for HiddenWindow {
        fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
            // SAFETY: Win32 display handles carry no external pointer/resource.
            unsafe {
                Ok(DisplayHandle::borrow_raw(RawDisplayHandle::Windows(
                    WindowsDisplayHandle::new(),
                )))
            }
        }
    }
    // SAFETY: STATIC is a built-in class, strings are terminated, and no parent,
    // menu, instance data or creation parameter is provided. WS_VISIBLE is absent.
    let window = unsafe {
        const CLASS: &[u16] = &[83, 84, 65, 84, 73, 67, 0];
        CreateWindowExW(
            WS_EX_TOOLWINDOW,
            CLASS.as_ptr(),
            CLASS.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            16,
            16,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
        )
    };
    assert!(!window.is_null(), "{}", std::io::Error::last_os_error());
    let owner = HiddenWindow(window);
    // SAFETY: owned live HWND, no pointers/buffers.
    let original = unsafe { GetWindowLongPtrW(window, GWL_EXSTYLE) };
    for (opacity, expected) in [
        (WindowOpacity::Transparent, 191),
        (WindowOpacity::Opaque, 255),
    ] {
        apply(&owner, opacity).unwrap();
        let mut alpha = 0;
        let mut flags = 0;
        // SAFETY: owned live HWND and valid writable outputs for the call.
        let result =
            unsafe { GetLayeredWindowAttributes(window, ptr::null_mut(), &mut alpha, &mut flags) };
        assert_ne!(result, 0, "{}", std::io::Error::last_os_error());
        assert_eq!(alpha, expected);
        assert_eq!(flags, LWA_ALPHA);
        // SAFETY: same live HWND; style query does not retain resources.
        assert_eq!(
            unsafe { GetWindowLongPtrW(window, GWL_EXSTYLE) },
            original | WS_EX_LAYERED as isize
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires an X11 display; CI runs this explicitly under Xvfb"]
fn native_x11_opacity_round_trips_a_32_bit_cardinal_on_an_unmapped_window() {
    use raw_window_handle::{XlibDisplayHandle, XlibWindowHandle};
    use std::{
        ffi::c_ulong,
        ptr::{self, NonNull},
    };
    use x11_dl::xlib;

    struct HiddenWindow {
        api: xlib::Xlib,
        display: NonNull<xlib::Display>,
        window: c_ulong,
    }
    impl Drop for HiddenWindow {
        fn drop(&mut self) {
            // SAFETY: this guard exclusively owns both resources, created below.
            unsafe {
                (self.api.XDestroyWindow)(self.display.as_ptr(), self.window);
                (self.api.XCloseDisplay)(self.display.as_ptr());
            }
        }
    }
    impl HasWindowHandle for HiddenWindow {
        fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
            // SAFETY: the guard keeps its XID alive through the returned borrow.
            unsafe {
                Ok(WindowHandle::borrow_raw(RawWindowHandle::Xlib(
                    XlibWindowHandle::new(self.window),
                )))
            }
        }
    }
    impl HasDisplayHandle for HiddenWindow {
        fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
            // SAFETY: guard keeps the connection alive, on this single test thread.
            unsafe {
                Ok(DisplayHandle::borrow_raw(RawDisplayHandle::Xlib(
                    XlibDisplayHandle::new(Some(self.display.cast()), 0),
                )))
            }
        }
    }
    let api = xlib::Xlib::open().unwrap();
    // SAFETY: NULL selects DISPLAY; this test creates its own connection and
    // window, with valid root/size. The window is never mapped onto the desktop.
    let owner = unsafe {
        let display = NonNull::new((api.XOpenDisplay)(ptr::null())).expect("Xvfb DISPLAY required");
        let root = (api.XDefaultRootWindow)(display.as_ptr());
        let window = (api.XCreateSimpleWindow)(display.as_ptr(), root, 0, 0, 16, 16, 0, 0, 0);
        HiddenWindow {
            api,
            display,
            window,
        }
    };
    for (opacity, expected) in [
        (WindowOpacity::Transparent, 0xbfff_ffff),
        (WindowOpacity::Opaque, 0xffff_ffff),
    ] {
        apply(&owner, opacity).unwrap();
        let mut kind = 0;
        let mut format = 0;
        let mut count = 0;
        let mut after = 0;
        let mut data = ptr::null_mut();
        // SAFETY: same live connection/window, valid outputs, one property word.
        // Xlib's returned allocation is read only when it reports one format-32
        // item, and always released by XFree before any assertions can unwind.
        let (status, value) = unsafe {
            let atom = (owner.api.XInternAtom)(
                owner.display.as_ptr(),
                c"_NET_WM_WINDOW_OPACITY".as_ptr(),
                1,
            );
            let status = (owner.api.XGetWindowProperty)(
                owner.display.as_ptr(),
                owner.window,
                atom,
                0,
                1,
                0,
                xlib::XA_CARDINAL,
                &mut kind,
                &mut format,
                &mut count,
                &mut after,
                &mut data,
            );
            let value = if !data.is_null() && count == 1 && format == 32 {
                Some(data.cast::<c_ulong>().read() as u32)
            } else {
                None
            };
            if !data.is_null() {
                (owner.api.XFree)(data.cast());
            }
            (status, value)
        };
        assert_eq!(status, 0);
        assert_eq!((kind, format, count, after), (xlib::XA_CARDINAL, 32, 1, 0));
        assert_eq!(value, Some(expected));
    }
}
