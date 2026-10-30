//! Public-loader observations against unchanged master Config path functions.
use polished_map_project::{
    BlocksetDocument, MapDocument, MapPalettePlan, MasterGraphicsOptions, MasterTilesetDocument,
    OpenOptions, PaletteDocument, RoofDocument, event_script_path, roof_palette_path,
    tileset_choices,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn write(root: &Path, name: &str, bytes: impl AsRef<[u8]>) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn paths(root: &Path, name: &str) -> Value {
    paths_for_map(
        root,
        name,
        Some(&root.join("maps/Town.blk")),
        Some("landmark_town"),
    )
}

fn paths_for_map(root: &Path, name: &str, map: Option<&Path>, landmark: Option<&str>) -> Value {
    let oracle = std::env::var_os("POLISHED_MAP_ASSET_PATHS_ORACLE")
        .expect("set POLISHED_MAP_ASSET_PATHS_ORACLE");
    let output = Command::new(oracle)
        .arg(root.to_str().unwrap().replace('\\', "/"))
        .arg(name)
        .arg(map.map_or_else(
            || "-".to_owned(),
            |p| p.to_str().unwrap().replace('\\', "/"),
        ))
        .arg(landmark.unwrap_or("-"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn path(reference: &Value, key: &str) -> PathBuf {
    PathBuf::from(reference[key].as_str().unwrap())
}

fn canonical(path: &Path) -> PathBuf {
    // Save targets canonicalize their parent even when the output does not exist.
    fs::canonicalize(path).unwrap_or_else(|_| {
        fs::canonicalize(path.parent().unwrap())
            .unwrap()
            .join(path.file_name().unwrap())
    })
}

fn graphics(root: &Path, directory: &str, name: &str, ext: &str, tiles: usize) {
    let relative = format!("{directory}/{name}.{ext}");
    match ext {
        "png" => {
            let file = root.join(relative);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            image::GrayImage::from_pixel(8, 8 * tiles as u32, image::Luma([255]))
                .save(file)
                .unwrap();
        }
        "CHR" => write(root, &relative, "\tdb 0\n".repeat(16 * tiles)),
        "2bpp.lz" => {
            // Independent literal-run encoding, one tile per command.
            let mut bytes = Vec::new();
            for _ in 0..tiles {
                bytes.push(15);
                bytes.extend([0; 16]);
            }
            bytes.push(255);
            write(root, &relative, bytes);
        }
        "2bpp" => write(root, &relative, vec![0; 16 * tiles]),
        _ => unreachable!(),
    }
}

fn compare_graphics(root: &Path, name: &str) {
    let reference = paths(root, name);
    let actual =
        MasterTilesetDocument::open_in_project(root, name, &MasterGraphicsOptions::default());
    if !path(&reference, "graphics").is_file() {
        assert!(
            actual.is_err(),
            "missing full graphics must not use base graphics"
        );
        return;
    }
    let doc = actual.unwrap();
    let sources: Vec<_> = ["graphics", "before", "after", "palette_map"]
        .into_iter()
        .map(|key| path(&reference, key))
        .filter(|p| p.is_file())
        .map(|p| canonical(&p))
        .collect();
    assert_eq!(
        doc.source_files()
            .iter()
            .map(|s| s.path().to_owned())
            .collect::<Vec<_>>(),
        sources
    );
    let mut outputs: Vec<_> = ["before_png", "png", "after_png"]
        .into_iter()
        .map(|key| path(&reference, key))
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| canonical(&p))
        .collect();
    let palette = path(&reference, "palette_map");
    if palette.is_file() {
        outputs.push(canonical(&palette));
    }
    assert_eq!(
        doc.outputs()
            .iter()
            .map(|s| s.path().to_owned())
            .collect::<Vec<_>>(),
        outputs
    );
}

#[test]
#[ignore = "requires POLISHED_MAP_ASSET_PATHS_ORACLE; run in reference CI"]
fn graphics_directories_extensions_compound_sources_and_outputs_match_source() {
    for directory in ["gfx/tilesets", "MAPDATA"] {
        for name in [
            "demo",
            "demo.before",
            "demo.,after",
            "demo.before,after",
            "demo.before.part,after.part",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path();
            let reference = {
                fs::create_dir_all(root.join(directory)).unwrap();
                paths(root, name)
            };
            // A base file in the lower-priority directory must not be a fallback.
            graphics(root, "gfx/tilesets", "demo", "2bpp", 1);
            let mut parts = vec![name.to_owned()];
            for key in ["before_png", "after_png"] {
                let p = path(&reference, key);
                if let Some(stem) = p.file_stem() {
                    parts.push(stem.to_str().unwrap().to_owned());
                }
            }
            for part in &parts {
                for ext in ["png", "2bpp", "CHR", "2bpp.lz"] {
                    graphics(root, directory, part, ext, 1);
                }
            }
            compare_graphics(root, name);
            // Remove optional sources first, then main. Each rung compares both
            // read precedence and the original PNG destinations (even if absent).
            for part in parts.iter().skip(1).chain(parts.iter().take(1)) {
                for ext in ["png", "2bpp", "CHR", "2bpp.lz"] {
                    fs::remove_file(root.join(format!("{directory}/{part}.{ext}"))).unwrap();
                    compare_graphics(root, name);
                }
            }
        }
    }
    let dir = tempfile::tempdir().unwrap();
    graphics(dir.path(), "gfx/tilesets", "demo", "2bpp", 1);
    fs::create_dir(dir.path().join("MAPDATA")).unwrap();
    compare_graphics(dir.path(), "demo"); // Empty MAPDATA still wins.
}

#[test]
#[ignore = "requires POLISHED_MAP_ASSET_PATHS_ORACLE; run in reference CI"]
fn block_collision_and_palette_map_precedence_match_source() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let name = "demo.before,after";
    graphics(root, "gfx/tilesets", name, "2bpp", 1);
    let block_paths: Vec<_> = [name, "demo"]
        .into_iter()
        .flat_map(|name| {
            [
                format!("data/tilesets/{name}_metatiles.bin"),
                format!("gfx/blocksets/{name}.bst"),
                format!("MAPDATA/{name}.CEL"),
                format!("tilesets/{name}_metatiles.bin"),
            ]
        })
        .collect();
    let collision_paths: Vec<_> = [name, "demo"]
        .into_iter()
        .flat_map(|name| {
            [
                format!("data/tilesets/{name}_collision.asm"),
                format!("data/tilesets/{name}_collision.inc"),
                format!("data/tilesets/{name}_collision.bin"),
                format!("tilesets/{name}_collision.asm"),
                format!("tilesets/{name}_collision.bin"),
            ]
        })
        .collect();
    let palette_paths: Vec<_> = [name, "demo"]
        .into_iter()
        .flat_map(|name| {
            [
                format!("gfx/tilesets/{name}_palette_map.asm"),
                format!("gfx/tilesets/{name}_palette_map.inc"),
                format!("color/tilesets/{name}.asm"),
                format!("tilesets/{name}_palette_map.asm"),
            ]
        })
        .collect();
    for p in &block_paths {
        if p.ends_with(".CEL") {
            write(root, p, "\tdb 0\n".repeat(16));
        } else {
            write(root, p, [0; 16]);
        }
    }
    // The legacy block layout creates MAPDATA; source graphics must follow it.
    graphics(root, "MAPDATA", name, "2bpp", 1);
    for p in &collision_paths {
        if p.ends_with(".bin") {
            write(root, p, [0; 4]);
        } else {
            write(root, p, "\ttilecoll WALL, FLOOR, WALL, FLOOR\n");
        }
    }
    for p in &palette_paths {
        write(root, p, "\ttilepal 0, GRAY\n");
    }
    for p in collision_paths.iter().chain(block_paths.iter()) {
        let reference = paths(root, name);
        let doc = BlocksetDocument::open_in_project(root, name).unwrap();
        assert_eq!(doc.path(), canonical(&path(&reference, "blocks")));
        let collision = path(&reference, "collisions");
        assert_eq!(
            doc.collision_path().map(Path::to_owned),
            collision.is_file().then(|| canonical(&collision))
        );
        if collision.is_file() {
            assert_eq!(
                doc.binary_collisions(),
                reference["binary_collisions"] == "true"
            );
        }
        fs::remove_file(root.join(p)).unwrap();
    }
    assert!(BlocksetDocument::open_in_project(root, name).is_err());
    for p in &palette_paths {
        compare_graphics(root, name);
        fs::remove_file(root.join(p)).unwrap();
    }
    compare_graphics(root, name); // Missing palette maps select monochrome.
}

#[test]
#[ignore = "requires POLISHED_MAP_ASSET_PATHS_ORACLE; run in reference CI"]
fn roofs_events_constants_and_palette_import_paths_match_source() {
    for directory in ["gfx/tilesets", "MAPDATA"] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join(directory)).unwrap();
        let name = "demo.before,after";
        write(root, "maps/Town.blk", [0]);
        write(
            root,
            "data/maps/maps.asm",
            "\tmap Town, TILESET_DEMO, TOWN, LANDMARK_TOWN, MUSIC, FALSE, PALETTE_AUTO\n",
        );
        let doc = MapDocument::open_in_project(
            &root.join("maps/Town.blk"),
            &OpenOptions {
                width: Some(1),
                height: Some(1),
            },
            Some(root),
        )
        .unwrap();
        let palettes = [
            "gfx/tilesets/bg_tiles.pal",
            "tilesets/bg.pal",
            "gfx/tilesets/roofs.pal",
            "tilesets/roof.pal",
        ];
        let specials = [
            "maps/Town.pal".to_owned(),
            format!("{directory}/landmark_town.pal"),
            format!("{directory}/{name}.pal"),
            format!("{directory}/demo.pal"),
        ];
        for p in palettes
            .iter()
            .copied()
            .chain(specials.iter().map(String::as_str))
        {
            write(root, p, "\tRGB 0,0,0\n");
        }
        let check_palettes = || {
            let reference = paths(root, name);
            let mut plan =
                MapPalettePlan::for_map(&doc, name, &MasterGraphicsOptions::default()).unwrap();
            assert_eq!(plan.next_import_path().unwrap(), path(&reference, "bg"));
            plan.advance(&mut PaletteDocument::default(), false)
                .unwrap();
            let special = path(&reference, "special");
            assert_eq!(
                plan.next_import_path().map(canonical),
                special.is_file().then(|| canonical(&special))
            );
            assert_eq!(roof_palette_path(root), path(&reference, "roof_colors"));
        };
        for p in palettes
            .iter()
            .copied()
            .chain(specials.iter().map(String::as_str))
        {
            check_palettes();
            fs::remove_file(root.join(p)).unwrap();
        }
        check_palettes();
        for ext in ["png", "2bpp", "CHR", "2bpp.lz"] {
            graphics(root, "gfx/tilesets/roofs", "roof", ext, 9);
        }
        for ext in ["png", "2bpp", "CHR", "2bpp.lz"] {
            let reference = paths(root, name);
            let result = RoofDocument::open_in_project(root, "roof", [0; 9]);
            if ext == "2bpp.lz" {
                // Original LZ decoding pads to 256 tiles, so a selected LZ roof
                // fails the nine-tile check. Codec behavior has its own oracle.
                let polished_map_project::ProjectError::Format { path: selected, .. } =
                    result.unwrap_err()
                else {
                    panic!("expected oversized LZ roof rejection");
                };
                assert_eq!(selected, path(&reference, "roof"));
                fs::remove_file(root.join("gfx/tilesets/roofs/roof.2bpp.lz")).unwrap();
                continue;
            }
            let roof = result.unwrap();
            assert_eq!(roof.source().path(), canonical(&path(&reference, "roof")));
            assert_eq!(
                roof.output().path(),
                canonical(&path(&reference, "roof_png"))
            );
            fs::remove_file(root.join(format!("gfx/tilesets/roofs/roof.{ext}"))).unwrap();
        }
        assert!(RoofDocument::open_in_project(root, "roof", [0; 9]).is_err());
        let scripts = [
            "maps/Town.asm",
            "SXY/Town.asm",
            "data/maps/objects/Town.asm",
            "data/mapObjects/Town.asm",
        ];
        for p in scripts {
            write(root, p, "");
        }
        for p in scripts {
            assert_eq!(
                event_script_path(root, "Town").unwrap(),
                path(&paths(root, name), "events")
            );
            fs::remove_file(root.join(p)).unwrap();
        }
        assert_eq!(
            event_script_path(root, "Town").unwrap(),
            path(&paths(root, name), "events")
        );
        graphics(root, directory, "01", "2bpp", 1);
        let constants = [
            "constants/tileset_constants.asm",
            "constants/tileset_constants.inc",
            "constants/tilemap_constants.asm",
        ];
        for (i, p) in constants.iter().enumerate() {
            write(root, p, format!("\tconst TILESET_LABEL{i}\n"));
        }
        for p in constants {
            let reference = paths(root, name);
            let source = fs::read_to_string(path(&reference, "constants")).unwrap();
            let expected = source.trim().strip_prefix("const TILESET_").unwrap();
            let choices = tileset_choices(root).unwrap();
            assert_eq!(
                choices.iter().find(|c| c.name == "01").unwrap().label,
                format!("01: {expected}")
            );
            fs::remove_file(root.join(p)).unwrap();
        }
        assert_eq!(
            tileset_choices(root)
                .unwrap()
                .iter()
                .find(|c| c.name == "01")
                .unwrap()
                .label,
            "01"
        );
    }
}

#[test]
#[ignore = "requires POLISHED_MAP_ASSET_PATHS_ORACLE; run in reference CI"]
fn unsaved_maps_and_missing_landmarks_do_not_invent_palette_sources() {
    for directory in ["gfx/tilesets", "MAPDATA"] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let name = "demo.before,after";
        fs::create_dir_all(root.join(directory)).unwrap();
        write(root, "maps/Town.blk", [0]);
        write(root, "maps/Town.pal", "");
        // New has no filename; these tempting names must never be inferred.
        write(root, "NewMap.pal", "");
        write(root, &format!("{directory}/.pal"), "");
        write(root, &format!("{directory}/landmark_town.pal"), "");
        write(root, &format!("{directory}/{name}.pal"), "");
        write(root, &format!("{directory}/demo.pal"), "");
        let saved = MapDocument::open_in_project(
            &root.join("maps/Town.blk"),
            &OpenOptions {
                width: Some(1),
                height: Some(1),
            },
            Some(root),
        )
        .unwrap();
        let new = MapDocument::new_map(root, 1, 1).unwrap();
        for removed in [
            None,
            Some("maps/Town.pal".to_owned()),
            Some(format!("{directory}/{name}.pal")),
            Some(format!("{directory}/demo.pal")),
        ] {
            if let Some(p) = removed {
                fs::remove_file(root.join(p)).unwrap();
            }
            for doc in [&saved, &new] {
                let reference = paths_for_map(
                    root,
                    name,
                    doc.has_backing_file()
                        .then_some(root.join("maps/Town.blk").as_path()),
                    None,
                );
                let mut plan =
                    MapPalettePlan::for_map(doc, name, &MasterGraphicsOptions::default()).unwrap();
                plan.advance(&mut PaletteDocument::default(), false)
                    .unwrap();
                let special = path(&reference, "special");
                assert_eq!(
                    plan.next_import_path().map(canonical),
                    special.is_file().then(|| canonical(&special))
                );
            }
        }
    }
}
