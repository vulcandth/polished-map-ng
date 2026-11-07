//! Smithay Clipboard
//!
//! Provides access to the Wayland clipboard for gui applications. The user
//! should have surface around.

#![deny(clippy::all, clippy::if_not_else, clippy::enum_glob_use)]
use std::ffi::c_void;
use std::io::Result;
use std::sync::mpsc::{self, Receiver};

use sctk::reexports::calloop::channel::{self, Sender};
use sctk::reexports::client::backend::Backend;
use sctk::reexports::client::protocol::wl_data_offer::WlDataOffer;
use sctk::reexports::client::Connection;

mod mime;
mod state;
mod worker;

pub use state::binary::{Selection, Transfer, MAX_BYTES};
use std::sync::{atomic::AtomicUsize, Arc};

/// Access to a Wayland clipboard.
pub struct Clipboard {
    request_sender: Sender<worker::Command>,
    outstanding: Arc<AtomicUsize>,
    external_offer: Option<Option<WlDataOffer>>,
    request_receiver: Receiver<Result<String>>,
    clipboard_thread: Option<std::thread::JoinHandle<()>>,
}

impl Clipboard {
    /// Creates new clipboard which will be running on its own thread with its
    /// own event queue to handle clipboard requests.
    ///
    /// # Safety
    ///
    /// `display` must be a valid `*mut wl_display` pointer, and it must remain
    /// valid for as long as `Clipboard` object is alive.
    pub unsafe fn new(display: *mut c_void) -> Self {
        let backend = unsafe { Backend::from_foreign_display(display.cast()) };
        let connection = Connection::from_backend(backend);

        // Create channel to send data to clipboard thread.
        let (request_sender, rx_chan) = channel::channel();
        // Create channel to get data from the clipboard thread.
        let (clipboard_reply_sender, request_receiver) = mpsc::channel();

        let name = String::from("smithay-clipboard");
        let clipboard_thread = worker::spawn(name, connection, rx_chan, clipboard_reply_sender);

        Self {
            request_receiver,
            request_sender,
            clipboard_thread,
            outstanding: Arc::default(),
            external_offer: None,
        }
    }

    /// Use the selection retained by the host window adapter, on the same display.
    /// This avoids competing receive offers when a compositor supports only the
    /// last offer sent to a client. `None` explicitly represents an empty selection.
    pub fn set_external_selection(&mut self, selection: Option<(WlDataOffer, Vec<String>)>) {
        let offer = selection.as_ref().map(|(offer, _)| offer.clone());
        if self.external_offer.as_ref() == Some(&offer) {
            return;
        }
        self.external_offer = Some(offer);
        let _ = self.request_sender.send(worker::Command::ExternalSelection(selection));
    }

    /// Request binary data without blocking the GUI. MIME types are tried in order;
    /// a matching offer is selected once, with no fallback after transfer failure.
    pub fn load_bytes(&self, mimes: Vec<String>) -> Result<Transfer<Selection>> {
        state::binary::validate_mimes(&mimes)?;
        let (transfer, reply) = state::binary::pair(&self.outstanding)?;
        self.request_sender
            .send(worker::Command::LoadBytes(mimes, reply))
            .map_err(|_| std::io::Error::other("clipboard worker stopped"))?;
        Ok(transfer)
    }

    /// Advertise exact MIME representations on the focused seat. Completion means
    /// the request was issued, not that another application has read the data.
    pub fn store_bytes(&self, payload: Vec<(String, Vec<u8>)>) -> Result<Transfer<()>> {
        state::binary::validate_payload(&payload)?;
        let (transfer, reply) = state::binary::pair(&self.outstanding)?;
        self.request_sender
            .send(worker::Command::StoreBytes(payload, reply))
            .map_err(|_| std::io::Error::other("clipboard worker stopped"))?;
        Ok(transfer)
    }

    /// Load clipboard data.
    ///
    /// Loads content from a clipboard on a last observed seat.
    pub fn load(&self) -> Result<String> {
        let _ = self.request_sender.send(worker::Command::Load);

        if let Ok(reply) = self.request_receiver.recv() {
            reply
        } else {
            // The clipboard thread is dead, however we shouldn't crash downstream, so
            // propogating an error.
            Err(std::io::Error::other("clipboard is dead."))
        }
    }

    /// Store to a clipboard.
    ///
    /// Stores to a clipboard on a last observed seat.
    pub fn store<T: Into<String>>(&self, text: T) {
        let request = worker::Command::Store(text.into());
        let _ = self.request_sender.send(request);
    }

    /// Load primary clipboard data.
    ///
    /// Loads content from a  primary clipboard on a last observed seat.
    pub fn load_primary(&self) -> Result<String> {
        let _ = self.request_sender.send(worker::Command::LoadPrimary);

        if let Ok(reply) = self.request_receiver.recv() {
            reply
        } else {
            // The clipboard thread is dead, however we shouldn't crash downstream, so
            // propogating an error.
            Err(std::io::Error::other("clipboard is dead."))
        }
    }

    /// Store to a primary clipboard.
    ///
    /// Stores to a primary clipboard on a last observed seat.
    pub fn store_primary<T: Into<String>>(&self, text: T) {
        let request = worker::Command::StorePrimary(text.into());
        let _ = self.request_sender.send(request);
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        // Shutdown smithay-clipboard.
        let _ = self.request_sender.send(worker::Command::Exit);
        if let Some(clipboard_thread) = self.clipboard_thread.take() {
            let _ = clipboard_thread.join();
        }
    }
}
