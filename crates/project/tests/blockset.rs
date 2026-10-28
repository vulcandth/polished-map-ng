use polished_map_core::BlockCommand;
use polished_map_project::{BlocksetDocument, ProjectError, prepare_blockset_save_as};
use std::fs;

#[test]
fn blockset_save_reopens_exact_bytes_and_external_changes_conflict() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("demo.bin");
    fs::write(&path, [0; 32]).unwrap();
    let mut doc = BlocksetDocument::open(&path).unwrap();
    doc.blockset_mut()
        .apply(&BlockCommand::Set {
            block: 1,
            tiles: [255; 16],
        })
        .unwrap();
    doc.save().unwrap();
    assert!(!doc.blockset().is_modified());
    assert_eq!(
        BlocksetDocument::open(&path).unwrap().blockset().bytes(),
        [[0; 16], [255; 16]].concat()
    );
    doc.blockset_mut()
        .apply(&BlockCommand::Copy { from: 1, to: 0 })
        .unwrap();
    fs::write(&path, [7; 32]).unwrap();
    assert!(matches!(doc.save(), Err(ProjectError::Conflict(_))));
    assert!(doc.blockset().is_modified());
    assert_eq!(fs::read(path).unwrap(), [7; 32]);
}

#[test]
fn cel_import_never_overwrites_assembly_and_copy_rechecks_destination() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("test.CEL");
    let text = "Label:\n\tdb 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15 ; original\n";
    fs::write(&source, text).unwrap();
    let mut doc = BlocksetDocument::open(&source).unwrap();
    assert!(doc.needs_save_as());
    assert!(doc.save().is_err());
    let path = dir.path().join("copy.bin");
    let target = prepare_blockset_save_as(&path).unwrap();
    fs::write(&path, [7; 16]).unwrap();
    assert!(matches!(
        doc.save_as(&target),
        Err(ProjectError::Conflict(_))
    ));
    let target = prepare_blockset_save_as(&path).unwrap();
    doc.save_as(&target).unwrap();
    assert_eq!(fs::read(&path).unwrap(), (0..16).collect::<Vec<u8>>());
    assert_eq!(fs::read_to_string(source).unwrap(), text);
    assert!(!doc.needs_save_as());
    assert!(prepare_blockset_save_as(&dir.path().join("wrong.blk")).is_err());
    let map_target = polished_map_project::prepare_save_as(&dir.path().join("map.blk")).unwrap();
    assert!(doc.save_as(&map_target).is_err());
    assert!(!map_target.path().exists());
    let mut map = polished_map_project::MapDocument::new_map(dir.path(), 1, 1).unwrap();
    let block_target = prepare_blockset_save_as(&dir.path().join("other.bin")).unwrap();
    assert!(map.save_as(&block_target).is_err());
    assert!(!block_target.path().exists());
}
