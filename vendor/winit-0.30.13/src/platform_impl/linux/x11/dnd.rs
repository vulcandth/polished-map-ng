use super::atoms::AtomName::None as DndNone;
use super::atoms::*;
use super::{util, CookieResultExt, X11Error, XConnection};
use std::os::raw::*;
use std::sync::Arc;
use x11rb::protocol::xproto::{self, ConnectionExt};

#[derive(Debug, Clone, Copy)]
pub enum DndState {
    Accepted,
    Rejected,
}

pub enum SelectionData {
    Bytes(Vec<u8>),
    Incremental,
}

pub struct Dnd {
    xconn: Arc<XConnection>,
    // Populated by XdndEnter event handler
    pub version: Option<c_long>,
    pub type_list: Option<Vec<xproto::Atom>>,
    // Populated by XdndPosition event handler
    pub source_window: Option<xproto::Window>,
    pub target_window: Option<xproto::Window>,
    pub position: Option<crate::dpi::PhysicalPosition<f64>>,
    pub pending_time: Option<xproto::Timestamp>,
    pub incremental: Option<Vec<u8>>,
}

impl Dnd {
    pub fn new(xconn: Arc<XConnection>) -> Result<Self, X11Error> {
        Ok(Dnd {
            xconn,
            version: None,
            type_list: None,
            source_window: None,
            target_window: None,
            position: None,
            pending_time: None,
            incremental: None,
        })
    }

    pub fn reset(&mut self) {
        self.version = None;
        self.type_list = None;
        self.source_window = None;
        self.target_window = None;
        self.position = None;
        self.pending_time = None;
        self.incremental = None;
    }

    pub unsafe fn send_status(
        &self,
        this_window: xproto::Window,
        target_window: xproto::Window,
        state: DndState,
    ) -> Result<(), X11Error> {
        let atoms = self.xconn.atoms();
        let (accepted, action) = match state {
            DndState::Accepted => (1, atoms[XdndActionCopy]),
            DndState::Rejected => (0, atoms[DndNone]),
        };
        self.xconn
            .send_client_msg(
                target_window,
                target_window,
                atoms[XdndStatus] as _,
                None,
                [this_window, accepted | 2, 0, 0, action as _],
            )?
            .ignore_error();

        Ok(())
    }

    pub unsafe fn send_finished(
        &self,
        this_window: xproto::Window,
        target_window: xproto::Window,
        state: DndState,
    ) -> Result<(), X11Error> {
        let atoms = self.xconn.atoms();
        let (accepted, action) = match state {
            DndState::Accepted => (1, atoms[XdndActionCopy]),
            DndState::Rejected => (0, atoms[DndNone]),
        };
        self.xconn
            .send_client_msg(
                target_window,
                target_window,
                atoms[XdndFinished] as _,
                None,
                [this_window, accepted, action as _, 0, 0],
            )?
            .ignore_error();

        Ok(())
    }

    pub unsafe fn get_type_list(
        &self,
        source_window: xproto::Window,
    ) -> Result<Vec<xproto::Atom>, util::GetPropertyError> {
        let atoms = self.xconn.atoms();
        self.xconn.get_property(
            source_window,
            atoms[XdndTypeList],
            xproto::Atom::from(xproto::AtomEnum::ATOM),
        )
    }

    pub unsafe fn convert_selection(&self, window: xproto::Window, time: xproto::Timestamp) {
        let atoms = self.xconn.atoms();
        self.xconn
            .xcb_connection()
            .convert_selection(
                window,
                atoms[XdndSelection],
                atoms[TextUriList],
                atoms[XdndSelection],
                time,
            )
            .expect_then_ignore_error("Failed to send XdndSelection event")
    }

    pub fn read_data(&self, window: xproto::Window) -> Option<SelectionData> {
        let atoms = self.xconn.atoms();
        let reply = self
            .xconn
            .xcb_connection()
            .get_property(
                true,
                window,
                atoms[XdndSelection],
                xproto::AtomEnum::ANY,
                0,
                (crate::file_drop::MAX_BYTES / 4 + 1) as u32,
            )
            .ok()?
            .reply()
            .ok()?;
        if reply.bytes_after != 0 || reply.value.len() > crate::file_drop::MAX_BYTES {
            let _ = self
                .xconn
                .xcb_connection()
                .delete_property(window, atoms[XdndSelection]);
            return None;
        }
        if reply.type_ == atoms[INCR] && reply.format == 32 && self.incremental.is_none() {
            return (reply.value32()?.next()? as usize <= crate::file_drop::MAX_BYTES)
                .then_some(SelectionData::Incremental);
        }
        (reply.format == 8 && reply.type_ == atoms[TextUriList])
            .then_some(SelectionData::Bytes(reply.value))
    }
}
