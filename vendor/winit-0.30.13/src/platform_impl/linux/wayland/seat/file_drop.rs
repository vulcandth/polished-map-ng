//! Receive-only file drops on winit's own Wayland connection and event queue.
//! Polished Map NG extension, Apache-2.0. Clipboard selections remain with egui's adapter.
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::os::fd::{AsRawFd, OwnedFd};
use std::time::Duration;

use crate::dpi::{LogicalPosition, PhysicalPosition};
use crate::event::WindowEvent;
use crate::platform_impl::wayland::{make_wid, state::WinitState, WindowId};
use calloop::{
    generic::Generic,
    timer::{TimeoutAction, Timer},
    Interest, Mode, PostAction, RegistrationToken,
};
use sctk::data_device_manager::{
    data_device::{DataDeviceData, DataDeviceHandler},
    data_offer::{DataOfferData, DataOfferHandler, DragOffer},
    DataDeviceManagerState,
};
use sctk::globals::GlobalData;
use sctk::reexports::client::protocol::{
    wl_data_device::WlDataDevice,
    wl_data_device_manager::{DndAction, WlDataDeviceManager},
    wl_data_offer::WlDataOffer,
    wl_surface::WlSurface,
};
use sctk::reexports::client::{
    backend::ObjectId, delegate_dispatch, Connection, Proxy, QueueHandle,
};

const MIME: &str = "text/uri-list";

#[derive(Debug)]
pub(super) struct PendingDrop {
    offer: DragOffer,
    window: WindowId,
    position: PhysicalPosition<f64>,
    bytes: Vec<u8>,
    source: Option<RegistrationToken>,
    timeout: Option<RegistrationToken>,
}

impl WinitState {
    pub(super) fn cancel_file_drop(&mut self, seat: &ObjectId) {
        if let Some(pending) = self
            .seats
            .get_mut(seat)
            .and_then(|seat| seat.pending_drop.take())
        {
            if let Some(token) = pending.source {
                self.loop_handle.remove(token);
            }
            if let Some(token) = pending.timeout {
                self.loop_handle.remove(token);
            }
            pending.offer.destroy();
        }
    }

    fn finish_file_drop(&mut self, seat: &ObjectId, success: bool) {
        if let Some(pending) = self
            .seats
            .get_mut(seat)
            .and_then(|seat| seat.pending_drop.take())
        {
            if let Some(token) = pending.timeout {
                self.loop_handle.remove(token);
            }
            if success && self.windows.borrow().contains_key(&pending.window) {
                if let Some(paths) = crate::file_drop::uri_list(&pending.bytes) {
                    if let Some(event) =
                        crate::file_drop::completed(paths.clone(), pending.position)
                    {
                        self.events_sink.push_window_event(event, pending.window);
                        for path in paths {
                            self.events_sink
                                .push_window_event(WindowEvent::DroppedFile(path), pending.window);
                        }
                        self.dispatched_events = true;
                        pending.offer.finish();
                    }
                }
            }
            pending.offer.destroy();
        }
    }
}

impl DataDeviceHandler for WinitState {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        device: &WlDataDevice,
        _: f64,
        _: f64,
        surface: &WlSurface,
    ) {
        let Some(offer) = device
            .data::<DataDeviceData>()
            .and_then(|data| data.drag_offer())
        else {
            return;
        };
        // Only the actual content surface, not its decoration subsurfaces.
        let supported = self.windows.borrow().contains_key(&make_wid(surface))
            && offer.with_mime_types(|types| types.iter().any(|mime| mime == MIME));
        offer.accept_mime_type(offer.serial, supported.then(|| MIME.to_owned()));
        offer.set_actions(
            if supported {
                DndAction::Copy
            } else {
                DndAction::empty()
            },
            if supported {
                DndAction::Copy
            } else {
                DndAction::empty()
            },
        );
    }

    // SCTK tracks enter/motion coordinates per offer. Leave destroys undropped
    // offers; a completed drop's asynchronous read survives the following leave.
    fn motion(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice, _: f64, _: f64) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}
    fn selection(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}

    fn drop_performed(&mut self, _: &Connection, _: &QueueHandle<Self>, device: &WlDataDevice) {
        let Some(data) = device.data::<DataDeviceData>() else {
            return;
        };
        let seat = data.seat().id();
        let Some(offer) = data.drag_offer() else {
            return;
        };
        let window = make_wid(&offer.surface);
        let position = self.windows.borrow().get(&window).map(|window| {
            LogicalPosition::new(offer.x, offer.y)
                .to_physical(window.lock().unwrap().scale_factor())
        });
        let allowed_action =
            offer.inner().version() < 3 || offer.selected_action == DndAction::Copy;
        if position.is_none()
            || !allowed_action
            || !offer.with_mime_types(|types| types.iter().any(|mime| mime == MIME))
        {
            offer.destroy();
            return;
        }
        self.cancel_file_drop(&seat);
        let Ok(pipe) = offer.receive(MIME.to_owned()) else {
            offer.destroy();
            return;
        };
        let file = File::from(OwnedFd::from(pipe));
        // SAFETY: fcntl only sets status flags on this owned live descriptor.
        let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
        if flags < 0
            || unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
        {
            offer.destroy();
            return;
        }
        let Some(seat_state) = self.seats.get_mut(&seat) else {
            offer.destroy();
            return;
        };
        seat_state.pending_drop = Some(PendingDrop {
            offer,
            window,
            position: position.unwrap(),
            bytes: Vec::new(),
            source: None,
            timeout: None,
        });
        let read_seat = seat.clone();
        let source = self.loop_handle.insert_source(
            Generic::new(file, Interest::READ, Mode::Level),
            move |_, file, state| {
                let Some(pending) = state
                    .seats
                    .get_mut(&read_seat)
                    .and_then(|seat| seat.pending_drop.as_mut())
                else {
                    return Ok(PostAction::Remove);
                };
                let mut file: &File = file;
                let mut buffer = [0u8; 8192];
                // Bound work per dispatch as well as total bytes. Never block winit.
                for _ in 0..8 {
                    match file.read(&mut buffer) {
                        Ok(0) => {
                            state.finish_file_drop(&read_seat, true);
                            return Ok(PostAction::Remove);
                        }
                        Ok(n) if pending.bytes.len() + n <= crate::file_drop::MAX_BYTES => {
                            pending.bytes.extend_from_slice(&buffer[..n])
                        }
                        Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                        Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                        _ => {
                            state.finish_file_drop(&read_seat, false);
                            return Ok(PostAction::Remove);
                        }
                    }
                }
                Ok(PostAction::Continue)
            },
        );
        let Ok(source) = source else {
            self.cancel_file_drop(&seat);
            return;
        };
        self.seats
            .get_mut(&seat)
            .unwrap()
            .pending_drop
            .as_mut()
            .unwrap()
            .source = Some(source);
        let timeout_seat = seat.clone();
        let timeout = self.loop_handle.insert_source(
            Timer::from_duration(Duration::from_secs(10)),
            move |_, _, state| {
                if let Some(pending) = state
                    .seats
                    .get_mut(&timeout_seat)
                    .and_then(|seat| seat.pending_drop.as_mut())
                {
                    pending.timeout = None;
                }
                state.cancel_file_drop(&timeout_seat);
                TimeoutAction::Drop
            },
        );
        match timeout {
            Ok(token) => {
                self.seats
                    .get_mut(&seat)
                    .unwrap()
                    .pending_drop
                    .as_mut()
                    .unwrap()
                    .timeout = Some(token)
            }
            Err(_) => self.cancel_file_drop(&seat),
        }
    }
}

impl DataOfferHandler for WinitState {
    fn source_actions(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        _: DndAction,
    ) {
    }
    fn selected_action(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        _: DndAction,
    ) {
    }
}

delegate_dispatch!(WinitState: [WlDataDeviceManager: GlobalData] => DataDeviceManagerState);
delegate_dispatch!(WinitState: [WlDataDevice: DataDeviceData] => DataDeviceManagerState);
delegate_dispatch!(WinitState: [WlDataOffer: DataOfferData] => DataDeviceManagerState);
