use polished_map_core::{BlockCommand, CollisionRow};
use polished_map_project::{
    BlocksetDocument, ProjectError, prepare_blockset_save_as, prepare_collision_save_as,
};
use std::{fs, path::Path};

fn write(root: &Path, path: &str, bytes: impl AsRef<[u8]>) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
fn row(value: &str) -> CollisionRow {
    CollisionRow::Asm(std::array::from_fn(|_| value.into()))
}

#[test]
fn collision_discovery_prefers_full_name_before_base_and_asm_before_binary() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "data/tilesets/demo_metatiles.bin", [0; 16]);
    write(
        root,
        "data/tilesets/demo_collision.asm",
        "\ttilecoll BASE, BASE, BASE, BASE\n",
    );
    write(
        root,
        "data/tilesets/demo.before_collision.bin",
        [1, 2, 3, 4],
    );
    let doc = BlocksetDocument::open_in_project(root, "demo.before").unwrap();
    assert!(doc.binary_collisions());
    assert_eq!(
        doc.blockset().collisions().unwrap(),
        [CollisionRow::Binary([1, 2, 3, 4])]
    );
    write(
        root,
        "data/tilesets/demo.before_collision.inc",
        "\ttilecoll INC, INC, INC, INC\n",
    );
    let doc = BlocksetDocument::open_in_project(root, "demo.before").unwrap();
    assert!(!doc.binary_collisions());
    assert_eq!(doc.blockset().collisions().unwrap(), [row("INC")]);
    write(
        root,
        "data/tilesets/demo.before_collision.asm",
        "\ttilecoll ASM, ASM, ASM, ASM\n",
    );
    assert_eq!(
        BlocksetDocument::open_in_project(root, "demo.before")
            .unwrap()
            .blockset()
            .collisions()
            .unwrap(),
        [row("ASM")]
    );
}

#[test]
fn paired_edits_preserve_unrelated_asm_comments_line_endings_and_suffix() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let blocks = root.join("blocks.bin");
    let collisions = root.join("collision.asm");
    let text = "; Keep header\r\nTable:\r\n\ttilecoll WALL, WALL, WALL, WALL  ; custom comment\r\n\ttilecoll FLOOR, FLOOR, FLOOR, FLOOR ; 01\r\nOtherLabel:\r\n\tdb 42\r\n";
    fs::write(&blocks, [0; 32]).unwrap();
    fs::write(&collisions, text).unwrap();
    let mut doc = BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).unwrap();
    doc.blockset_mut()
        .apply_batch(&[
            BlockCommand::Set {
                block: 1,
                tiles: [7; 16],
            },
            BlockCommand::SetCollisions {
                block: 0,
                collision: row("WATER"),
            },
        ])
        .unwrap();
    doc.save().unwrap();
    assert_eq!(
        fs::read_to_string(&collisions).unwrap(),
        text.replace("WALL, WALL, WALL, WALL", "WATER, WATER, WATER, WATER")
    );
    assert_eq!(fs::read(&blocks).unwrap(), [[0; 16], [7; 16]].concat());
    assert!(!doc.blockset().is_modified());
    assert_eq!(
        BlocksetDocument::open_with_collisions(&blocks, Some(&collisions))
            .unwrap()
            .blockset()
            .collisions()
            .unwrap()[0],
        row("WATER")
    );
    assert_eq!(fs::read_dir(root).unwrap().count(), 2);
}

#[test]
fn external_collision_change_prevents_either_file_being_written() {
    let dir = tempfile::tempdir().unwrap();
    let blocks = dir.path().join("blocks.bin");
    let collisions = dir.path().join("collision.bin");
    fs::write(&blocks, [0; 16]).unwrap();
    fs::write(&collisions, [1, 2, 3, 4]).unwrap();
    let mut doc = BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).unwrap();
    doc.blockset_mut()
        .apply(&BlockCommand::Set {
            block: 0,
            tiles: [8; 16],
        })
        .unwrap();
    fs::write(&collisions, [9; 4]).unwrap();
    assert!(matches!(doc.save(), Err(ProjectError::Conflict(_))));
    assert_eq!(fs::read(blocks).unwrap(), [0; 16]);
    assert_eq!(fs::read(collisions).unwrap(), [9; 4]);
    assert!(doc.blockset().is_modified());
}

#[test]
fn binary_resize_pads_missing_rows_and_preserves_unowned_suffix() {
    let dir = tempfile::tempdir().unwrap();
    let blocks = dir.path().join("blocks.bin");
    let collisions = dir.path().join("collision.bin");
    fs::write(&blocks, [0; 32]).unwrap();
    fs::write(&collisions, [1, 2, 3, 4, 5, 6, 7, 8, 99]).unwrap();
    let mut doc = BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).unwrap();
    doc.blockset_mut()
        .apply_batch(&[
            BlockCommand::Copy { from: 1, to: 0 },
            BlockCommand::Resize { count: 3 },
        ])
        .unwrap();
    doc.save().unwrap();
    assert_eq!(
        fs::read(&collisions).unwrap(),
        [5, 6, 7, 8, 5, 6, 7, 8, 0, 0, 0, 0, 99]
    );
    fs::write(&collisions, [7; 4]).unwrap();
    let doc = BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).unwrap();
    assert_eq!(
        doc.blockset().collisions().unwrap()[1],
        CollisionRow::Binary([0; 4])
    );
    fs::write(&collisions, [7; 3]).unwrap();
    assert!(BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).is_err());
}

#[test]
fn asm_resize_inserts_rows_before_unowned_data_and_paired_copy_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let blocks = root.join("blocks.bin");
    let collisions = root.join("collision.asm");
    fs::write(&blocks, [0; 16]).unwrap();
    fs::write(&collisions, "\ttilecoll A, B, C, D ; 00\nTail:\n\tdb 42\n").unwrap();
    let mut doc = BlocksetDocument::open_with_collisions(&blocks, Some(&collisions)).unwrap();
    doc.blockset_mut()
        .apply(&BlockCommand::Resize { count: 2 })
        .unwrap();
    let target = prepare_blockset_save_as(&root.join("copy.bin")).unwrap();
    assert!(doc.save_as(&target).is_err());
    assert!(!target.path().exists());
    let other = prepare_collision_save_as(&root.join("copy.asm"), false).unwrap();
    doc.save_as_pair(&target, &other).unwrap();
    assert_eq!(
        fs::read_to_string(other.path()).unwrap(),
        "\ttilecoll A, B, C, D ; 00\n\ttilecoll , , ,  ; 01\nTail:\n\tdb 42\n"
    );
    assert_eq!(fs::read(&blocks).unwrap(), [0; 16]);
    assert_eq!(
        fs::read_to_string(collisions).unwrap(),
        "\ttilecoll A, B, C, D ; 00\nTail:\n\tdb 42\n"
    );
    doc.blockset_mut()
        .apply(&BlockCommand::Resize { count: 1 })
        .unwrap();
    doc.save().unwrap();
    assert_eq!(
        fs::read_to_string(other.path()).unwrap(),
        "\ttilecoll A, B, C, D ; 00\nTail:\n\tdb 42\n"
    );
}
