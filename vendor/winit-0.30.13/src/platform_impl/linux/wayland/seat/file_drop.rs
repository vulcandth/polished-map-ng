//! Receive-only file drops on winit's own Wayland connection and event queue.
//! Polished Map NG extension, Apache-2.0. Implements wl_data_device/offer's
//! lifecycle directly: dropped offers outlive leave/new-enter until EOF/cancel.
//! SCTK supplies manager binding and the pipe helper, not drag-offer ownership.
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::os::fd::{AsRawFd, OwnedFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::dpi::{LogicalPosition, PhysicalPosition};
use crate::event::WindowEvent;
use crate::platform_impl::wayland::{make_wid, state::WinitState, WindowId};
use calloop::{
    generic::Generic,
    timer::{TimeoutAction, Timer},
    Interest, Mode, PostAction, RegistrationToken,
};
use sctk::data_device_manager::DataDeviceManagerState;
use sctk::globals::GlobalData;
use sctk::reexports::client::protocol::{
    wl_data_device::{self, WlDataDevice},
    wl_data_device_manager::{DndAction, WlDataDeviceManager},
    wl_data_offer::{self, WlDataOffer},
    wl_seat::WlSeat,
};
use sctk::reexports::client::{
    backend::ObjectId, delegate_dispatch, event_created_child, Connection, Dispatch, Proxy,
    QueueHandle,
};

const MIME: &str = "text/uri-list";
const MAX_PENDING: usize = 16;

#[derive(Debug)]
struct Offer(WlDataOffer);
impl Drop for Offer {
    fn drop(&mut self) {
        self.0.destroy();
    }
}

#[derive(Debug, Default)]
pub(super) struct OfferData {
    uri_list: AtomicBool,
    copy: AtomicBool,
}

#[derive(Debug)]
pub(super) struct DeviceData {
    seat: ObjectId,
}

#[derive(Debug)]
struct Hover {
    offer: Offer,
    window: WindowId,
    position: LogicalPosition<f64>,
}

#[derive(Debug)]
struct PendingDrop {
    offer: Offer,
    window: WindowId,
    position: PhysicalPosition<f64>,
    bytes: Vec<u8>,
    source: Option<RegistrationToken>,
    timeout: Option<RegistrationToken>,
}

#[derive(Debug)]
pub(crate) struct DropDevice {
    device: WlDataDevice,
    unassigned: Vec<Offer>,
    hover: Option<Hover>,
    pending: Vec<PendingDrop>,
}

impl DropDevice {
    pub(crate) fn new(
        manager: &DataDeviceManagerState,
        queue: &QueueHandle<WinitState>,
        seat: &WlSeat,
    ) -> Self {
        Self {
            device: manager.data_device_manager().get_data_device(
                seat,
                queue,
                DeviceData { seat: seat.id() },
            ),
            unassigned: Vec::new(),
            hover: None,
            pending: Vec::new(),
        }
    }

    fn take_offer(&mut self, id: &ObjectId) -> Option<Offer> {
        let index = self.unassigned.iter().position(|offer| offer.0.id() == *id)?;
        Some(self.unassigned.remove(index))
    }
}

impl Drop for DropDevice {
    fn drop(&mut self) {
        if self.device.version() >= 2 {
            self.device.release();
        }
    }
}

impl WinitState {
    fn drop_device(&mut self, seat: &ObjectId) -> Option<&mut DropDevice> {
        self.seats.get_mut(seat)?.data_device.as_mut()
    }

    fn pending_drop(&mut self, seat: &ObjectId, offer: &ObjectId) -> Option<&mut PendingDrop> {
        self.drop_device(seat)?.pending.iter_mut().find(|pending| pending.offer.0.id() == *offer)
    }

    fn take_pending_drop(&mut self, seat: &ObjectId, offer: &ObjectId) -> Option<PendingDrop> {
        let device = self.drop_device(seat)?;
        let index = device.pending.iter().position(|pending| pending.offer.0.id() == *offer)?;
        Some(device.pending.remove(index))
    }

    fn cancel_file_drop(&mut self, seat: &ObjectId, offer: &ObjectId) {
        if let Some(pending) = self.take_pending_drop(seat, offer) {
            if let Some(token) = pending.source {
                self.loop_handle.remove(token);
            }
            if let Some(token) = pending.timeout {
                self.loop_handle.remove(token);
            }
        }
    }

    pub(super) fn cancel_file_drops(&mut self, seat: &ObjectId) {
        let offers: Vec<_> = self
            .drop_device(seat)
            .map(|device| device.pending.iter().map(|pending| pending.offer.0.id()).collect())
            .unwrap_or_default();
        for offer in offers {
            self.cancel_file_drop(seat, &offer);
        }
    }

    pub(crate) fn cancel_window_file_drops(&mut self, window: WindowId) {
        let mut pending = Vec::new();
        for (seat, state) in &mut self.seats {
            if let Some(device) = state.data_device.as_mut() {
                if device.hover.as_ref().is_some_and(|hover| hover.window == window) {
                    device.hover = None;
                }
                pending.extend(
                    device
                        .pending
                        .iter()
                        .filter(|drop| drop.window == window)
                        .map(|drop| (seat.clone(), drop.offer.0.id())),
                );
            }
        }
        for (seat, offer) in pending {
            self.cancel_file_drop(&seat, &offer);
        }
    }

    fn finish_file_drop(&mut self, seat: &ObjectId, offer: &ObjectId, success: bool) {
        if let Some(pending) = self.take_pending_drop(seat, offer) {
            // Called by the read source, which removes itself after this returns.
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
                        if pending.offer.0.version() >= 3 {
                            pending.offer.0.finish();
                        }
                    }
                }
            }
        }
    }

    fn receive_file_drop(&mut self, seat: ObjectId, hover: Hover) {
        let metadata = hover.offer.0.data::<OfferData>().unwrap();
        let allowed = hover.offer.0.version() < 3 || metadata.copy.load(Ordering::Relaxed);
        let position = self
            .windows
            .borrow()
            .get(&hover.window)
            .map(|window| hover.position.to_physical(window.lock().unwrap().scale_factor()));
        if position.is_none()
            || !allowed
            || !metadata.uri_list.load(Ordering::Relaxed)
            || self.drop_device(&seat).is_none_or(|device| device.pending.len() >= MAX_PENDING)
        {
            return;
        }
        let Ok(pipe) =
            sctk::data_device_manager::data_offer::receive(&hover.offer.0, MIME.to_owned())
        else {
            return;
        };
        let file = File::from(OwnedFd::from(pipe));
        // SAFETY: fcntl only sets status flags on this owned live descriptor.
        let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
        if flags < 0
            || unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
        {
            return;
        }
        let offer = hover.offer.0.id();
        self.drop_device(&seat).unwrap().pending.push(PendingDrop {
            offer: hover.offer,
            window: hover.window,
            position: position.unwrap(),
            bytes: Vec::new(),
            source: None,
            timeout: None,
        });
        let read_seat = seat.clone();
        let read_offer = offer.clone();
        let source = self.loop_handle.insert_source(
            Generic::new(file, Interest::READ, Mode::Level),
            move |_, file, state| {
                let Some(pending) = state.pending_drop(&read_seat, &read_offer) else {
                    return Ok(PostAction::Remove);
                };
                let mut file: &File = file;
                let mut buffer = [0u8; 8192];
                // Bound work per dispatch as well as total bytes. Never block winit.
                for _ in 0..8 {
                    match file.read(&mut buffer) {
                        Ok(0) => {
                            state.finish_file_drop(&read_seat, &read_offer, true);
                            return Ok(PostAction::Remove);
                        }
                        Ok(n) if pending.bytes.len() + n <= crate::file_drop::MAX_BYTES => {
                            pending.bytes.extend_from_slice(&buffer[..n])
                        }
                        Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                        Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                        _ => {
                            state.finish_file_drop(&read_seat, &read_offer, false);
                            return Ok(PostAction::Remove);
                        }
                    }
                }
                Ok(PostAction::Continue)
            },
        );
        let Ok(source) = source else {
            self.cancel_file_drop(&seat, &offer);
            return;
        };
        self.pending_drop(&seat, &offer).unwrap().source = Some(source);
        let timeout_seat = seat.clone();
        let timeout_offer = offer.clone();
        let timeout = self.loop_handle.insert_source(
            Timer::from_duration(Duration::from_secs(10)),
            move |_, _, state| {
                if let Some(pending) = state.pending_drop(&timeout_seat, &timeout_offer) {
                    pending.timeout = None;
                }
                state.cancel_file_drop(&timeout_seat, &timeout_offer);
                TimeoutAction::Drop
            },
        );
        match timeout {
            Ok(token) => self.pending_drop(&seat, &offer).unwrap().timeout = Some(token),
            Err(_) => self.cancel_file_drop(&seat, &offer),
        }
    }
}

impl Dispatch<WlDataDevice, DeviceData> for WinitState {
    event_created_child!(WinitState, WlDataDevice, [0 => (WlDataOffer, OfferData::default())]);

    fn event(
        state: &mut Self,
        _: &WlDataDevice,
        event: wl_data_device::Event,
        data: &DeviceData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_device::Event::DataOffer { id } => {
                let offer = Offer(id);
                if let Some(device) = state.drop_device(&data.seat) {
                    device.unassigned.push(offer);
                }
            }
            wl_data_device::Event::Enter { serial, surface, x, y, id } => {
                let window = make_wid(&surface);
                let content = state.windows.borrow().contains_key(&window);
                let Some(device) = state.drop_device(&data.seat) else {
                    return;
                };
                // Only an undropped hover ends here. Completed transfers have
                // their own offer/source/timer and survive subsequent drags.
                device.hover = None;
                let Some(offer) = id.and_then(|id| device.take_offer(&id.id())) else {
                    return;
                };
                let supported = content
                    && offer.0.data::<OfferData>().unwrap().uri_list.load(Ordering::Relaxed);
                offer.0.accept(serial, supported.then(|| MIME.to_owned()));
                if offer.0.version() >= 3 {
                    let actions = if supported { DndAction::Copy } else { DndAction::empty() };
                    offer.0.set_actions(actions, actions);
                }
                device.hover = Some(Hover { offer, window, position: LogicalPosition::new(x, y) });
            }
            wl_data_device::Event::Motion { x, y, .. } => {
                if let Some(hover) =
                    state.drop_device(&data.seat).and_then(|device| device.hover.as_mut())
                {
                    hover.position = LogicalPosition::new(x, y);
                }
            }
            wl_data_device::Event::Leave => {
                if let Some(device) = state.drop_device(&data.seat) {
                    device.hover = None;
                }
            }
            wl_data_device::Event::Drop => {
                if let Some(hover) =
                    state.drop_device(&data.seat).and_then(|device| device.hover.take())
                {
                    state.receive_file_drop(data.seat.clone(), hover);
                }
            }
            wl_data_device::Event::Selection { id: Some(id) } => {
                // This device never reads or replaces the clipboard selection.
                if let Some(device) = state.drop_device(&data.seat) {
                    device.take_offer(&id.id());
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<WlDataOffer, OfferData> for WinitState {
    fn event(
        _: &mut Self,
        _: &WlDataOffer,
        event: wl_data_offer::Event,
        data: &OfferData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_offer::Event::Offer { mime_type } if mime_type == MIME => {
                data.uri_list.store(true, Ordering::Relaxed);
            }
            wl_data_offer::Event::Action { dnd_action } => {
                data.copy.store(
                    dnd_action.into_result().ok() == Some(DndAction::Copy),
                    Ordering::Relaxed,
                );
            }
            _ => {}
        }
    }
}

delegate_dispatch!(WinitState: [WlDataDeviceManager: GlobalData] => DataDeviceManagerState);
