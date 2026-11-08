use super::*;
use std::fs;

fn file(root: &Path, name: &str, contents: &[u8]) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn shared_extension_uses_selected_assets_not_markers_or_unrelated_tilesets() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    file(root, "Makefile", b"");
    file(root, "main.asm", b"");
    let map = file(root, "maps/Unknown.blk", &[0]);
    file(root, "data/tilesets/mono_metatiles.bin", &[0; 16]);
    file(root, "data/tilesets/color_metatiles.bin", &[0; 16]);
    file(root, "data/tilesets/color_attributes.bin", &[0; 16]);
    file(root, "gfx/tilesets/mono.2bpp", &[0; 16]);
    file(root, "gfx/tilesets/color.2bpp", &[0; 16]);
    assert!(matches!(
        detect_map_format(&map, None, None),
        Err(FormatDetectionError::ContextRequired { .. })
    ));
    let mono = detect_map_format(&map, None, Some("mono")).unwrap();
    assert_eq!(mono.format(), ProjectFormat::Master);
    assert_eq!(mono.metadata().root.as_deref(), Some(root));
    assert!(
        mono.evidence()
            .iter()
            .any(|e| e.kind == FormatEvidenceKind::UnattributedBlocks)
    );
    assert_eq!(
        detect_map_format(&map, None, Some("color"))
            .unwrap()
            .format(),
        ProjectFormat::PlusPlus
    );
    // Shared file bytes and unrelated assets did not change.
    assert_eq!(fs::read(map).unwrap(), [0]);
}

#[test]
fn filename_hint_is_resolved_without_picking_the_first_graphics_file() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    file(root, "Makefile", b"");
    file(root, "gfx/tilesets/aaa.2bpp", &[0; 16]);
    file(root, "gfx/tilesets/target.2bpp", &[0; 16]);
    file(root, "data/tilesets/target_attributes.bin", &[0; 16]);
    let map = file(root, "maps/Demo.1x1.target.blk", &[0]);
    let detected = detect_map_format(&map, None, None).unwrap();
    assert_eq!(detected.format(), ProjectFormat::PlusPlus);
    assert_eq!(detected.metadata().tileset.as_deref(), Some("target"));
    assert_eq!(detected.metadata().dimensions, Some((1, 1)));
    // Missing offsets are the selected loader's error, never a master fallback.
    assert!(crate::plusplus::BlocksetDocument::open_in_project(root, "target").is_err());
}

#[test]
fn conflicting_assets_are_explained_and_explicit_extensions_do_not_hide_them() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let attrs = file(root, "data/tilesets/demo_attributes.bin", &[0; 16]);
    let palettes = file(
        root,
        "gfx/tilesets/demo_palette_map.asm",
        b"\ttilepal 0, GRAY\n",
    );
    for name in ["Map.blk", "Map.ablk", "Map.map"] {
        let error = detect_map_format(&root.join(name), Some(root), Some("demo")).unwrap_err();
        let FormatDetectionError::ConflictingEvidence { evidence, .. } = error else {
            panic!("{error}")
        };
        assert!(evidence.iter().any(|e| e.path == attrs));
        assert!(evidence.iter().any(|e| e.path == palettes));
    }
}

#[test]
fn compound_resolution_is_shared_with_actual_loaders() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    file(root, "data/tilesets/demo_metatiles.bin", &[7; 16]);
    file(root, "data/tilesets/demo_attributes.bin", &[0; 16]);
    let preferred = file(root, "tilesets/demo.extra_attributes.bin", &[1; 16]);
    let map = root.join("Demo.blk");
    let detected = detect_map_format(&map, Some(root), Some("demo.extra")).unwrap();
    assert_eq!(detected.evidence()[0].path, preferred);
    assert_eq!(
        AssetKind::Attributes.path(root, "demo.extra").unwrap(),
        preferred
    );
    crate::plusplus::BlocksetDocument::open_in_project(root, "demo.extra").unwrap();
    // A malformed preferred attribute file must fail, not try the valid base.
    fs::write(&preferred, [0]).unwrap();
    assert_eq!(
        detect_map_format(&map, Some(root), Some("demo.extra"))
            .unwrap()
            .format(),
        ProjectFormat::PlusPlus
    );
    assert!(crate::plusplus::BlocksetDocument::open_in_project(root, "demo.extra").is_err());
}

#[test]
fn source_and_legacy_layouts_remain_distinguishable() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let blocks = file(
        root,
        "MAPDATA/demo.CEL",
        b"\tdb 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0\n",
    );
    let detected = detect_map_format(&root.join("Map.map"), Some(root), Some("demo")).unwrap();
    assert_eq!(detected.format(), ProjectFormat::Master);
    assert_eq!(detected.encoding(), MapEncoding::Assembly);
    assert!(
        detected
            .evidence()
            .iter()
            .any(|e| e.path == blocks && e.kind == FormatEvidenceKind::LegacyBlockset)
    );
    crate::BlocksetDocument::open_in_project(root, "demo").unwrap();
    assert_eq!(
        detect_map_format(&root.join("Map.ABLK"), None, None)
            .unwrap()
            .format(),
        ProjectFormat::PlusPlus
    );
    assert!(detect_map_format(&root.join("Map.png"), Some(root), None).is_err());
    assert!(detect_map_format(&root.join("Map.blk"), Some(root), Some("../demo")).is_err());
}

#[test]
fn asm_map_encoding_does_not_require_per_tile_attributes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    file(root, "data/tilesets/demo_attributes.bin", &[0; 16]);
    let map = file(root, "Demo.map", b"\tdb 0\n");
    let detected = detect_map_format(&map, Some(root), Some("demo")).unwrap();
    assert_eq!(detected.format(), ProjectFormat::PlusPlus);
    assert_eq!(detected.encoding(), MapEncoding::Assembly);
    assert_eq!(fs::read(map).unwrap(), b"\tdb 0\n");
}

#[test]
fn nearest_union_root_and_explicit_root_are_independent_of_format() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    file(root, "Makefile", b"");
    file(root, "nested/layout.link", b"");
    let map = file(root, "nested/maps/Demo.blk", &[0]);
    assert_eq!(
        find_unified_project_root(&map).unwrap(),
        Some(root.join("nested"))
    );
    file(root, "data/tilesets/demo_metatiles.bin", &[0; 16]);
    assert!(detect_map_format(&map, None, Some("demo")).is_err());
    assert_eq!(
        detect_map_format(&map, Some(root), Some("demo"))
            .unwrap()
            .format(),
        ProjectFormat::Master
    );
}

#[test]
fn asset_io_failure_is_not_absence_or_an_alternate_format() {
    let temp = tempfile::tempdir().unwrap();
    // A file in place of a path component produces a real IO error, unlike a
    // nonexistent candidate, on both Unix and Windows.
    file(temp.path(), "data", b"not a directory");
    assert!(matches!(
        detect_map_format(
            &temp.path().join("Map.blk"),
            Some(temp.path()),
            Some("demo")
        ),
        Err(FormatDetectionError::Project(ProjectError::Io { .. }))
    ));
}

#[test]
fn shared_collision_resolution_retains_both_parity_precedences() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    file(root, "data/tilesets/demo_metatiles.bin", &[0; 16]);
    file(root, "data/tilesets/demo_attributes.bin", &[0; 16]);
    let binary = file(root, "data/tilesets/demo_collision.bin", &[0; 4]);
    let asm = file(
        root,
        "tilesets/demo_collision.asm",
        b"\ttilecoll WALL, WALL, WALL, WALL\n",
    );
    let master = crate::BlocksetDocument::open_in_project(root, "demo").unwrap();
    let plusplus = crate::plusplus::BlocksetDocument::open_in_project(root, "demo").unwrap();
    assert_eq!(
        master.collision_path(),
        Some(fs::canonicalize(binary).unwrap().as_path())
    );
    assert_eq!(
        plusplus.collision_path(),
        Some(fs::canonicalize(asm).unwrap().as_path())
    );
}
