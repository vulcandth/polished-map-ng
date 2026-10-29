use polished_map_project::{discover_map, list_roofs, tileset_options};
use std::{fs, path::Path};

fn write(root: &Path, path: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, []).unwrap();
}

#[test]
fn selectors_sort_full_filenames_before_deduplicating_and_preserve_numbered_labels() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for name in [
        "10.png",
        "2.png",
        "01.png",
        "01.2bpp",
        "demo.png",
        "demo.before,after.2bpp.unique.lz",
        "demo.2bpp",
        "ignored.colored.png",
        "note.txt",
    ] {
        write(root, &format!("gfx/tilesets/{name}"));
    }
    write(root, "constants/tileset_constants.inc");
    fs::write(
        root.join("constants/tileset_constants.inc"),
        "\tconst TILESET_JOHTO\n\tconst KANTO\n",
    )
    .unwrap();
    let options = tileset_options(root, Some("johto")).unwrap();
    assert_eq!(
        options
            .choices()
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["01", "2", "10", "demo", "demo.before,after"]
    );
    assert_eq!(options.choices()[0].label, "01: JOHTO");
    assert_eq!(options.selected(), Some(0));
    let compound = tileset_options(root, Some("demo")).unwrap();
    assert_eq!(compound.selected(), Some(4));
    assert_eq!(
        tileset_options(root, Some("unknown")).unwrap().selected(),
        Some(0)
    );
    write(root, "Unknown.1x1.blk");
    assert!(
        discover_map(&root.join("Unknown.1x1.blk"), Some(root))
            .unwrap()
            .tileset
            .is_none()
    );
    for name in [
        "roof10.png",
        "roof2.png",
        "roof2.chr",
        "roof1.2bpp.lz",
        "roof2.colored.png",
    ] {
        write(root, &format!("gfx/tilesets/roofs/{name}"));
    }
    assert_eq!(
        list_roofs(root).unwrap(),
        ["roof1", "roof2", "roof2.colored", "roof10"]
    );
    write(root, "MAPDATA/legacy.CHR");
    assert_eq!(
        tileset_options(root, None).unwrap().choices()[0].name,
        "legacy"
    );
}

#[test]
fn selector_empty_and_numeric_equivalent_names_are_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(tileset_options(dir.path(), None).unwrap().selected(), None);
    for name in [
        "2b.png",
        "02a.png",
        "2a.png",
        "猫10.png",
        "猫2.png",
        "Z.png",
        "a.png",
    ] {
        write(dir.path(), &format!("gfx/tilesets/{name}"));
    }
    let options = tileset_options(dir.path(), None).unwrap();
    assert_eq!(
        options
            .choices()
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["02a", "2a", "2b", "Z", "a", "猫2", "猫10"]
    );
}
