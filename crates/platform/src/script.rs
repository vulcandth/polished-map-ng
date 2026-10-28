//! External script editing adapted from master main-window.cpp::view_event_script.
//! LGPL-3.0-only. Launch only in response to an explicit user action.
use std::{
    fmt, io,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum ScriptOpenError {
    InvalidLine,
    NotFile,
    Io(io::Error),
    Platform(String),
}
impl fmt::Display for ScriptOpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLine => f.write_str("Event script line numbers start at one"),
            Self::NotFile => f.write_str("The event script is not a regular file"),
            Self::Io(e) => write!(f, "Cannot open event script: {e}"),
            Self::Platform(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for ScriptOpenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
impl From<io::Error> for ScriptOpenError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// A request opens the saved file; it never writes unsaved GUI edits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptOpenRequest {
    path: PathBuf,
    line: Option<NonZeroUsize>,
}
impl ScriptOpenRequest {
    pub fn new(path: PathBuf, line: Option<usize>) -> Result<Self, ScriptOpenError> {
        let line = line
            .map(|line| NonZeroUsize::new(line).ok_or(ScriptOpenError::InvalidLine))
            .transpose()?;
        Ok(Self { path, line })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn line(&self) -> Option<usize> {
        self.line.map(NonZeroUsize::get)
    }

    /// Blocking desktop operation: call on a worker thread, never the GUI thread.
    /// Success means the OS opener accepted/completed the request, not that a
    /// particular editor is visible or has navigated to the requested line.
    pub fn open(&self) -> Result<(), ScriptOpenError> {
        let path = self.file_path()?;
        #[cfg(windows)]
        {
            super::script_windows::open(&path, self.line())
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let mut command = desktop_command(&path);
            let status = command.stdin(std::process::Stdio::null()).status()?;
            if status.success() {
                Ok(())
            } else {
                Err(ScriptOpenError::Platform(format!(
                    "The event script opener failed ({status})"
                )))
            }
        }
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        {
            let _ = path;
            Err(ScriptOpenError::Platform(
                "External script editing is unsupported on this platform".into(),
            ))
        }
    }
    fn file_path(&self) -> Result<PathBuf, ScriptOpenError> {
        let path = self.path.canonicalize()?;
        if !path.is_file() {
            return Err(ScriptOpenError::NotFile);
        }
        Ok(path)
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn desktop_command(path: &Path) -> std::process::Command {
    // Canonical absolute paths cannot be interpreted as options or URLs.
    // No shell: spaces, quotes, dollars and non-UTF8 path bytes remain data.
    let mut command = std::process::Command::new(if cfg!(target_os = "macos") {
        "/usr/bin/open"
    } else {
        "/usr/bin/xdg-open"
    });
    command.arg(path);
    command
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn script_requests_validate_files_and_one_based_lines_without_launching() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            ScriptOpenRequest::new(dir.path().into(), Some(0)),
            Err(ScriptOpenError::InvalidLine)
        ));
        let request = ScriptOpenRequest::new(dir.path().into(), None).unwrap();
        assert!(matches!(request.file_path(), Err(ScriptOpenError::NotFile)));
        let path = dir.path().join("map ü $ &.asm");
        let request = ScriptOpenRequest::new(path.clone(), Some(42)).unwrap();
        assert!(matches!(request.file_path(), Err(ScriptOpenError::Io(_))));
        std::fs::write(&path, "\tbg_event 0, 0, SIGN\n").unwrap();
        assert_eq!(request.line(), Some(42));
        assert_eq!(request.path(), path);
        assert!(request.file_path().unwrap().is_absolute());
    }
    #[cfg(unix)]
    #[test]
    fn desktop_arguments_preserve_non_unicode_and_shell_characters() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};
        let path = PathBuf::from(OsString::from_vec(b"/tmp/-map '\"$;\xff.asm".to_vec()));
        let command = desktop_command(&path);
        assert_eq!(command.get_args().collect::<Vec<_>>(), [path.as_os_str()]);
        assert_eq!(
            command.get_program(),
            if cfg!(target_os = "macos") {
                "/usr/bin/open"
            } else {
                "/usr/bin/xdg-open"
            }
        );
    }
}
