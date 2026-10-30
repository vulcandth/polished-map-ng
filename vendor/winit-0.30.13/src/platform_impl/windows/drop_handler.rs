use std::ffi::{c_void, OsString};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::core::{IUnknown, GUID, HRESULT};
use windows_sys::Win32::Foundation::{
    DV_E_FORMATETC, E_NOINTERFACE, E_POINTER, HWND, POINT, POINTL, S_OK,
};
use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
use windows_sys::Win32::System::Com::{IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL};
use windows_sys::Win32::System::Ole::{
    ReleaseStgMedium, CF_HDROP, DROPEFFECT_COPY, DROPEFFECT_NONE,
};
use windows_sys::Win32::UI::Shell::{DragQueryFileW, HDROP};

use tracing::debug;

use crate::platform_impl::platform::definitions::{
    IDataObjectVtbl, IDropTarget, IDropTargetVtbl, IUnknownVtbl,
};
use crate::platform_impl::platform::WindowId;

use crate::event::Event;
use crate::window::WindowId as RootWindowId;

#[repr(C)]
pub struct FileDropHandlerData {
    pub interface: IDropTarget,
    refcount: AtomicUsize,
    window: HWND,
    send_event: Box<dyn Fn(Event<()>)>,
    cursor_effect: u32,
    hovered_is_valid: bool, /* If the currently hovered item is not valid there must not be any
                             * `HoveredFileCancelled` emitted */
}

pub struct FileDropHandler {
    pub data: *mut FileDropHandlerData,
}

#[allow(non_snake_case)]
impl FileDropHandler {
    pub fn new(window: HWND, send_event: Box<dyn Fn(Event<()>)>) -> FileDropHandler {
        let data = Box::new(FileDropHandlerData {
            interface: IDropTarget {
                lpVtbl: &DROP_TARGET_VTBL as *const IDropTargetVtbl,
            },
            refcount: AtomicUsize::new(1),
            window,
            send_event,
            cursor_effect: DROPEFFECT_NONE,
            hovered_is_valid: false,
        });
        FileDropHandler {
            data: Box::into_raw(data),
        }
    }

    // Implement IUnknown
    pub unsafe extern "system" fn QueryInterface(
        this: *mut IUnknown,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT {
        if riid.is_null() || ppvObject.is_null() {
            return E_POINTER;
        }
        unsafe {
            *ppvObject = ptr::null_mut();
        }
        let iid = unsafe { &*riid };
        // IUnknown and IDropTarget share this object's single COM identity.
        if matches!(iid.data1, 0 | 0x122)
            && iid.data2 == 0
            && iid.data3 == 0
            && iid.data4 == [0xc0, 0, 0, 0, 0, 0, 0, 0x46]
        {
            unsafe {
                Self::AddRef(this);
                *ppvObject = this.cast();
            }
            S_OK
        } else {
            E_NOINTERFACE
        }
    }

    pub unsafe extern "system" fn AddRef(this: *mut IUnknown) -> u32 {
        let drop_handler_data = unsafe { Self::from_interface(this) };
        let count = drop_handler_data.refcount.fetch_add(1, Ordering::Release) + 1;
        count as u32
    }

    pub unsafe extern "system" fn Release(this: *mut IUnknown) -> u32 {
        let drop_handler = unsafe { Self::from_interface(this) };
        let count = drop_handler.refcount.fetch_sub(1, Ordering::Release) - 1;
        if count == 0 {
            // Destroy the underlying data
            drop(unsafe { Box::from_raw(drop_handler as *mut FileDropHandlerData) });
        }
        count as u32
    }

    pub unsafe extern "system" fn DragEnter(
        this: *mut IDropTarget,
        pDataObj: *const IDataObject,
        _grfKeyState: u32,
        _pt: POINTL,
        pdwEffect: *mut u32,
    ) -> HRESULT {
        use crate::event::WindowEvent::HoveredFile;
        let drop_handler = unsafe { Self::from_interface(this) };
        let valid = unsafe {
            Self::iterate_filenames(pDataObj, |filename| {
                drop_handler.send_event(Event::WindowEvent {
                    window_id: RootWindowId(WindowId(drop_handler.window)),
                    event: HoveredFile(filename),
                });
            })
        };
        drop_handler.hovered_is_valid = valid;
        drop_handler.cursor_effect = if drop_handler.hovered_is_valid {
            DROPEFFECT_COPY
        } else {
            DROPEFFECT_NONE
        };
        unsafe {
            *pdwEffect = drop_handler.cursor_effect;
        }

        S_OK
    }

    pub unsafe extern "system" fn DragOver(
        this: *mut IDropTarget,
        _grfKeyState: u32,
        _pt: POINTL,
        pdwEffect: *mut u32,
    ) -> HRESULT {
        let drop_handler = unsafe { Self::from_interface(this) };
        unsafe {
            *pdwEffect = drop_handler.cursor_effect;
        }

        S_OK
    }

    pub unsafe extern "system" fn DragLeave(this: *mut IDropTarget) -> HRESULT {
        use crate::event::WindowEvent::HoveredFileCancelled;
        let drop_handler = unsafe { Self::from_interface(this) };
        if drop_handler.hovered_is_valid {
            drop_handler.send_event(Event::WindowEvent {
                window_id: RootWindowId(WindowId(drop_handler.window)),
                event: HoveredFileCancelled,
            });
        }
        drop_handler.hovered_is_valid = false;

        S_OK
    }

    pub unsafe extern "system" fn Drop(
        this: *mut IDropTarget,
        pDataObj: *const IDataObject,
        _grfKeyState: u32,
        pt: POINTL,
        pdwEffect: *mut u32,
    ) -> HRESULT {
        use crate::event::WindowEvent::DroppedFile;
        let drop_handler = unsafe { Self::from_interface(this) };
        let mut paths = Vec::new();
        let valid = unsafe { Self::iterate_filenames(pDataObj, |path| paths.push(path)) };
        let mut delivered = false;
        if valid {
            // OLE supplies screen coordinates for THIS drop. Never use GetCursorPos.
            let mut point = POINT { x: pt.x, y: pt.y };
            let event = (unsafe { ScreenToClient(drop_handler.window, &mut point) } != 0)
                .then(|| {
                    crate::file_drop::completed(
                        paths.clone(),
                        crate::dpi::PhysicalPosition::new(point.x as f64, point.y as f64),
                    )
                })
                .flatten();
            if let Some(event) = event {
                drop_handler.send_event(Event::WindowEvent {
                    window_id: RootWindowId(WindowId(drop_handler.window)),
                    event,
                });
                delivered = true;
            }
            for path in paths {
                drop_handler.send_event(Event::WindowEvent {
                    window_id: RootWindowId(WindowId(drop_handler.window)),
                    event: DroppedFile(path),
                });
            }
        }
        drop_handler.hovered_is_valid = false;
        unsafe {
            *pdwEffect = if delivered {
                DROPEFFECT_COPY
            } else {
                DROPEFFECT_NONE
            };
        }

        S_OK
    }

    unsafe fn from_interface<'a, InterfaceT>(this: *mut InterfaceT) -> &'a mut FileDropHandlerData {
        unsafe { &mut *(this as *mut _) }
    }

    unsafe fn iterate_filenames<F>(data_obj: *const IDataObject, mut callback: F) -> bool
    where
        F: FnMut(PathBuf),
    {
        let drop_format = FORMATETC {
            cfFormat: CF_HDROP,
            ptd: ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL as u32,
        };

        let mut medium = unsafe { std::mem::zeroed() };
        let get_data_fn = unsafe { (*(*data_obj).cast::<IDataObjectVtbl>()).GetData };
        let get_data_result = unsafe { get_data_fn(data_obj as *mut _, &drop_format, &mut medium) };
        if get_data_result >= 0 {
            if medium.tymed != TYMED_HGLOBAL as u32 {
                unsafe { ReleaseStgMedium(&mut medium) };
                return false;
            }
            let hdrop = unsafe { medium.u.hGlobal as HDROP };

            // The second parameter (0xFFFFFFFF) instructs the function to return the item count
            let item_count = unsafe { DragQueryFileW(hdrop, 0xffffffff, ptr::null_mut(), 0) };
            let mut total = 0usize;
            let mut paths = Vec::new();
            let mut valid = item_count > 0 && item_count as usize <= crate::file_drop::MAX_FILES;

            for i in 0..if valid { item_count } else { 0 } {
                // Get the length of the path string NOT including the terminating null character.
                // Previously, this was using a fixed size array of MAX_PATH length, but the
                // Windows API allows longer paths under certain circumstances.
                let character_count =
                    unsafe { DragQueryFileW(hdrop, i, ptr::null_mut(), 0) as usize };
                let str_len = character_count + 1;
                total = total.saturating_add(str_len.saturating_mul(2));
                if total > crate::file_drop::MAX_BYTES {
                    valid = false;
                    break;
                }

                // Fill path_buf with the null-terminated file name
                let mut path_buf = vec![0; str_len];
                let copied =
                    unsafe { DragQueryFileW(hdrop, i, path_buf.as_mut_ptr(), str_len as u32) };
                if copied as usize != character_count {
                    valid = false;
                    break;
                }

                paths.push(OsString::from_wide(&path_buf[0..character_count]).into());
            }
            // GetData transfers an STGMEDIUM, not a WM_DROPFILES handle. Honor
            // pUnkForRelease and free both hover and drop media exactly once.
            unsafe { ReleaseStgMedium(&mut medium) };
            if valid {
                for path in paths {
                    callback(path);
                }
            }
            valid
        } else if get_data_result == DV_E_FORMATETC {
            // If the dropped item is not a file this error will occur.
            // In this case it is OK to return without taking further action.
            debug!("Error occurred while processing dropped/hovered item: item is not a file.");
            false
        } else {
            debug!("Unexpected error occurred while processing dropped/hovered item.");
            false
        }
    }
}

impl FileDropHandlerData {
    fn send_event(&self, event: Event<()>) {
        (self.send_event)(event);
    }
}

impl Drop for FileDropHandler {
    fn drop(&mut self) {
        unsafe {
            FileDropHandler::Release(self.data as *mut IUnknown);
        }
    }
}

static DROP_TARGET_VTBL: IDropTargetVtbl = IDropTargetVtbl {
    parent: IUnknownVtbl {
        QueryInterface: FileDropHandler::QueryInterface,
        AddRef: FileDropHandler::AddRef,
        Release: FileDropHandler::Release,
    },
    DragEnter: FileDropHandler::DragEnter,
    DragOver: FileDropHandler::DragOver,
    DragLeave: FileDropHandler::DragLeave,
    Drop: FileDropHandler::Drop,
};
