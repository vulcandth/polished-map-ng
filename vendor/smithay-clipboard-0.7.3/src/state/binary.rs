//! Bounded binary selection transfers on the existing GUI connection.
//! Polished Map NG adaptation, 2026-11-06; MIT, like smithay-clipboard.
use super::*;
use sctk::data_device_manager::ReadPipe;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc,
};
use std::time::{Duration, Instant};

/// Maximum encoded selection size (64 MiB), including all advertised formats.
pub const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_TRANSFERS: usize = 16;
const TIMEOUT: Duration = Duration::from_secs(10);
const QUANTUM: usize = 256 * 1024;

/// One selected MIME representation. No text conversion is performed.
#[derive(Debug, PartialEq, Eq)]
pub struct Selection {
    /// MIME type selected in the caller's preference order.
    pub mime: String,
    /// The exact bytes received, bounded by [`MAX_BYTES`].
    pub bytes: Vec<u8>,
}

/// Nonblocking reply. Dropping it cancels any outstanding receive.
pub struct Transfer<T> {
    rx: mpsc::Receiver<Result<T>>,
    ticket: Arc<Ticket>,
}

impl<T> Transfer<T> {
    /// Returns `None` while the request is pending. Consume a completed result once.
    pub fn poll(&self) -> Option<Result<T>> {
        match self.rx.try_recv() {
            Ok(value) => Some(value),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err(Error::other("clipboard worker stopped")))
            },
        }
    }
}

impl<T> Drop for Transfer<T> {
    fn drop(&mut self) {
        self.ticket.cancelled.store(true, Ordering::Release);
    }
}

struct Ticket {
    cancelled: AtomicBool,
    outstanding: Arc<AtomicUsize>,
    deadline: Instant,
}
impl Drop for Ticket {
    fn drop(&mut self) {
        self.outstanding.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(crate) struct Reply<T> {
    tx: mpsc::Sender<Result<T>>,
    ticket: Arc<Ticket>,
}
impl<T> Reply<T> {
    fn invalid(&self) -> Option<Error> {
        if self.ticket.cancelled.load(Ordering::Acquire) {
            Some(Error::new(ErrorKind::Interrupted, "clipboard request cancelled"))
        } else if Instant::now() >= self.ticket.deadline {
            Some(Error::new(ErrorKind::TimedOut, "clipboard transfer timed out"))
        } else {
            None
        }
    }
    pub(crate) fn finish(self, result: Result<T>) {
        let _ = self.tx.send(result);
    }
}

pub(crate) fn pair<T>(outstanding: &Arc<AtomicUsize>) -> Result<(Transfer<T>, Reply<T>)> {
    outstanding
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| (n < MAX_TRANSFERS).then_some(n + 1))
        .map_err(|_| Error::new(ErrorKind::WouldBlock, "too many clipboard requests"))?;
    let ticket = Arc::new(Ticket {
        cancelled: AtomicBool::new(false),
        outstanding: outstanding.clone(),
        deadline: Instant::now() + TIMEOUT,
    });
    let (tx, rx) = mpsc::channel();
    Ok((Transfer { rx, ticket: ticket.clone() }, Reply { tx, ticket }))
}

pub(crate) fn validate_mimes(mimes: &[String]) -> Result<()> {
    if mimes.is_empty()
        || mimes.len() > 16
        || mimes.iter().any(|m| {
            m.is_empty() || m.len() > 255 || m.bytes().any(|c| c == 0 || c.is_ascii_control())
        })
    {
        return Err(Error::new(ErrorKind::InvalidInput, "invalid clipboard MIME list"));
    }
    Ok(())
}

pub(crate) fn validate_payload(payload: &[(String, Vec<u8>)]) -> Result<()> {
    validate_mimes(&payload.iter().map(|(m, _)| m.clone()).collect::<Vec<_>>())?;
    let mut total = 0usize;
    for (i, (mime, bytes)) in payload.iter().enumerate() {
        if payload[..i].iter().any(|(m, _)| m == mime) {
            return Err(Error::new(ErrorKind::InvalidInput, "duplicate clipboard MIME type"));
        }
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| Error::other("clipboard size overflow"))?;
        if total > MAX_BYTES {
            return Err(Error::other("clipboard exceeds 64 MiB"));
        }
    }
    Ok(())
}

struct Source {
    source: CopyPasteSource,
    seat: ObjectId,
    payload: Vec<(String, Rc<[u8]>)>,
}
struct Receive {
    pipe: ReadPipe,
    seat: ObjectId,
    mime: String,
    bytes: Vec<u8>,
    reply: Reply<Selection>,
}
struct Send {
    pipe: WritePipe,
    seat: ObjectId,
    bytes: Rc<[u8]>,
    offset: usize,
    deadline: Instant,
}

#[derive(Default)]
pub(crate) struct BinaryState {
    sources: Vec<Source>,
    receives: Vec<Receive>,
    sends: Vec<Send>,
}

impl BinaryState {
    pub(crate) fn active(&self) -> bool {
        !self.receives.is_empty() || !self.sends.is_empty()
    }

    pub(crate) fn cancel_seat(&mut self, seat: &ObjectId) {
        self.sources.retain(|s| &s.seat != seat);
        self.sends.retain(|s| &s.seat != seat);
        let mut i = 0;
        while i < self.receives.len() {
            if &self.receives[i].seat == seat {
                self.receives.swap_remove(i).reply.finish(Err(Error::other("clipboard seat lost")));
            } else {
                i += 1;
            }
        }
    }

    pub(crate) fn cancelled(&mut self, source: &WlDataSource) {
        self.sources.retain(|s| s.source.inner() != source);
    }

    /// Returns the pipe only if this is not one of our binary sources.
    pub(crate) fn send(
        &mut self,
        source: &WlDataSource,
        mime: &str,
        pipe: WritePipe,
    ) -> Option<WritePipe> {
        let Some(source) = self.sources.iter().find(|s| s.source.inner() == source) else {
            return Some(pipe);
        };
        if self.sends.len() >= MAX_TRANSFERS {
            return None;
        }
        let (_, bytes) = source.payload.iter().find(|(m, _)| m == mime)?;
        if set_non_blocking(pipe.as_raw_fd()).is_err() {
            return None;
        }
        self.sends.push(Send {
            pipe,
            seat: source.seat.clone(),
            bytes: bytes.clone(),
            offset: 0,
            deadline: Instant::now() + TIMEOUT,
        });
        None
    }

    pub(crate) fn tick(&mut self) {
        let mut i = 0;
        while i < self.receives.len() {
            let receive = &mut self.receives[i];
            let result = receive
                .reply
                .invalid()
                .map(Err)
                .or_else(|| read_chunk(&mut receive.pipe, &mut receive.bytes));
            if let Some(result) = result {
                let receive = self.receives.swap_remove(i);
                receive.reply.finish(
                    result.map(|()| Selection { mime: receive.mime, bytes: receive.bytes }),
                );
            } else {
                i += 1;
            }
        }
        self.sends.retain_mut(|send| {
            Instant::now() < send.deadline
                && write_chunk(&mut send.pipe, &send.bytes, &mut send.offset).is_none()
        });
    }
}

// Limit work per dispatch even for an always-ready peer; never block the event queue.
fn read_chunk(pipe: &mut impl Read, bytes: &mut Vec<u8>) -> Option<Result<()>> {
    let mut buffer = [0u8; 8192];
    let mut count = 0;
    while count < QUANTUM {
        match pipe.read(&mut buffer) {
            Ok(0) => return Some(Ok(())),
            Ok(n) => {
                if n > MAX_BYTES - bytes.len() {
                    return Some(Err(Error::other("clipboard exceeds 64 MiB")));
                }
                bytes.extend_from_slice(&buffer[..n]);
                count += n;
            },
            Err(e) if e.kind() == ErrorKind::WouldBlock => return None,
            Err(e) if e.kind() == ErrorKind::Interrupted => return None,
            Err(e) => return Some(Err(e)),
        }
    }
    None
}

fn write_chunk(pipe: &mut impl Write, bytes: &[u8], offset: &mut usize) -> Option<Result<()>> {
    let end = bytes.len().min(offset.saturating_add(QUANTUM));
    while *offset < end {
        match pipe.write(&bytes[*offset..end]) {
            Ok(0) => {
                return Some(Err(Error::new(ErrorKind::WriteZero, "clipboard receiver stopped")));
            },
            Ok(n) => *offset += n,
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::Interrupted => {
                return None;
            },
            Err(e) => return Some(Err(e)),
        }
    }
    (*offset == bytes.len()).then_some(Ok(()))
}

impl State {
    pub(crate) fn store_binary(&mut self, payload: Vec<(String, Vec<u8>)>, reply: Reply<()>) {
        if let Some(err) = reply.invalid() {
            reply.finish(Err(err));
            return;
        }
        let result = (|| {
            validate_payload(&payload)?;
            if self.binary.sources.len() >= MAX_TRANSFERS {
                return Err(Error::other("too many clipboard sources"));
            }
            let id = self.latest_seat.as_ref().ok_or_else(|| Error::other("no clipboard seat"))?;
            let seat = self.seats.get(id).ok_or_else(|| Error::other("clipboard seat lost"))?;
            if !seat.has_focus {
                return Err(Error::other("client does not have clipboard focus"));
            }
            let device = seat
                .data_device
                .as_ref()
                .ok_or_else(|| Error::other("clipboard data device unavailable"))?;
            let manager = self
                .data_device_manager_state
                .as_ref()
                .ok_or_else(|| Error::other("clipboard data-device protocol unavailable"))?;
            let source = manager
                .create_copy_paste_source(&self.queue_handle, payload.iter().map(|(m, _)| m));
            source.set_selection(device, seat.latest_serial);
            self.binary.sources.push(Source {
                source,
                seat: id.clone(),
                payload: payload.into_iter().map(|(m, b)| (m, Rc::from(b))).collect(),
            });
            Ok(())
        })();
        reply.finish(result);
    }

    pub(crate) fn load_binary(&mut self, mimes: Vec<String>, reply: Reply<Selection>) {
        if let Some(err) = reply.invalid() {
            reply.finish(Err(err));
            return;
        }
        let result = (|| {
            validate_mimes(&mimes)?;
            if self.binary.receives.len() >= MAX_TRANSFERS {
                return Err(Error::other("too many clipboard receives"));
            }
            let id = self.latest_seat.as_ref().ok_or_else(|| Error::other("no clipboard seat"))?;
            let seat = self.seats.get(id).ok_or_else(|| Error::other("clipboard seat lost"))?;
            if !seat.has_focus {
                return Err(Error::other("client does not have clipboard focus"));
            }
            let (mime, pipe) = if let Some(selection) = &self.external_selection {
                let (offer, offered) = selection.as_ref().ok_or_else(|| {
                    Error::new(ErrorKind::NotFound, "clipboard selection is empty")
                })?;
                if !offer.is_alive() {
                    return Err(Error::other("clipboard selection changed"));
                }
                let mime =
                    mimes.iter().find(|m| offered.contains(m)).cloned().ok_or_else(|| {
                        Error::new(ErrorKind::NotFound, "clipboard has no requested MIME type")
                    })?;
                let pipe = sctk::data_device_manager::data_offer::receive(offer, mime.clone())?;
                (mime, pipe)
            } else {
                let offer =
                    seat.data_device.as_ref().and_then(|d| d.data().selection_offer()).ok_or_else(
                        || Error::new(ErrorKind::NotFound, "clipboard selection is empty"),
                    )?;
                let mime = offer
                    .with_mime_types(|offered| mimes.iter().find(|m| offered.contains(m)).cloned())
                    .ok_or_else(|| {
                        Error::new(ErrorKind::NotFound, "clipboard has no requested MIME type")
                    })?;
                let pipe = offer.receive(mime.clone()).map_err(|e| Error::other(e.to_string()))?;
                (mime, pipe)
            };
            set_non_blocking(pipe.as_raw_fd())?;
            Ok((id.clone(), mime, pipe))
        })();
        match result {
            Ok((seat, mime, pipe)) => {
                self.binary.receives.push(Receive { pipe, seat, mime, bytes: Vec::new(), reply })
            },
            Err(err) => reply.finish(Err(err)),
        }
    }
}

#[cfg(test)]
mod tests;
