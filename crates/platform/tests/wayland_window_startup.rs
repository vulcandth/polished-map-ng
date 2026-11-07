//! Exercise production egui-winit startup against a private protocol peer.
//! The peer verifies fullscreen priority; no GPU or desktop is accessed.
#![cfg(target_os = "linux")]
#![allow(unsafe_code)]

use egui_winit::egui::{Context, ViewportBuilder};
use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use wayland_sys::client::wayland_client_handle;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::wayland::EventLoopBuilderExtWayland,
    window::{Window, WindowId},
};

struct Probe {
    fullscreen: bool,
    maximized: bool,
    window: Option<Window>,
    destroyed: bool,
}

impl ApplicationHandler for Probe {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let builder = ViewportBuilder::default()
            .with_title("Isolated startup probe")
            .with_visible(false)
            .with_inner_size([800.0, 600.0])
            .with_fullscreen(self.fullscreen)
            .with_maximized(self.maximized);
        let window = egui_winit::create_window(&Context::default(), event_loop, &builder).unwrap();
        assert_eq!(window.fullscreen().is_some(), self.fullscreen);
        if !self.fullscreen {
            assert_eq!(window.is_maximized(), self.maximized);
        }
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.window.take();
            }
            WindowEvent::Destroyed => {
                self.destroyed = true;
                let RawDisplayHandle::Wayland(display) =
                    event_loop.display_handle().unwrap().as_raw()
                else {
                    panic!("probe must use its private Wayland connection");
                };
                // SAFETY: the event loop owns this live display on the sole
                // dispatch thread. A roundtrip drains teardown before exit.
                assert!(
                    unsafe {
                        (wayland_client_handle().wl_display_roundtrip)(
                            display.display.as_ptr().cast(),
                        )
                    } >= 0
                );
                event_loop.exit();
            }
            _ => {}
        }
    }
}

#[test]
#[ignore = "tools/wayland_window_startup_probe.py supplies the isolated peer"]
fn native_wayland_window_startup() {
    let scenario = std::env::var("POLISHED_MAP_WAYLAND_STARTUP_PROBE").unwrap();
    let (fullscreen, maximized) = match scenario.as_str() {
        "fullscreen" => (true, false),
        "fullscreen-and-maximized" => (true, true),
        "maximized" => (false, true),
        "normal" => (false, false),
        _ => panic!("unknown startup scenario"),
    };
    let event_loop = EventLoop::builder()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let mut probe = Probe {
        fullscreen,
        maximized,
        window: None,
        destroyed: false,
    };
    event_loop.run_app(&mut probe).unwrap();
    assert!(probe.destroyed);
}
