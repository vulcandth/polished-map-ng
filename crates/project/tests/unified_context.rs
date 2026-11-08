use polished_map_core::{Command, Point};
use polished_map_project::{MapDocument, OpenOptions, ProjectError, plusplus};
use std::fs;

#[test]
fn storage_rebinding_keeps_edits_history_source_hash_and_explicit_save_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Map.blk");
    fs::write(&path, [0, 1]).unwrap();
    let options = OpenOptions {
        width: Some(2),
        height: Some(1),
    };
    let mut original = MapDocument::open_in_project(&path, &options, Some(dir.path())).unwrap();
    original
        .map_mut()
        .apply(&Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block: 9,
        })
        .unwrap();
    original.map_mut().undo();
    let mut attributed = original.with_plusplus_tileset("attributed").unwrap();
    assert_eq!(attributed.content_hash(), original.content_hash());
    assert!(attributed.map().can_redo());
    attributed.map_mut().redo();
    assert_eq!(attributed.map().blocks(), [9, 1]);
    assert!(attributed.needs_save_as());
    assert!(matches!(
        attributed.save(),
        Err(ProjectError::DestinationRequired)
    ));
    let mut classic = attributed.with_classic_tileset("classic").unwrap();
    assert!(classic.needs_save_as()); // Switching back does not silently authorize the old source.
    assert!(classic.map().can_undo());
    classic.map_mut().undo();
    assert_eq!(classic.map().blocks(), [0, 1]);
    assert!(classic.map().can_redo());
    let copy = dir.path().join("Copy.ablk");
    attributed
        .save_as(&plusplus::prepare_save_as(&copy).unwrap())
        .unwrap();
    assert!(!attributed.needs_save_as());
    assert_eq!(fs::read(copy).unwrap(), [9, 1]);
    assert_eq!(fs::read(path).unwrap(), [0, 1]);
    assert!(original.with_plusplus_tileset("../bad").is_err());
}

#[test]
fn unified_new_map_keeps_union_root_policy_after_save_as() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("Makefile"), "").unwrap();
    let mut doc = plusplus::MapDocument::new_unified_map(dir.path(), 1, 1).unwrap();
    doc.save_as(&plusplus::prepare_save_as(&target.join("Map.ablk")).unwrap())
        .unwrap();
    assert_eq!(
        doc.metadata().root.as_deref(),
        Some(target.canonicalize().unwrap().as_path())
    );
}
