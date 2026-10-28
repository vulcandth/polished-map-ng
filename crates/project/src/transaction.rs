//! Recoverable staging for paired block/collision writes. This is not a filesystem
//! transaction: each replacement is atomic, and external writers must still be checked.
use crate::{
    ProjectError,
    document::{optional_hash, persist_bytes},
    read_bounded,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub(crate) struct Change {
    pub path: PathBuf,
    pub expected: Option<[u8; 32]>,
    pub bytes: Vec<u8>,
}

#[derive(Serialize)]
struct Entry<'a> {
    target: &'a Path,
    before_sha256: Option<[u8; 32]>,
    after_sha256: [u8; 32],
    before_file: Option<String>,
    after_file: String,
}

pub(crate) fn persist_pair(changes: &[Change]) -> Result<(), ProjectError> {
    persist_with(changes, |change| {
        persist_bytes(&change.path, change.expected, &change.bytes)
    })
}

fn durable_write(path: &Path, bytes: &[u8]) -> Result<(), ProjectError> {
    let mut file = fs::File::create(path).map_err(|e| ProjectError::io(path, e))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| ProjectError::io(path, e))
}

fn persist_with(
    changes: &[Change],
    mut write: impl FnMut(&Change) -> Result<(), ProjectError>,
) -> Result<(), ProjectError> {
    if changes.is_empty() {
        return Ok(());
    }
    let mut before = Vec::new();
    for (i, change) in changes.iter().enumerate() {
        if changes[..i].iter().any(|c| c.path == change.path) {
            return Err(ProjectError::Format {
                path: change.path.clone(),
                message: "paired destinations must be different files".into(),
            });
        }
        if optional_hash(&change.path)? != change.expected {
            return Err(ProjectError::Conflict(change.path.clone()));
        }
        before.push(if change.expected.is_some() {
            Some(read_bounded(&change.path, 4 * 1024 * 1024)?)
        } else {
            None
        });
        if before[i]
            .as_ref()
            .map(|b| <[u8; 32]>::from(Sha256::digest(b)))
            != change.expected
        {
            return Err(ProjectError::Conflict(change.path.clone()));
        }
    }
    let parent = changes[0]
        .path
        .parent()
        .ok_or_else(|| ProjectError::Format {
            path: changes[0].path.clone(),
            message: "no parent directory".into(),
        })?;
    let recovery = tempfile::Builder::new()
        .prefix(".polished-map-recovery-")
        .tempdir_in(parent)
        .map_err(|e| ProjectError::io(parent, e))?;
    let mut entries = Vec::new();
    for (i, change) in changes.iter().enumerate() {
        let before_file = before[i].as_ref().map(|_| format!("before-{i}.bin"));
        if let Some(bytes) = &before[i] {
            durable_write(&recovery.path().join(before_file.as_ref().unwrap()), bytes)?;
        }
        let after_file = format!("after-{i}.bin");
        durable_write(&recovery.path().join(&after_file), &change.bytes)?;
        entries.push(Entry {
            target: &change.path,
            before_sha256: change.expected,
            after_sha256: Sha256::digest(&change.bytes).into(),
            before_file,
            after_file,
        });
    }
    let manifest=serde_json::to_vec_pretty(&serde_json::json!({"version":1,"purpose":"paired blockset/collision save","entries":entries})).map_err(|e|ProjectError::Format{path:recovery.path().into(),message:e.to_string()})?;
    durable_write(&recovery.path().join("manifest.json"), &manifest)?;
    #[cfg(unix)]
    fs::File::open(recovery.path())
        .and_then(|f| f.sync_all())
        .map_err(|e| ProjectError::io(recovery.path(), e))?;
    // Preflight all hashes again after staging, before the first destination changes.
    for change in changes {
        if optional_hash(&change.path)? != change.expected {
            return Err(ProjectError::Conflict(change.path.clone()));
        }
    }
    for (i, change) in changes.iter().enumerate() {
        if let Err(error) = write(change) {
            let mut rollback_error = None;
            for j in (0..i).rev() {
                let expected = Some(<[u8; 32]>::from(Sha256::digest(&changes[j].bytes)));
                let result = if let Some(bytes) = &before[j] {
                    persist_bytes(&changes[j].path, expected, bytes)
                } else {
                    match optional_hash(&changes[j].path) {
                        Err(e) => Err(e),
                        Ok(hash) if hash != expected => {
                            Err(ProjectError::Conflict(changes[j].path.clone()))
                        }
                        Ok(_) => fs::remove_file(&changes[j].path)
                            .map_err(|e| ProjectError::io(&changes[j].path, e)),
                    }
                };
                if let Err(e) = result {
                    rollback_error = Some(e.to_string());
                }
            }
            if let Some(rollback) = rollback_error {
                return Err(ProjectError::RecoveryRequired {
                    path: recovery.keep(),
                    message: format!("{error}; rollback: {rollback}"),
                });
            }
            return Err(error);
        }
    }
    // TempDir removes only this owned staging directory. A process crash leaves it
    // with before/after bytes and a manifest for inspection and manual recovery.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failure_removes_new_first_destination_and_duplicate_targets_never_write() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        let changes = [
            Change {
                path: a.clone(),
                expected: None,
                bytes: vec![3],
            },
            Change {
                path: b.clone(),
                expected: None,
                bytes: vec![4],
            },
        ];
        let mut count = 0;
        let result = persist_with(&changes, |change| {
            count += 1;
            if count == 2 {
                return Err(ProjectError::Conflict(b.clone()));
            }
            persist_bytes(&change.path, change.expected, &change.bytes)
        });
        assert!(result.is_err());
        assert!(!a.exists());
        assert!(!b.exists());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
        let changes = [
            Change {
                path: a.clone(),
                expected: None,
                bytes: vec![3],
            },
            Change {
                path: a.clone(),
                expected: None,
                bytes: vec![4],
            },
        ];
        assert!(persist_pair(&changes).is_err());
        assert!(!a.exists());
    }
    #[test]
    fn second_write_failure_restores_first_and_preserves_external_writer() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        fs::write(&a, [1]).unwrap();
        fs::write(&b, [2]).unwrap();
        let changes = [
            Change {
                path: a.clone(),
                expected: optional_hash(&a).unwrap(),
                bytes: vec![3],
            },
            Change {
                path: b.clone(),
                expected: optional_hash(&b).unwrap(),
                bytes: vec![4],
            },
        ];
        let mut count = 0;
        let result = persist_with(&changes, |change| {
            count += 1;
            if count == 2 {
                fs::write(&b, [9]).unwrap();
            }
            persist_bytes(&change.path, change.expected, &change.bytes)
        });
        assert!(matches!(result, Err(ProjectError::Conflict(_))));
        assert_eq!(fs::read(a).unwrap(), [1]);
        assert_eq!(fs::read(b).unwrap(), [9]);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }
    #[test]
    fn failed_rollback_keeps_recovery_files_and_never_clobbers_new_external_data() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        fs::write(&a, [1]).unwrap();
        fs::write(&b, [2]).unwrap();
        let changes = [
            Change {
                path: a.clone(),
                expected: optional_hash(&a).unwrap(),
                bytes: vec![3],
            },
            Change {
                path: b.clone(),
                expected: optional_hash(&b).unwrap(),
                bytes: vec![4],
            },
        ];
        let mut count = 0;
        let result = persist_with(&changes, |change| {
            count += 1;
            if count == 2 {
                fs::write(&a, [8]).unwrap();
                return Err(ProjectError::Conflict(b.clone()));
            }
            persist_bytes(&change.path, change.expected, &change.bytes)
        });
        let Err(ProjectError::RecoveryRequired { path, .. }) = result else {
            panic!("expected recovery files");
        };
        assert_eq!(fs::read(a).unwrap(), [8]);
        assert_eq!(fs::read(b).unwrap(), [2]);
        assert_eq!(fs::read(path.join("before-0.bin")).unwrap(), [1]);
        assert_eq!(fs::read(path.join("after-1.bin")).unwrap(), [4]);
        assert!(path.join("manifest.json").is_file());
    }
}
