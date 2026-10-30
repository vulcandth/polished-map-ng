//! Read-only migration of pinned master preferences.cpp and FLTK 1.4.5's
//! Fl_Preferences file format. LGPL-3.0-only adaptation; see NOTICE.md.
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

const MAX_BYTES: usize = 4 * 1024 * 1024;
const FILE_NAME: &str = "Polished Map.prefs";

#[derive(Debug, thiserror::Error)]
pub enum LegacyPreferenceError {
    #[error("could not read legacy preferences: {0}")]
    Read(#[from] crate::ProjectError),
    #[error("could not inspect legacy preference path: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid legacy preferences: {0}")]
    Invalid(&'static str),
}

/// Source platform paths, supplied explicitly so discovery is testable without
/// changing process environment or reading the real user's settings.
pub enum LegacyUserLocation {
    Windows {
        roaming_appdata: Option<PathBuf>,
    },
    Mac {
        home: Option<PathBuf>,
    },
    Unix {
        home: Option<PathBuf>,
        xdg_config_home: Option<PathBuf>,
    },
}

/// First existing executable-adjacent filename wins. On Unix FLTK chooses the
/// vendor *directory* before the file: an existing XDG/Rangi hides ~/.fltk/Rangi
/// even if its Polished Map.prefs is absent. Do not merge fallback stores.
pub fn find_legacy_preferences(
    executable_dir: &Path,
    user: &LegacyUserLocation,
) -> Result<Option<PathBuf>, LegacyPreferenceError> {
    for name in ["polishedmap.prefs", FILE_NAME] {
        let path = executable_dir.join(name);
        if path.try_exists()? {
            return Ok(Some(path));
        }
    }
    let root = match user {
        LegacyUserLocation::Windows { roaming_appdata } => roaming_appdata.clone(),
        LegacyUserLocation::Mac { home } => home.as_ref().map(|p| p.join("Library/Preferences")),
        LegacyUserLocation::Unix {
            home,
            xdg_config_home,
        } => {
            let current = xdg_config_home
                .as_ref()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| expand_home(p, home.as_deref()))
                .or_else(|| home.as_ref().map(|p| p.join(".config")));
            let current_vendor_exists = match &current {
                Some(p) => p.join("Rangi").try_exists()?,
                None => false,
            };
            let old = home.as_ref().map(|p| p.join(".fltk"));
            let old_vendor_exists = match &old {
                Some(p) => p.join("Rangi").try_exists()?,
                None => false,
            };
            if !current_vendor_exists && old_vendor_exists {
                old
            } else {
                current
            }
        }
    };
    let Some(root) = root else {
        return Ok(None);
    };
    let path = root.join("Rangi").join(FILE_NAME);
    Ok(path.try_exists()?.then_some(path))
}

// Replace only the tokens handled by pinned FLTK, preserving native path units.
// No shell expansion or lossy Unicode conversion is involved.
fn expand_home(path: &Path, home: Option<&Path>) -> PathBuf {
    let Some(home) = home else {
        return path.to_owned();
    };
    #[cfg(unix)]
    fn units(s: &OsStr) -> Vec<u16> {
        use std::os::unix::ffi::OsStrExt;
        s.as_bytes().iter().map(|&b| u16::from(b)).collect()
    }
    #[cfg(windows)]
    fn units(s: &OsStr) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        s.encode_wide().collect()
    }
    let mut value = units(path.as_os_str());
    let home = units(home.as_os_str());
    if value.starts_with(&[b'~' as u16, b'/' as u16]) {
        value.splice(..1, home.iter().copied());
    }
    for token in ["${HOME}", "$HOME/"] {
        let needle: Vec<_> = token.encode_utf16().collect();
        if let Some(index) = value.windows(needle.len()).position(|v| v == needle) {
            let len = if token.ends_with('/') {
                needle.len() - 1
            } else {
                needle.len()
            };
            value.splice(index..index + len, home.iter().copied());
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        std::ffi::OsString::from_vec(value.into_iter().map(|v| v as u8).collect()).into()
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        std::ffi::OsString::from_wide(&value).into()
    }
}

/// Root-group values only. Unknown keys/groups are retained or skipped without
/// affecting supported fields. No API writes or modifies the original file.
#[derive(Debug, Default)]
pub struct LegacyPreferences(BTreeMap<String, Vec<u8>>);

impl LegacyPreferences {
    pub fn read(path: &Path) -> Result<Self, LegacyPreferenceError> {
        Self::parse(&crate::read_bounded(path, MAX_BYTES)?)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, LegacyPreferenceError> {
        use LegacyPreferenceError::Invalid;
        if bytes.len() > MAX_BYTES || bytes.contains(&0) {
            return Err(Invalid("oversized file or embedded NUL"));
        }
        let mut lines = bytes
            .split(|&b| b == b'\n')
            .map(|line| line.strip_suffix(b"\r").unwrap_or(line));
        if lines.next() != Some(b"; FLTK preferences file format 1.0".as_slice())
            || !lines.next().is_some_and(|s| s.starts_with(b"; vendor:"))
            || !lines
                .next()
                .is_some_and(|s| s.starts_with(b"; application:"))
        {
            return Err(Invalid("missing or unsupported FLTK header"));
        }
        let mut result = Self::default();
        let mut root = true;
        let mut previous: Option<String> = None;
        for line in lines {
            if line.len() > 1022 || line.contains(&b'\r') {
                return Err(Invalid("invalid physical line length or carriage return"));
            }
            if line.is_empty() {
                continue;
            }
            if line.starts_with(b"[") {
                if !line.ends_with(b"]") {
                    return Err(Invalid("unterminated group"));
                }
                root = line == b"[.]";
                previous = None;
                continue;
            }
            if !root {
                continue;
            }
            if matches!(line[0], b';' | b'#') {
                previous = None;
                continue;
            }
            if let Some(continuation) = line.strip_prefix(b"+") {
                let key = previous.as_ref().ok_or(Invalid("orphan continuation"))?;
                result
                    .0
                    .get_mut(key)
                    .expect("key inserted before continuation")
                    .extend_from_slice(continuation);
                continue;
            }
            let (key, value) = match line.iter().position(|&b| b == b':') {
                Some(i) => (&line[..i], &line[i + 1..]),
                None => (line, b"".as_slice()),
            };
            if key.is_empty() || key.len() > 127 || !key.iter().all(|b| b.is_ascii_graphic()) {
                return Err(Invalid("invalid key"));
            }
            let key = String::from_utf8(key.to_vec()).map_err(|_| Invalid("non-ASCII key"))?;
            result.0.insert(key.clone(), value.to_vec());
            previous = Some(key);
        }
        Ok(result)
    }

    pub fn integer(&self, key: &str, default: i32) -> Result<i32, LegacyPreferenceError> {
        let Some(raw) = self.0.get(key) else {
            return Ok(default);
        };
        let raw = raw.trim_ascii_start();
        let sign = usize::from(matches!(raw.first(), Some(b'+' | b'-')));
        let end = sign
            + raw[sign..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .count();
        if end == sign {
            return Ok(0);
        } // FLTK uses atoi, including prefix parsing.
        std::str::from_utf8(&raw[..end])
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or(LegacyPreferenceError::Invalid("integer overflow"))
    }

    pub fn path(&self, key: &str) -> Result<Option<PathBuf>, LegacyPreferenceError> {
        let Some(raw) = self.0.get(key) else {
            return Ok(None);
        };
        let bytes = decode_text(raw)?;
        if bytes.is_empty() {
            return Ok(None);
        }
        if bytes.contains(&0) {
            return Err(LegacyPreferenceError::Invalid("NUL in recent path"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            Ok(Some(std::ffi::OsString::from_vec(bytes).into()))
        }
        #[cfg(windows)]
        {
            Ok(Some(
                String::from_utf8(bytes)
                    .map_err(|_| LegacyPreferenceError::Invalid("non-UTF-8 Windows path"))?
                    .into(),
            ))
        }
    }
}

fn decode_text(raw: &[u8]) -> Result<Vec<u8>, LegacyPreferenceError> {
    let mut value = Vec::with_capacity(raw.len());
    let mut bytes = raw.iter().copied();
    while let Some(b) = bytes.next() {
        if b != b'\\' {
            value.push(b);
            continue;
        }
        value.push(match bytes.next() {
            Some(b'\\') => b'\\',
            Some(b'n') => b'\n',
            Some(b'r') => b'\r',
            Some(first @ b'0'..=b'3') => {
                let (Some(second @ b'0'..=b'7'), Some(third @ b'0'..=b'7')) =
                    (bytes.next(), bytes.next())
                else {
                    return Err(LegacyPreferenceError::Invalid("incomplete octal escape"));
                };
                (first - b'0') * 64 + (second - b'0') * 8 + (third - b'0')
            }
            _ => return Err(LegacyPreferenceError::Invalid("invalid text escape")),
        });
    }
    Ok(value)
}

#[cfg(test)]
mod tests;
