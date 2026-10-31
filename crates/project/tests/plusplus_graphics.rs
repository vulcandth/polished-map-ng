use polished_map_core::{
    Palette, Point, Tile,
    plusplus::{AttributeBlockset, BankOrder, RawTileCommand as C},
};
use polished_map_project::{
    PngPlatform, ProjectError, codecs,
    plusplus::{RoofDocument, TilesetDocument, read_tiles},
};
use std::{fs, path::Path};

fn component(root: &Path, name: &str, n: usize, hue: u8) {
    fs::create_dir_all(root.join("gfx/tilesets")).unwrap();
    fs::write(
        root.join(format!("gfx/tilesets/{name}.2bpp")),
        codecs::encode_2bpp(&vec![Tile::new([hue; 64]).unwrap(); n]),
    )
    .unwrap();
}
fn blocks() -> AttributeBlockset {
    AttributeBlockset::from_bytes(&[0; 16], &[0; 16]).unwrap()
}

#[test]
fn compound_load_pads_rows_and_save_preserves_binary_sources_and_component_boundaries() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for (name, hue) in [("before", 1), ("main.before,after", 2), ("after", 3)] {
        component(root, name, 17, hue);
    }
    let mut doc =
        TilesetDocument::open_in_project(root, "main.before,after", PngPlatform::Unix).unwrap();
    assert_eq!(doc.loaded_tile_count(), 81);
    assert!(!doc.enables_512_tiles());
    for (i, hue, defined) in [
        (0, 1, true),
        (16, 1, true),
        (17, 0, false),
        (32, 2, true),
        (48, 2, true),
        (49, 0, false),
        (64, 3, true),
        (80, 3, true),
        (81, 0, false),
    ] {
        assert_eq!(*doc.model().tiles()[i].hues(), [hue; 64]);
        assert_eq!(doc.model().defined()[i], defined);
    }
    let sources = doc
        .sources()
        .iter()
        .map(|s| (s.path().to_path_buf(), fs::read(s.path()).unwrap()))
        .collect::<Vec<_>>();
    let before_hash = doc.content_hash();
    doc.apply_batch(&[C::Paint {
        tile: 32,
        points: vec![Point { x: 0, y: 0 }],
        hue: 3,
    }])
    .unwrap();
    let plan = doc.preview_save(&blocks(), BankOrder::default()).unwrap();
    assert_eq!(plan.len(), 3);
    assert!(plan.iter().all(|p| p.previous_hash.is_none()));
    assert!(doc.outputs().iter().all(|p| !p.path().exists()));
    doc.save(&blocks(), BankOrder::default()).unwrap();
    assert!(!doc.model().is_modified());
    assert_ne!(before_hash, doc.content_hash());
    for (path, bytes) in sources {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    for (name, count, hue) in [
        ("before", 32, 1),
        ("main.before,after", 32, 2),
        ("after", 32, 3),
    ] {
        let tiles = read_tiles(
            &root.join(format!("gfx/tilesets/{name}.png")),
            PngPlatform::Unix,
        )
        .unwrap();
        assert_eq!(tiles.len(), count);
        assert_eq!(tiles[0].hues()[1], hue);
    }
    let reopened =
        TilesetDocument::open_in_project(root, "main.before,after", PngPlatform::Unix).unwrap();
    assert_eq!(reopened.model().tiles()[32].hues()[0], 3);
    assert_eq!(reopened.loaded_tile_count(), 96);
    // After PNG promotion, unrelated edits to the old binary import do not conflict.
    fs::write(root.join("gfx/tilesets/before.2bpp"), b"obsolete").unwrap();
    doc.apply_batch(&[C::Paint {
        tile: 33,
        points: vec![Point { x: 0, y: 0 }],
        hue: 0,
    }])
    .unwrap();
    doc.save(&blocks(), BankOrder::default()).unwrap();
}

#[test]
fn compound_conflicts_and_failed_exports_do_not_write_any_output() {
    for conflict in 0..6 {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for name in ["before", "main.before,after", "after"] {
            component(root, name, 16, 1);
        }
        let mut doc =
            TilesetDocument::open_in_project(root, "main.before,after", PngPlatform::Unix).unwrap();
        doc.apply_batch(&[C::Paint {
            tile: 16,
            points: vec![Point { x: 0, y: 0 }],
            hue: 2,
        }])
        .unwrap();
        let paths = doc
            .sources()
            .into_iter()
            .chain(doc.outputs())
            .map(|p| p.path().to_path_buf())
            .collect::<Vec<_>>();
        fs::write(&paths[conflict], b"external writer").unwrap();
        let before = paths.iter().map(|p| fs::read(p).ok()).collect::<Vec<_>>();
        assert!(matches!(
            doc.save(&blocks(), BankOrder::default()),
            Err(ProjectError::Conflict(_))
        ));
        assert_eq!(
            paths.iter().map(|p| fs::read(p).ok()).collect::<Vec<_>>(),
            before
        );
        assert!(doc.model().is_modified());
    }
    let dir = tempfile::tempdir().unwrap();
    component(dir.path(), "main.,missing", 16, 1);
    let mut doc =
        TilesetDocument::open_in_project(dir.path(), "main.,missing", PngPlatform::Unix).unwrap();
    doc.apply_batch(&[C::Paint {
        tile: 0,
        points: vec![Point { x: 0, y: 0 }],
        hue: 2,
    }])
    .unwrap();
    assert!(doc.save(&blocks(), BankOrder::default()).is_err());
    assert!(doc.outputs().iter().all(|p| !p.path().exists()));
}

#[test]
fn empty_optional_before_retains_source_output_semantics_and_total_is_bounded() {
    let dir = tempfile::tempdir().unwrap();
    component(dir.path(), "main.missing", 16, 2);
    let mut doc =
        TilesetDocument::open_in_project(dir.path(), "main.missing", PngPlatform::Unix).unwrap();
    assert_eq!(doc.loaded_tile_count(), 16);
    doc.apply_batch(&[C::Paint {
        tile: 0,
        points: vec![Point { x: 0, y: 0 }],
        hue: 3,
    }])
    .unwrap();
    doc.save(&blocks(), BankOrder::default()).unwrap();
    // Original output count 0 means infer from the sheet, even for absent before.
    assert_eq!(
        fs::read(dir.path().join("gfx/tilesets/missing.png")).unwrap(),
        fs::read(dir.path().join("gfx/tilesets/main.missing.png")).unwrap()
    );
    component(dir.path(), "huge.absent", 512, 1);
    let mut huge =
        TilesetDocument::open_in_project(dir.path(), "huge.absent", PngPlatform::Unix).unwrap();
    huge.apply_batch(&[C::Paint {
        tile: 0,
        points: vec![Point { x: 0, y: 0 }],
        hue: 2,
    }])
    .unwrap();
    assert!(huge.preview_save(&blocks(), BankOrder::default()).is_err());
    assert!(huge.save(&blocks(), BankOrder::default()).is_err());
    assert!(huge.outputs().iter().all(|p| !p.path().exists()));
    component(dir.path(), "duplicate.shared,shared", 16, 1);
    component(dir.path(), "shared", 16, 2);
    let mut duplicate =
        TilesetDocument::open_in_project(dir.path(), "duplicate.shared,shared", PngPlatform::Unix)
            .unwrap();
    duplicate
        .apply_batch(&[C::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 2,
        }])
        .unwrap();
    assert!(
        duplicate
            .preview_save(&blocks(), BankOrder::default())
            .is_err()
    );
    assert!(duplicate.save(&blocks(), BankOrder::default()).is_err());
    assert!(duplicate.outputs().iter().all(|p| !p.path().exists()));
    component(dir.path(), "full.prefix", 512, 1);
    component(dir.path(), "prefix", 1, 1);
    assert!(
        TilesetDocument::open_in_project(dir.path(), "full.prefix", PngPlatform::Unix).is_err()
    );
    component(dir.path(), "expanded", 257, 1);
    assert!(
        TilesetDocument::open_in_project(dir.path(), "expanded", PngPlatform::Unix)
            .unwrap()
            .enables_512_tiles()
    );
}

#[test]
fn roof_reuses_pixel_editor_and_png_saves_without_touching_base_graphics() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("gfx/tilesets/roofs")).unwrap();
    let input = root.join("gfx/tilesets/roofs/demo.2bpp");
    fs::write(
        &input,
        codecs::encode_2bpp(&vec![Tile::new([1; 64]).unwrap(); 9]),
    )
    .unwrap();
    component(root, "base", 32, 2);
    let base = fs::read(root.join("gfx/tilesets/base.2bpp")).unwrap();
    let mut roof = RoofDocument::open_in_project(root, "demo", PngPlatform::Unix).unwrap();
    assert!(!roof.is_modified());
    let old = roof.snapshot(0).unwrap();
    assert!(
        roof.apply_batch(&[
            C::Paint {
                tile: 0,
                points: vec![Point { x: 0, y: 0 }],
                hue: 2
            },
            C::Delete { tile: 1 }
        ])
        .is_err()
    );
    assert_eq!(roof.snapshot(0).unwrap(), old);
    roof.apply_batch(&[C::ImportRgb {
        tile: 8,
        width: 1,
        height: 1,
        pixels: vec![0; 3],
        allow_512: true,
    }])
    .unwrap();
    assert_eq!(roof.snapshot(8).unwrap().pixels[0], 3);
    assert_eq!(roof.snapshot(8).unwrap().pixels[1], 1);
    roof.save().unwrap();
    assert!(!roof.is_modified());
    let reopened = RoofDocument::open_in_project(root, "demo", PngPlatform::Unix).unwrap();
    assert_eq!(reopened.tiles(), roof.tiles());
    assert_eq!(fs::read(root.join("gfx/tilesets/base.2bpp")).unwrap(), base);
    assert!(roof.snapshot(9).is_err());
    assert!(roof.snapshot(256).is_err());
    roof.apply_batch(&[C::Fill {
        tile: 0,
        at: Point { x: 0, y: 0 },
        hue: 3,
    }])
    .unwrap();
    fs::write(roof.output().path(), b"external writer").unwrap();
    assert!(matches!(roof.save(), Err(ProjectError::Conflict(_))));
    assert!(roof.is_modified());
}

#[test]
#[ignore = "requires independently fetched pinned Polished Crystal project"]
fn real_graphics_edit_save_reopen_preserves_rendering_and_roofs() {
    let root = std::path::PathBuf::from(
        std::env::var_os("POLISHED_CRYSTAL_FIXTURE").expect("set fixture"),
    );
    let temp = tempfile::tempdir().unwrap();
    let dest = temp.path();
    fs::create_dir_all(dest.join("gfx/tilesets/roofs")).unwrap();
    // Only temporary copies are edited. Never redistribute the fixture or renders.
    for entry in fs::read_dir(root.join("gfx/tilesets")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "png") {
            fs::copy(
                &path,
                dest.join("gfx/tilesets").join(path.file_name().unwrap()),
            )
            .unwrap();
        }
    }
    let colors: [Palette; 8] = std::array::from_fn(|i| polished_map_core::day_palettes()[i]);
    let map = polished_map_core::Map::from_bytes(16, 16, (0..=255).collect()).unwrap();
    let mut tested = 0;
    for entry in fs::read_dir(root.join("data/tilesets")).unwrap() {
        let path = entry.unwrap().path();
        let Some(base) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix("_metatiles.bin"))
        else {
            continue;
        };
        let names = fs::read_dir(root.join("gfx/tilesets"))
            .unwrap()
            .filter_map(|e| {
                let path = e.unwrap().path();
                if path.extension().is_none_or(|e| e != "png") {
                    return None;
                }
                let name = path.file_stem()?.to_str()?;
                (name == base || name.starts_with(&format!("{base}."))).then(|| name.to_owned())
            })
            .collect::<Vec<_>>();
        assert!(!names.is_empty(), "missing graphics {base}");
        for name in names {
            let b = polished_map_project::plusplus::BlocksetDocument::open_in_project(&root, &name)
                .unwrap();
            let mut doc = TilesetDocument::open_in_project(dest, &name, PngPlatform::Unix).unwrap();
            let index = doc.model().defined().iter().rposition(|&d| d).unwrap() as u16;
            let hue = (doc.model().tiles()[usize::from(index)].hues()[0] + 1) % 4;
            doc.apply_batch(&[C::Paint {
                tile: index,
                points: vec![Point { x: 0, y: 0 }],
                hue,
            }])
            .unwrap();
            let expected = doc
                .graphics(b.blockset(), colors, BankOrder::default())
                .render_map(&map)
                .unwrap();
            doc.save(b.blockset(), BankOrder::default()).unwrap();
            let reopened =
                TilesetDocument::open_in_project(dest, &name, PngPlatform::Unix).unwrap();
            assert_eq!(
                reopened.model().tiles()[usize::from(index)].hues()[0],
                hue,
                "{name}"
            );
            assert_eq!(
                reopened
                    .graphics(b.blockset(), colors, BankOrder::default())
                    .render_map(&map)
                    .unwrap(),
                expected,
                "{name}"
            );
            tested += 1;
        }
    }
    assert_eq!(tested, 51);
    let mut roofs = 0;
    for entry in fs::read_dir(root.join("gfx/tilesets/roofs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "png") {
            continue;
        }
        fs::copy(
            &path,
            dest.join("gfx/tilesets/roofs")
                .join(path.file_name().unwrap()),
        )
        .unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();
        let mut roof = RoofDocument::open_in_project(dest, name, PngPlatform::Unix).unwrap();
        roof.apply_batch(&[C::Paint {
            tile: 8,
            points: vec![Point { x: 7, y: 7 }],
            hue: 2,
        }])
        .unwrap();
        roof.save().unwrap();
        assert_eq!(
            roof.tiles(),
            RoofDocument::open_in_project(dest, name, PngPlatform::Unix)
                .unwrap()
                .tiles()
        );
        roofs += 1;
    }
    assert_eq!(roofs, 6);
}
