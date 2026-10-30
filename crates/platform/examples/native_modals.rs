//! Hidden native ownership/lifetime probe. No GUI automation or visible window.
//! Run explicitly; Linux requires a private Xvfb/Wayland peer.
#![allow(unsafe_code)]
use egui_winit::{
    egui::{Context, ViewportId},
    modal_windows,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

fn owner(child: &Window, expected: Option<&Window>) {
    #[cfg(windows)]
    {
        use windows_sys_legacy::Win32::UI::WindowsAndMessaging::{GW_OWNER, GetWindow};
        fn hwnd(w: &Window) -> isize {
            let RawWindowHandle::Win32(h) = w.window_handle().unwrap().as_raw() else {
                panic!("Win32 window")
            };
            h.hwnd.get()
        }
        // SAFETY: borrowed winit windows remain alive for these read-only calls.
        assert_eq!(
            unsafe { GetWindow(hwnd(child), GW_OWNER) },
            expected.map_or(0, hwnd)
        );
    }
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::NSView;
        fn native(w: &Window) -> objc2::rc::Retained<objc2_app_kit::NSWindow> {
            let RawWindowHandle::AppKit(h) = w.window_handle().unwrap().as_raw() else {
                panic!("AppKit window")
            };
            // SAFETY: a borrowed live NSView on the AppKit main thread.
            unsafe { &*h.ns_view.as_ptr().cast::<NSView>() }
                .window()
                .unwrap()
        }
        let actual = native(child).parentWindow();
        assert_eq!(
            actual.as_deref().map(|w| w as *const _),
            expected.map(native).as_deref().map(|w| w as *const _)
        );
    }
    #[cfg(target_os = "linux")]
    {
        if matches!(
            child.window_handle().unwrap().as_raw(),
            RawWindowHandle::Wayland(_)
        ) {
            return;
        }
        use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};
        fn xid(w: &Window) -> u32 {
            match w.window_handle().unwrap().as_raw() {
                RawWindowHandle::Xlib(h) => h.window.try_into().unwrap(),
                RawWindowHandle::Xcb(h) => h.window.get(),
                _ => panic!("X11 window"),
            }
        }
        let (conn, _) = x11rb::connect(None).unwrap();
        let atom = |name: &[u8]| conn.intern_atom(false, name).unwrap().reply().unwrap().atom;
        let parents = conn
            .get_property(
                false,
                xid(child),
                atom(b"WM_TRANSIENT_FOR"),
                AtomEnum::WINDOW,
                0,
                1,
            )
            .unwrap()
            .reply()
            .unwrap();
        assert_eq!(
            parents.value32().into_iter().flatten().collect::<Vec<_>>(),
            expected.map(xid).into_iter().collect::<Vec<_>>()
        );
        let state = conn
            .get_property(
                false,
                xid(child),
                atom(b"_NET_WM_STATE"),
                AtomEnum::ATOM,
                0,
                1024,
            )
            .unwrap()
            .reply()
            .unwrap();
        assert_eq!(
            state
                .value32()
                .into_iter()
                .flatten()
                .any(|v| v == atom(b"_NET_WM_STATE_MODAL")),
            expected.is_some()
        );
    }
}
fn enabled(window: &Window, expected: bool) {
    #[cfg(windows)]
    {
        let RawWindowHandle::Win32(h) = window.window_handle().unwrap().as_raw() else {
            panic!("Win32 window")
        };
        // SAFETY: read-only query of a borrowed live window.
        assert_eq!(
            unsafe {
                windows_sys_legacy::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled(
                    h.hwnd.get(),
                )
            } != 0,
            expected
        );
    }
    #[cfg(not(windows))]
    let _ = (window, expected);
}
fn register(
    ctx: &Context,
    id: ViewportId,
    window: &Arc<Window>,
    event_loop: &ActiveEventLoop,
) -> egui_winit::State {
    let mut state = egui_winit::State::new(
        ctx.clone(),
        id,
        event_loop,
        Some(window.scale_factor() as f32),
        None,
        None,
    );
    state.register_window(window);
    state
}
fn exercise(event_loop: &ActiveEventLoop) {
    let ctx = Context::default();
    let make = || {
        Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Polished Map isolated modal probe")
                        .with_visible(false)
                        .with_active(false),
                )
                .unwrap(),
        )
    };
    let root = make();
    let child = make();
    let nested = make();
    let child_id = ViewportId::from_hash_of("child");
    let nested_id = ViewportId::from_hash_of("nested");
    let root_state = register(&ctx, ViewportId::ROOT, &root, event_loop);
    let child_state = register(&ctx, child_id, &child, event_loop);
    let nested_state = register(&ctx, nested_id, &nested, event_loop);
    modal_windows::set_parent(&ctx, child_id, ViewportId::ROOT).unwrap();
    assert_eq!(modal_windows::take_error(&ctx), None);
    owner(&child, Some(&root));
    enabled(&root, false);
    enabled(&child, true);
    modal_windows::set_parent(&ctx, nested_id, child_id).unwrap();
    assert_eq!(modal_windows::take_error(&ctx), None);
    owner(&nested, Some(&child));
    enabled(&child, false);
    drop(nested_state);
    owner(&nested, None);
    enabled(&child, true);
    enabled(&root, false);
    // Recreate a renderer window before its old State is dropped.
    let replacement = make();
    let replacement_state = register(&ctx, child_id, &replacement, event_loop);
    owner(&child, None);
    enabled(&root, true);
    modal_windows::set_parent(&ctx, child_id, ViewportId::ROOT).unwrap();
    drop(child_state);
    owner(&replacement, Some(&root));
    enabled(&root, false);
    assert!(modal_windows::with_window(&ctx, child_id, |w| w.id() == replacement.id()).unwrap());
    // Parent teardown must release the whole chain before native destruction.
    drop(root_state);
    owner(&replacement, None);
    enabled(&root, true);
    assert!(!modal_windows::is_blocked(&ctx, ViewportId::ROOT));
    drop(replacement_state);
    assert!(modal_windows::with_window(&ctx, child_id, |_| ()).is_none());
    #[cfg(windows)]
    {
        use winit::{modal::ModalWindow, platform::windows::WindowExtWindows};
        root.set_enable(false);
        let lease = ModalWindow::new(child.clone(), root.clone()).unwrap();
        drop(lease);
        enabled(&root, false); // preserve a pre-existing disabled owner
        root.set_enable(true);
    }
}
struct Probe {
    started: Instant,
    destroyed: usize,
}
impl ApplicationHandler for Probe {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        exercise(event_loop);
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            self.started + Duration::from_secs(10),
        ));
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::Destroyed) {
            self.destroyed += 1;
            if self.destroyed == 4 {
                #[cfg(target_os = "linux")]
                {
                    use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
                    if let RawDisplayHandle::Wayland(display) =
                        event_loop.display_handle().unwrap().as_raw()
                    {
                        // SAFETY: live borrowed display, sole event-loop thread,
                        // outside protocol dispatch. Drain teardown before exit.
                        assert!(
                            unsafe {
                                (wayland_sys::client::wayland_client_handle().wl_display_roundtrip)(
                                    display.display.as_ptr().cast(),
                                )
                            } >= 0
                        );
                    }
                }
                event_loop.exit();
            }
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        assert!(
            self.started.elapsed() < Duration::from_secs(10),
            "native window teardown timed out"
        );
    }
}
fn main() {
    assert_eq!(
        std::env::var("POLISHED_MAP_NATIVE_MODAL_PROBE").as_deref(),
        Ok("1"),
        "run this probe explicitly in an isolated/native test environment"
    );
    let mut builder = EventLoop::builder();
    #[cfg(target_os = "linux")]
    if std::env::var_os("POLISHED_MAP_MODAL_WAYLAND").is_some() {
        use winit::platform::wayland::EventLoopBuilderExtWayland;
        builder.with_wayland();
    } else {
        use winit::platform::x11::EventLoopBuilderExtX11;
        builder.with_x11();
    }
    let event_loop = builder.build().unwrap();
    let mut probe = Probe {
        started: Instant::now(),
        destroyed: 0,
    };
    event_loop.run_app(&mut probe).unwrap();
    assert_eq!(probe.destroyed, 4);
    println!("Native modal ownership, nested restoration, recreation and teardown passed");
}
