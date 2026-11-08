//! Unified discovery combines pinned option-dialogs.cpp metadata syntax without
//! widening either parity adapter. No game assets are included.
use polished_map_project::{
    FormatDetectionError, ProjectFormat, detect_map_format, discover_unified_map, plusplus,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn file(root: &Path, name: &str, bytes: impl AsRef<[u8]>) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn legacy_dimensions_and_headers_apply_to_occurrence_attributes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    file(root, "Makefile", "");
    file(
        root,
        "constants/map_constants.asm",
        "\tnewgroup\n\tmapconst RED_TOWN, 1, 2\n",
    );
    let header = file(
        root,
        "data/maps/headers/RedTown.asm",
        "\tmap_header RedTown, RED_TOWN, OVERWORLD, 0\n",
    );
    file(root, "MAPDATA/overworld.CHR", "");
    file(root, "data/tilesets/overworld_metatiles.bin", [0; 16]);
    file(root, "data/tilesets/overworld_attributes.bin", [0; 16]);
    let map = file(root, "maps/RedTown.ablk", [0, 0]);
    let detected = detect_map_format(&map, None, None).unwrap();
    assert_eq!(detected.format(), ProjectFormat::PlusPlus);
    assert_eq!(detected.metadata().dimensions, Some((2, 1)));
    assert_eq!(detected.metadata().group, 1);
    assert_eq!(detected.metadata().header_source, Some(header));
    assert_eq!(detected.metadata().tileset.as_deref(), Some("overworld"));
    let doc =
        plusplus::MapDocument::open_unified_in_project_with_report(&map, &Default::default(), None)
            .unwrap()
            .document;
    assert_eq!((doc.map().width(), doc.map().height()), (2, 1));
    assert_eq!(doc.metadata().tileset.as_deref(), Some("overworld"));
    assert!(plusplus::MapDocument::open(&map, &Default::default()).is_err());
    let parity = plusplus::discover_map(&map, Some(root)).unwrap();
    assert!(parity.dimensions.is_none() && parity.header_source.is_none());
}

#[test]
fn ablk_filename_hint_short_circuits_headers_and_resolves_chr_inventory() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    file(root, "MAPDATA/unrelated.CHR", "");
    file(root, "gfx/tilesets/target.CHR", "");
    file(root, "data/tilesets/target_attributes.bin", [0; 16]);
    file(
        root,
        "data/maps/maps.asm",
        "map Demo, TILESET_WRONG, INDOOR, SOMEWHERE, 0, 0, PALETTE_NITE\n",
    );
    let map = file(root, "Demo.2x1.target.ablk", [0, 0]);
    let detected = detect_map_format(&map, Some(root), None).unwrap();
    assert_eq!(detected.metadata().tileset.as_deref(), Some("target"));
    assert_eq!(detected.metadata().dimensions, Some((2, 1)));
    assert!(detected.metadata().header_source.is_none());
    assert!(detected.metadata().environment.is_empty());
    let doc = plusplus::MapDocument::open_unified_in_project_with_report(
        &map,
        &Default::default(),
        Some(root),
    )
    .unwrap()
    .document;
    assert_eq!(doc.metadata().tileset.as_deref(), Some("target"));
}

#[test]
fn conflicting_layout_aliases_require_a_real_tileset_choice() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    file(
        root,
        "constants/tileset_constants.asm",
        "\tconst TILESET_DEMO\n",
    );
    file(root, "MAPDATA/01.CHR", "");
    file(root, "gfx/tilesets/demo.2bpp", [0; 16]);
    file(root, "data/tilesets/01_metatiles.bin", [0; 16]);
    file(root, "data/tilesets/demo_attributes.bin", [0; 16]);
    let map = file(root, "Demo.1x1.demo.blk", [0]);
    let metadata = discover_unified_map(&map, Some(root)).unwrap();
    assert!(metadata.tileset.is_none());
    assert_eq!(metadata.dimensions, Some((1, 1)));
    assert!(matches!(
        detect_map_format(&map, Some(root), None),
        Err(FormatDetectionError::ConflictingTilesets { .. })
    ));
    assert_eq!(
        detect_map_format(&map, Some(root), Some("01"))
            .unwrap()
            .format(),
        ProjectFormat::Master
    );
    assert_eq!(
        detect_map_format(&map, Some(root), Some("demo"))
            .unwrap()
            .format(),
        ProjectFormat::PlusPlus
    );
}

#[test]
fn unified_save_as_uses_union_roots_and_asm_still_needs_binary_destination() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    file(root, "source/main.asm", "");
    file(
        root,
        "source/constants/map_constants.asm",
        "\tmapconst DEMO, 1, 2\n",
    );
    let source = file(root, "source/maps/Demo.map", "\tdb 0, 1\n");
    let mut doc = plusplus::MapDocument::open_unified_in_project_with_report(
        &source,
        &Default::default(),
        None,
    )
    .unwrap()
    .document;
    assert!(doc.save().is_err());
    file(root, "target/Makefile", "");
    let path = root.join("target/Demo.ablk");
    doc.save_as(&plusplus::prepare_save_as(&path).unwrap())
        .unwrap();
    assert_eq!(
        doc.metadata().root.as_deref(),
        Some(root.join("target").canonicalize().unwrap().as_path())
    );
    assert_eq!(fs::read(path).unwrap(), [0, 1]);
    assert_eq!(fs::read_to_string(source).unwrap(), "\tdb 0, 1\n");
}
