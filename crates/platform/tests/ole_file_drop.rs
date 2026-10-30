//! Hidden-window ABI test of the actual vendored callback, with a CF_HDROP provider.
//! Only the event sink/types are stand-ins; Win32 coordinates and COM media are real.
#![cfg(windows)]
#![allow(unsafe_code, dead_code)]

extern crate windows_sys_legacy as windows_sys;
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    path::PathBuf,
    ptr,
    rc::Rc,
};
use windows_sys::{
    Win32::{
        Foundation::{E_NOINTERFACE, HGLOBAL, POINT, POINTL, S_OK},
        Graphics::Gdi::ClientToScreen,
        System::{
            Com::{FORMATETC, IDataObject, STGMEDIUM, STGMEDIUM_0, TYMED_HGLOBAL},
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
        },
        UI::{
            Shell::DROPFILES,
            WindowsAndMessaging::{CreateWindowExW, DestroyWindow, WS_OVERLAPPEDWINDOW},
        },
    },
    core::{GUID, HRESULT, IUnknown, w},
};

#[path = "../../../vendor/winit-0.30.13/src/platform_impl/windows/definitions.rs"]
#[rustfmt::skip]
pub mod definitions;
#[path = "../../../vendor/winit-0.30.13/src/platform_impl/windows/drop_handler.rs"]
#[rustfmt::skip]
mod handler;
#[path = "../../../vendor/winit-0.30.13/src/file_drop.rs"]
mod file_drop;

pub mod platform_impl {
    pub mod platform {
        pub use crate::definitions;
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct WindowId(pub isize);
    }
}
pub mod window {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct WindowId(pub crate::platform_impl::platform::WindowId);
}
pub mod dpi {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PhysicalPosition<T> {
        pub x: T,
        pub y: T,
    }
    impl<T> PhysicalPosition<T> {
        pub fn new(x: T, y: T) -> Self {
            Self { x, y }
        }
    }
}
pub mod event {
    use super::*;
    #[derive(Debug)]
    pub enum Event<T> {
        WindowEvent {
            window_id: window::WindowId,
            event: WindowEvent,
        },
        UserEvent(T),
    }
    #[derive(Debug)]
    pub enum WindowEvent {
        HoveredFile(PathBuf),
        HoveredFileCancelled,
        DroppedFile(PathBuf),
        FilesDropped {
            transfer_id: u64,
            paths: Vec<PathBuf>,
            position: dpi::PhysicalPosition<f64>,
        },
    }
}

// The data object exposes the prefix through GetData that the receiver calls.
#[repr(C)]
struct DataVtable {
    parent: definitions::IUnknownVtbl,
    get_data:
        unsafe extern "system" fn(*mut IDataObject, *const FORMATETC, *mut STGMEDIUM) -> HRESULT,
}
#[repr(C)]
struct DataObject {
    vtable: *const DataVtable,
    names: Vec<u16>,
    released: Rc<Cell<usize>>,
}
#[repr(C)]
struct MediumOwner {
    vtable: *const definitions::IUnknownVtbl,
    memory: HGLOBAL,
    released: Rc<Cell<usize>>,
}

unsafe extern "system" fn no_interface(
    _: *mut IUnknown,
    _: *const GUID,
    out: *mut *mut c_void,
) -> HRESULT {
    unsafe {
        *out = ptr::null_mut();
    }
    E_NOINTERFACE
}
unsafe extern "system" fn add_ref(_: *mut IUnknown) -> u32 {
    1
}
unsafe extern "system" fn release(this: *mut IUnknown) -> u32 {
    // SAFETY: GetData transfers this boxed owner to ReleaseStgMedium exactly once.
    let owner = unsafe { Box::from_raw(this.cast::<MediumOwner>()) };
    unsafe {
        windows_sys::Win32::Foundation::GlobalFree(owner.memory);
    }
    owner.released.set(owner.released.get() + 1);
    0
}
static OWNER_VTABLE: definitions::IUnknownVtbl = definitions::IUnknownVtbl {
    QueryInterface: no_interface,
    AddRef: add_ref,
    Release: release,
};
unsafe extern "system" fn get_data(
    this: *mut IDataObject,
    _: *const FORMATETC,
    out: *mut STGMEDIUM,
) -> HRESULT {
    let object = unsafe { &*this.cast::<DataObject>() };
    let size = size_of::<DROPFILES>() + object.names.len() * 2;
    let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE, size) };
    assert!(!memory.is_null());
    let pointer = unsafe { GlobalLock(memory) };
    assert!(!pointer.is_null());
    unsafe {
        pointer.cast::<DROPFILES>().write(DROPFILES {
            pFiles: size_of::<DROPFILES>() as u32,
            pt: POINT { x: 0, y: 0 },
            fNC: 0,
            fWide: 1,
        });
        ptr::copy_nonoverlapping(
            object.names.as_ptr(),
            pointer
                .cast::<u8>()
                .add(size_of::<DROPFILES>())
                .cast::<u16>(),
            object.names.len(),
        );
        GlobalUnlock(memory);
    }
    let owner = Box::new(MediumOwner {
        vtable: &OWNER_VTABLE,
        memory,
        released: object.released.clone(),
    });
    unsafe {
        out.write(STGMEDIUM {
            tymed: TYMED_HGLOBAL as u32,
            u: STGMEDIUM_0 { hGlobal: memory },
            pUnkForRelease: Box::into_raw(owner).cast(),
        });
    }
    S_OK
}
static DATA_VTABLE: DataVtable = DataVtable {
    parent: definitions::IUnknownVtbl {
        QueryInterface: no_interface,
        AddRef: add_ref,
        Release: add_ref,
    },
    get_data,
};

#[test]
fn ole_drop_uses_by_value_screen_point_native_path_order_and_medium_release() {
    use std::os::windows::ffi::OsStringExt;
    // This window is never shown, focused or moved after creation.
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            w!("STATIC"),
            w!("Polished Map OLE test"),
            WS_OVERLAPPEDWINDOW,
            40,
            60,
            500,
            400,
            0,
            0,
            0,
            ptr::null(),
        )
    };
    assert_ne!(hwnd, 0);
    struct OwnedWindow(isize);
    impl Drop for OwnedWindow {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.0);
            }
        }
    }
    let _window = OwnedWindow(hwnd);
    let released = Rc::new(Cell::new(0));
    let first: Vec<u16> = "C:\\目录\\first.blk".encode_utf16().collect();
    let second = vec![b'C' as u16, 58, 92, 0xd800, 46, 98, 108, 107];
    let expected = vec![
        PathBuf::from(std::ffi::OsString::from_wide(&first)),
        PathBuf::from(std::ffi::OsString::from_wide(&second)),
    ];
    let names = first
        .into_iter()
        .chain([0])
        .chain(second)
        .chain([0, 0])
        .collect();
    let mut data = DataObject {
        vtable: &DATA_VTABLE,
        names,
        released: released.clone(),
    };
    let data_ptr = ptr::from_mut(&mut data).cast::<IDataObject>();
    let events = Rc::new(RefCell::new(Vec::new()));
    let sink = events.clone();
    let handler =
        handler::FileDropHandler::new(hwnd, Box::new(move |event| sink.borrow_mut().push(event)));
    let interface = handler.data.cast::<definitions::IDropTarget>();
    let vtable = unsafe { &*(*interface).lpVtbl };
    let mut screen = POINT { x: 321, y: 123 };
    assert_ne!(unsafe { ClientToScreen(hwnd, &mut screen) }, 0);
    let point = POINTL {
        x: screen.x,
        y: screen.y,
    };
    let mut effect = 1;
    assert_eq!(
        unsafe { (vtable.DragEnter)(interface, data_ptr, 0, point, &mut effect) },
        S_OK
    );
    assert_eq!(released.get(), 1, "hover must release its STGMEDIUM");
    assert_eq!(
        unsafe { (vtable.Drop)(interface, data_ptr, 0, point, &mut effect) },
        S_OK
    );
    assert_eq!(released.get(), 2, "drop honors pUnkForRelease");
    let recorded = events.borrow();
    let complete: Vec<_> = recorded
        .iter()
        .filter_map(|event| match event {
            event::Event::WindowEvent {
                event:
                    event::WindowEvent::FilesDropped {
                        paths, position, ..
                    },
                ..
            } => Some((paths, position)),
            _ => None,
        })
        .collect();
    assert_eq!(complete.len(), 1);
    assert_eq!(complete[0].0, &expected);
    assert_eq!(*complete[0].1, dpi::PhysicalPosition::new(321.0, 123.0));
    assert_eq!(effect, 1);
    let iid = GUID::from_u128(0x00000122_0000_0000_c000_000000000046);
    let mut queried = ptr::null_mut();
    assert_eq!(
        unsafe { (vtable.parent.QueryInterface)(interface.cast(), &iid, &mut queried) },
        S_OK
    );
    assert_eq!(queried, interface.cast());
    unsafe {
        (vtable.parent.Release)(queried.cast());
    }
    drop(recorded);
    let mut effect = 1;
    unsafe {
        (vtable.DragEnter)(interface, data_ptr, 0, point, &mut effect);
        (vtable.DragLeave)(interface);
    }
    assert_eq!(released.get(), 3, "cancelled hover releases its medium");
    let complete_count = events
        .borrow()
        .iter()
        .filter(|event| {
            matches!(
                event,
                event::Event::WindowEvent {
                    event: event::WindowEvent::FilesDropped { .. },
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        complete_count, 1,
        "cancellation must not emit another completed transfer"
    );
}
