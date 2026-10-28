//! Master config.cpp collision precedence and metatileset.cpp row semantics.
//! Preserve untouched ASM lines and suffix data instead of rewriting unrelated source.
use crate::{ProjectError, SaveTarget, document::prepare_target, read_bounded};
use polished_map_core::CollisionRow;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
struct AsmRow {
    line: usize,
    prefix: String,
    suffix: String,
}

#[derive(Debug)]
pub(crate) struct CollisionDocument {
    pub path: PathBuf,
    pub hash: [u8; 32],
    raw: Vec<u8>,
    initial: Vec<CollisionRow>,
    owned: Vec<AsmRow>,
    binary: bool,
}

pub fn prepare_collision_save_as(path: &Path, binary: bool) -> Result<SaveTarget, ProjectError> {
    let path = if path.extension().is_none() {
        path.with_extension(if binary { "bin" } else { "asm" })
    } else {
        path.to_path_buf()
    };
    validate_destination(&path, binary)?;
    prepare_target(&path)
}

pub(crate) fn validate_destination(path: &Path, binary: bool) -> Result<(), ProjectError> {
    let valid = path.extension().is_some_and(|ext| {
        if binary {
            ext.eq_ignore_ascii_case("bin")
        } else {
            ext.eq_ignore_ascii_case("asm") || ext.eq_ignore_ascii_case("inc")
        }
    });
    if valid {
        Ok(())
    } else {
        Err(ProjectError::Format {
            path: path.into(),
            message: "collision destination must keep its .bin or .asm/.inc format".into(),
        })
    }
}

impl CollisionDocument {
    pub fn open(path: &Path, count: usize) -> Result<Self, ProjectError> {
        let path = fs::canonicalize(path).map_err(|e| ProjectError::io(path, e))?;
        let raw = read_bounded(&path, 4 * 1024 * 1024)?;
        Self::from_bytes(path, raw, count)
    }
    pub fn from_bytes(path: PathBuf, raw: Vec<u8>, count: usize) -> Result<Self, ProjectError> {
        if raw.len() > 4 * 1024 * 1024 {
            return Err(ProjectError::Format {
                path,
                message: "collision file exceeds 4 MiB".into(),
            });
        }
        let binary = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("bin"));
        let empty = if binary {
            CollisionRow::Binary([0; 4])
        } else {
            CollisionRow::Asm(std::array::from_fn(|_| String::new()))
        };
        let mut initial = vec![empty; count];
        let mut owned = Vec::new();
        if binary {
            let n = raw.len().min(count * 4);
            if !n.is_multiple_of(4) {
                return Err(ProjectError::Format {
                    path,
                    message: "incomplete four-byte collision row".into(),
                });
            }
            for (i, row) in raw[..n].as_chunks::<4>().0.iter().enumerate() {
                initial[i] = CollisionRow::Binary(*row);
            }
        } else {
            let text = std::str::from_utf8(&raw).map_err(|e| ProjectError::Format {
                path: path.clone(),
                message: e.to_string(),
            })?;
            for (line_index, line) in text.split_inclusive('\n').enumerate() {
                if owned.len() == count {
                    break;
                }
                if !line.starts_with([' ', '\t']) {
                    continue;
                }
                let trimmed = line.trim_start_matches([' ', '\t']);
                let Some(rest) = trimmed
                    .strip_prefix("tilecoll")
                    .filter(|s| s.starts_with(char::is_whitespace))
                else {
                    continue;
                };
                let args = rest.trim_start_matches([' ', '\t']);
                let start = line.len() - args.len();
                let code = args.split(';').next().unwrap_or("").trim_end();
                let fields: Vec<_> = code.split(',').map(str::trim).collect();
                if fields.len() != 4 {
                    return Err(ProjectError::Format {
                        path,
                        message: format!(
                            "collision row on line {} needs four fields",
                            line_index + 1
                        ),
                    });
                }
                initial[owned.len()] = CollisionRow::Asm(std::array::from_fn(|i| fields[i].into()));
                owned.push(AsmRow {
                    line: line_index,
                    prefix: line[..start].into(),
                    suffix: args[code.len()..].into(),
                });
            }
        }
        Ok(Self {
            hash: Sha256::digest(&raw).into(),
            path,
            raw,
            initial,
            owned,
            binary,
        })
    }
    pub fn rows(&self) -> Vec<CollisionRow> {
        self.initial.clone()
    }
    pub fn is_binary(&self) -> bool {
        self.binary
    }
    pub fn encode(&self, rows: &[CollisionRow]) -> Result<Vec<u8>, ProjectError> {
        if rows.iter().any(|r| r.is_binary() != self.binary) {
            return Err(ProjectError::Format {
                path: self.path.clone(),
                message: "collision format changed".into(),
            });
        }
        if rows == self.initial {
            return Ok(self.raw.clone());
        }
        if self.binary {
            let mut out = Vec::new();
            for row in rows {
                if let CollisionRow::Binary(values) = row {
                    out.extend(values);
                }
            }
            out.extend_from_slice(&self.raw[self.raw.len().min(self.initial.len() * 4)..]);
            return Ok(out);
        }
        let text = std::str::from_utf8(&self.raw).map_err(|e| ProjectError::Format {
            path: self.path.clone(),
            message: e.to_string(),
        })?;
        let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
        let mut out = String::new();
        let mut index = 0;
        let append = |out: &mut String| {
            if self.owned.len() < rows.len() {
                if !out.is_empty() && !out.ends_with('\n') {
                    out.push_str(newline);
                }
                for (i, row) in rows.iter().enumerate().skip(self.owned.len()) {
                    if let CollisionRow::Asm(values) = row {
                        out.push_str(&format!(
                            "\ttilecoll {} ; {i:02x}{newline}",
                            values.join(", ")
                        ));
                    }
                }
            }
        };
        for (line_index, line) in text.split_inclusive('\n').enumerate() {
            if let Some(owned) = self.owned.get(index).filter(|r| r.line == line_index) {
                if let Some(row) = rows.get(index) {
                    if row == &self.initial[index] {
                        out.push_str(line);
                    } else if let CollisionRow::Asm(values) = row {
                        out.push_str(&owned.prefix);
                        out.push_str(&values.join(", "));
                        out.push_str(&owned.suffix);
                    }
                }
                index += 1;
                if index == self.owned.len() {
                    append(&mut out);
                }
            } else {
                out.push_str(line);
            }
        }
        if self.owned.is_empty() {
            append(&mut out);
        }
        Ok(out.into_bytes())
    }
}

pub(crate) fn collision_path(root: &Path, tileset: &str) -> Option<PathBuf> {
    let base = tileset.split('.').next().unwrap_or(tileset);
    [tileset, base]
        .into_iter()
        .flat_map(|name| {
            [
                root.join(format!("data/tilesets/{name}_collision.asm")),
                root.join(format!("data/tilesets/{name}_collision.inc")),
                root.join(format!("data/tilesets/{name}_collision.bin")),
                root.join(format!("tilesets/{name}_collision.asm")),
                root.join(format!("tilesets/{name}_collision.bin")),
            ]
        })
        .find(|p| p.is_file())
}
