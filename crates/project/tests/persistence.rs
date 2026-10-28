use polished_map_core::{Command, Point};
use polished_map_project::{MapDocument, OpenOptions, ProjectError, prepare_save_as};
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

#[test]
fn asm_save_as_creates_binary_without_changing_source_and_switches_backing_file() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.map");
    let asm = "\tdb $12, $34 ; preserve source\n";
    fs::write(&source, asm).unwrap();
    let mut doc = MapDocument::open(
        &source,
        &OpenOptions {
            width: Some(2),
            height: Some(1),
        },
    )
    .unwrap();
    assert!(doc.save().is_err());
    let target = prepare_save_as(&dir.path().join("日本語.2x1.blk")).unwrap();
    assert!(!target.exists());
    doc.save_as(&target).unwrap();
    assert_eq!(fs::read(target.path()).unwrap(), [0x12, 0x34]);
    assert_eq!(fs::read_to_string(&source).unwrap(), asm);
    assert_eq!(doc.path(), target.path());
    doc.map_mut()
        .apply(&Command::Replace { from: 0x12, to: 7 })
        .unwrap();
    doc.save().unwrap();
    assert_eq!(fs::read(target.path()).unwrap(), [7, 0x34]);
}

#[test]
fn save_as_requires_unchanged_observed_destination_and_preserves_document_on_failure() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.2x1.blk");
    fs::write(&source, [1, 2]).unwrap();
    let mut doc = MapDocument::open(&source, &OpenOptions::default()).unwrap();
    doc.map_mut()
        .apply(&Command::Replace { from: 1, to: 9 })
        .unwrap();
    let old_path = doc.path().to_path_buf();
    let path = dir.path().join("copy");
    let absent = prepare_save_as(&path).unwrap();
    assert_eq!(absent.path().extension().unwrap(), "blk");
    fs::write(absent.path(), [5, 6]).unwrap();
    assert!(matches!(
        doc.save_as(&absent),
        Err(ProjectError::Conflict(_))
    ));
    assert_eq!(doc.path(), old_path);
    assert!(doc.map().is_modified());
    let observed = prepare_save_as(absent.path()).unwrap();
    assert!(observed.exists());
    fs::write(observed.path(), [7, 8]).unwrap();
    assert!(matches!(
        doc.save_as(&observed),
        Err(ProjectError::Conflict(_))
    ));
    assert_eq!(fs::read(observed.path()).unwrap(), [7, 8]);
    let confirmed = prepare_save_as(observed.path()).unwrap();
    doc.save_as(&confirmed).unwrap();
    assert_eq!(fs::read(confirmed.path()).unwrap(), [9, 2]);
    assert_eq!(fs::read(source).unwrap(), [1, 2]);
    assert!(!doc.map().is_modified());
    assert!(prepare_save_as(&dir.path().join("unsafe.map")).is_err());
}

#[test]
fn new_map_is_dirty_without_a_backing_file_and_cannot_overwrite_suggested_name() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join("NewMap.blk");
    fs::write(&existing, [8, 9]).unwrap();
    let mut doc = MapDocument::new_map(dir.path(), 3, 2).unwrap();
    assert_eq!(doc.map().blocks(), [0; 6]);
    assert!(doc.map().is_modified());
    assert!(!doc.map().can_undo());
    assert!(!doc.has_backing_file());
    assert!(doc.summary().path.is_none());
    assert!(doc.content_hash().is_none());
    assert!(matches!(doc.save(), Err(ProjectError::DestinationRequired)));
    assert_eq!(fs::read(existing).unwrap(), [8, 9]);
    let target = prepare_save_as(&dir.path().join("Created.3x2.blk")).unwrap();
    doc.save_as(&target).unwrap();
    assert!(doc.has_backing_file());
    assert!(!doc.map().is_modified());
    assert_eq!(
        MapDocument::open(target.path(), &OpenOptions::default())
            .unwrap()
            .map()
            .blocks(),
        [0; 6]
    );
    assert!(MapDocument::new_map(dir.path(), u16::MAX, u16::MAX).is_err());
}
