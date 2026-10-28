//! Windows association and line navigation from master view_event_script.
//! LGPL-3.0-only. All pointers and COM lifetime are confined to this module.
#![allow(unsafe_code)]
use crate::ScriptOpenError;
use std::{
    ffi::{OsStr, OsString},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::Path,
    ptr,
};
use windows_sys::Win32::{
    System::Com::{
        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
    },
    UI::Shell::{FindExecutableW, ShellExecuteW},
};

struct Apartment;
impl Apartment {
    fn new() -> Result<Self, ScriptOpenError> {
        // SAFETY: null reserved argument; this worker initializes its own STA.
        // Every successful call (including S_FALSE) is balanced by Drop below.
        let result = unsafe {
            CoInitializeEx(
                ptr::null(),
                (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
            )
        };
        if result < 0 {
            return Err(ScriptOpenError::Platform(format!(
                "Cannot initialize the Windows script opener (0x{result:08X})"
            )));
        }
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: this value never leaves the synchronous open call's thread,
        // and exists only after a successful CoInitializeEx on that thread.
        unsafe { CoUninitialize() };
    }
}

fn shell_path(path: &Path) -> Result<OsString, ScriptOpenError> {
    use std::path::{Component, Prefix};
    let mut components = path.components();
    let mut result = match components.next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::VerbatimDisk(letter) => {
                std::path::PathBuf::from(format!("{}:\\", char::from(letter)))
            }
            Prefix::VerbatimUNC(server, share) => {
                let mut value = OsString::from("\\\\");
                value.push(server);
                value.push("\\");
                value.push(share);
                std::path::PathBuf::from(value)
            }
            Prefix::Disk(_) | Prefix::UNC(_, _) => return Ok(path.as_os_str().to_owned()),
            _ => {
                return Err(ScriptOpenError::Platform(
                    "Unsupported Windows event script path".into(),
                ));
            }
        },
        _ => {
            return Err(ScriptOpenError::Platform(
                "An absolute Windows event script path is required".into(),
            ));
        }
    };
    for component in components {
        if component != Component::RootDir {
            result.push(component);
        }
    }
    Ok(result.into_os_string())
}

fn editor_arguments(program: &Path, file: &OsStr, line: usize) -> Option<Vec<OsString>> {
    let name = program.file_name()?.to_str()?.to_ascii_lowercase();
    let mut location = file.to_owned();
    location.push(format!(":{line}"));
    match name.as_str() {
        "notepad2.exe" | "notepad3.exe" => {
            Some(vec!["/g".into(), line.to_string().into(), file.to_owned()])
        }
        "notepad++.exe" => Some(vec![format!("-n{line}").into(), file.to_owned()]),
        "code.exe" => Some(vec!["-g".into(), location]),
        "subl.exe" | "sublime_text.exe" => Some(vec![location]),
        _ => None,
    }
}

fn wide(value: &OsStr) -> Result<Vec<u16>, ScriptOpenError> {
    let mut value: Vec<_> = value.encode_wide().collect();
    if value.contains(&0) || value.len() >= 32767 {
        return Err(ScriptOpenError::Platform(
            "Invalid or overlong Windows script opener argument".into(),
        ));
    }
    value.push(0);
    Ok(value)
}

/// Quote each argument using Windows CRT backslash/quote rules. This is an
/// executable parameter string, never a cmd.exe/PowerShell command.
fn parameters(args: &[OsString]) -> Result<Vec<u16>, ScriptOpenError> {
    let mut result = Vec::new();
    for (index, arg) in args.iter().enumerate() {
        if index > 0 {
            result.push(32);
        }
        result.push(34);
        let mut slashes = 0;
        for unit in arg.encode_wide() {
            if unit == 92 {
                slashes += 1;
                continue;
            }
            result.extend(std::iter::repeat_n(
                92,
                slashes * if unit == 34 { 2 } else { 1 },
            ));
            slashes = 0;
            if unit == 34 {
                result.push(92);
            }
            result.push(unit);
        }
        result.extend(std::iter::repeat_n(92, slashes * 2));
        result.push(34);
    }
    wide(&OsString::from_wide(&result))
}

pub(super) fn open(path: &Path, line: Option<usize>) -> Result<(), ScriptOpenError> {
    let file = shell_path(path)?;
    let filename = wide(&file)?;
    let _apartment = Apartment::new()?;
    let mut program = [0u16; 260]; // FindExecutableW requires a MAX_PATH buffer.
    let executable = if line.is_some() {
        // SAFETY: filename is NUL-terminated; output is writable MAX_PATH units.
        // Null directory uses the current directory, as in the original.
        let result =
            unsafe { FindExecutableW(filename.as_ptr(), ptr::null(), program.as_mut_ptr()) }
                as isize;
        if result > 32 {
            program
                .iter()
                .position(|&unit| unit == 0)
                .filter(|&n| n > 0)
                .map(|n| OsString::from_wide(&program[..n]))
        } else {
            None
        }
    } else {
        None
    };
    let args = executable
        .as_ref()
        .zip(line)
        .and_then(|(program, line)| editor_arguments(Path::new(program), &file, line));
    let (verb, target, params) = if let (Some(program), Some(args)) = (executable, args) {
        (
            wide(OsStr::new("open"))?,
            wide(&program)?,
            Some(parameters(&args)?),
        )
    } else {
        (wide(OsStr::new("edit"))?, filename, None)
    };
    // SAFETY: all strings are bounded, NUL-terminated and live across the call.
    // Null owner is permitted; no borrowed GUI handle crosses to this worker.
    // COM is initialized until the call completes. 1 is SW_SHOWNORMAL.
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            verb.as_ptr(),
            target.as_ptr(),
            params.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            ptr::null(),
            1,
        )
    } as isize;
    if result > 32 {
        Ok(())
    } else {
        Err(ScriptOpenError::Platform(format!(
            "Windows could not open the event script (ShellExecute error {result}); check the .asm edit association"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_editor_lines_and_fallback_match_upstream() {
        let file = OsStr::new("C:\\maps ü\\Map $ &.asm");
        for (name, expected) in [
            ("Notepad2.EXE", vec!["/g", "42", "C:\\maps ü\\Map $ &.asm"]),
            ("notepad3.exe", vec!["/g", "42", "C:\\maps ü\\Map $ &.asm"]),
            ("notepad++.exe", vec!["-n42", "C:\\maps ü\\Map $ &.asm"]),
            ("Code.exe", vec!["-g", "C:\\maps ü\\Map $ &.asm:42"]),
            ("subl.exe", vec!["C:\\maps ü\\Map $ &.asm:42"]),
            ("sublime_text.exe", vec!["C:\\maps ü\\Map $ &.asm:42"]),
        ] {
            assert_eq!(
                editor_arguments(&Path::new("C:\\Editors").join(name), file, 42).unwrap(),
                expected.into_iter().map(OsString::from).collect::<Vec<_>>()
            );
        }
        assert!(editor_arguments(Path::new("notepad.exe"), file, 42).is_none());
        assert!(editor_arguments(Path::new("evilcode.exe"), file, 42).is_none());
    }
    #[test]
    fn windows_paths_and_argument_quoting_are_lossless() {
        for (source, expected) in [
            (r"\\?\C:\map ü\a.asm", r"C:\map ü\a.asm"),
            (r"\\?\UNC\server\share\a.asm", r"\\server\share\a.asm"),
        ] {
            assert_eq!(shell_path(Path::new(source)).unwrap(), expected);
        }
        let args = vec![
            OsString::from(""),
            OsString::from("a\"b"),
            OsString::from("C:\\a b\\"),
            OsString::from("$&;"),
        ];
        let encoded = parameters(&args).unwrap();
        assert_eq!(
            String::from_utf16(&encoded[..encoded.len() - 1]).unwrap(),
            "\"\" \"a\\\"b\" \"C:\\a b\\\\\" \"$&;\""
        );
        assert!(wide(OsStr::new("bad\0file")).is_err());
        assert!(wide(&OsString::from("a".repeat(32767))).is_err());
        let unpaired = OsString::from_wide(&[0xd800]);
        assert_eq!(wide(&unpaired).unwrap(), [0xd800, 0]);
    }
}
