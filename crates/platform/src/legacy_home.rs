//! FLTK 1.4.5 Fl_cocoa.mm::getUserPreferencesDirectory, pinned in upstream.lock.
//! HOME, NSHomeDirectory, then the real user's passwd entry. Read-only discovery.
use std::path::PathBuf;

/// Discover the legacy macOS home. Other platforms return None.
pub fn legacy_macos_home() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        select(
            std::env::var_os("HOME").map(PathBuf::from),
            || {
                Some(PathBuf::from(
                    objc2_foundation::NSHomeDirectory().to_string(),
                ))
            },
            native::passwd_home,
        )
    }
    #[cfg(not(target_os = "macos"))]
    None
}

#[cfg(any(target_os = "macos", test))]
fn select(
    home: Option<PathBuf>,
    foundation: impl FnOnce() -> Option<PathBuf>,
    passwd: impl FnOnce() -> Option<PathBuf>,
) -> Option<PathBuf> {
    let exists = |p: &PathBuf| !p.as_os_str().is_empty() && p.exists();
    home.filter(exists)
        .or_else(|| foundation().filter(exists))
        .or_else(passwd)
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod native {
    use super::*;
    use std::{
        ffi::{CStr, OsString},
        os::unix::ffi::OsStringExt,
    };

    pub(super) fn passwd_home() -> Option<PathBuf> {
        // getpwuid_r keeps the native record in caller-owned storage, avoiding
        // getpwuid's process-global buffer. Bound retries on unusually large entries.
        let mut buffer = vec![0_u8; 1024];
        loop {
            let mut entry = std::mem::MaybeUninit::<libc::passwd>::uninit();
            let mut result = std::ptr::null_mut();
            // SAFETY: entry, result and the length-checked writable buffer live
            // through this call. No pointer is read until success/non-null result.
            let error = unsafe {
                libc::getpwuid_r(
                    libc::getuid(),
                    entry.as_mut_ptr(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                    &mut result,
                )
            };
            if error == libc::ERANGE && buffer.len() < 1024 * 1024 {
                buffer.resize(buffer.len() * 2, 0);
                continue;
            }
            if error != 0 || result.is_null() {
                return None;
            }
            // SAFETY: successful getpwuid_r initialized entry and its pw_dir
            // points to a NUL-terminated string in buffer, copied before it drops.
            let directory = unsafe { entry.assume_init().pw_dir };
            if directory.is_null() {
                return None;
            }
            let bytes = unsafe { CStr::from_ptr(directory) }.to_bytes().to_vec();
            return (!bytes.is_empty()).then(|| PathBuf::from(OsString::from_vec(bytes)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_fallbacks_are_ordered_existing_and_lazy() {
        let valid = tempfile::tempdir().unwrap();
        let path = valid.path().to_path_buf();
        assert_eq!(
            select(
                Some(path.clone()),
                || panic!("valid HOME wins"),
                || panic!("valid HOME wins")
            ),
            Some(path.clone())
        );
        for home in [None, Some(PathBuf::new()), Some(path.join("missing"))] {
            assert_eq!(
                select(
                    home.clone(),
                    || Some(path.clone()),
                    || panic!("Foundation wins")
                ),
                Some(path.clone())
            );
            assert_eq!(
                select(
                    home,
                    || Some(path.join("also-missing")),
                    || Some(path.clone())
                ),
                Some(path.clone())
            );
        }
        assert_eq!(select(None, || None, || None), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_macos_home_fallbacks_without_environment_mutation() {
        let home = select(
            Some(PathBuf::from("/nonexistent-polished-map-home")),
            || {
                Some(PathBuf::from(
                    objc2_foundation::NSHomeDirectory().to_string(),
                ))
            },
            native::passwd_home,
        )
        .unwrap();
        assert!(home.is_dir());
        assert!(native::passwd_home().unwrap().is_dir());
    }
}
