use polished_map_core::{Command, Point};
use polished_map_project::{MapDocument, OpenOptions, ProjectError};
use std::fs;

#[test]
fn edit_save_reopen_and_external_changes_are_detected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("日本語.2x1.test.blk");
    fs::write(&path, [1, 2]).unwrap();
    let mut doc = MapDocument::open(&path, &OpenOptions::default()).unwrap();
    doc.map_mut()
        .apply(&Command::Paint {
            points: vec![Point { x: 1, y: 0 }],
            block: 9,
        })
        .unwrap();
    doc.save().unwrap();
    assert!(!doc.map().is_modified());
    assert_eq!(
        MapDocument::open(&path, &OpenOptions::default())
            .unwrap()
            .map()
            .blocks(),
        [1, 9]
    );
    doc.map_mut()
        .apply(&Command::Replace { from: 1, to: 3 })
        .unwrap();
    fs::write(&path, [4, 5]).unwrap();
    assert!(matches!(doc.save(), Err(ProjectError::Conflict(_))));
    assert!(doc.map().is_modified());
    assert_eq!(fs::read(&path).unwrap(), [4, 5]);
}

#[test]
fn invalid_dimensions_or_length_never_rewrite_input() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.blk");
    fs::write(&path, [1, 2]).unwrap();
    assert!(MapDocument::open(&path, &OpenOptions::default()).is_err());
    assert!(
        MapDocument::open(
            &path,
            &OpenOptions {
                width: Some(1),
                height: Some(1)
            }
        )
        .is_err()
    );
    assert_eq!(fs::read(&path).unwrap(), [1, 2]);
}
