use polished_map_core::{Blockset, Point, TileCommand};
use polished_map_project::{MasterGraphicsOptions, MasterTilesetDocument, ProjectError};
use std::{fs, path::Path};

fn write(root: &Path, name: &str, bytes: impl AsRef<[u8]>) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "gfx/tilesets/demo.2bpp",
        [[0, 255].repeat(8), [255, 0].repeat(8), vec![255; 16]].concat(),
    );
    write(
        root,
        "gfx/tilesets/demo_palette_map.asm",
        "; header\r\nTable:\r\n\ttilepal 0, GRAY, RED, GREEN, TEXT ; retain\r\nFooter:\r\n\tdb 42\r\n",
    );
}
fn edit(doc: &mut MasterTilesetDocument) {
    doc.apply_batch(&[
        TileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 3,
        },
        TileCommand::Palette {
            tile: 0,
            palette: 3,
        },
    ])
    .unwrap();
}

#[test]
fn empty_raw_and_chr_images_compose_blank_slots_then_edit_save_and_reopen() {
    for (extension, bytes) in [("2bpp", ""), ("CHR", ""), ("CHR", "Label:\n; no tiles\n")] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let name = format!("gfx/tilesets/demo.{extension}");
        write(root, &name, bytes);
        let mut doc =
            MasterTilesetDocument::open_in_project(root, "demo", &MasterGraphicsOptions::default())
                .unwrap();
        assert_eq!(doc.tileset().tiles().len(), 256);
        assert!(doc.tileset().tiles().iter().all(|t| t.hues() == &[0; 64]));
        assert!(!doc.tileset().is_modified());
        let blocks = Blockset::from_bytes(&[0; 16]).unwrap();
        assert!(doc.preview_save(&blocks).unwrap().is_empty());
        doc.apply_batch(&[TileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 3,
        }])
        .unwrap();
        doc.save(&blocks).unwrap();
        assert!(!doc.tileset().is_modified());
        assert_eq!(fs::read(root.join(&name)).unwrap(), bytes.as_bytes());
        let png = image::open(root.join("gfx/tilesets/demo.png"))
            .unwrap()
            .to_rgb8();
        assert_eq!(png.dimensions(), (128, 8));
        assert_eq!(png.get_pixel(0, 0).0, [0; 3]);
        assert_eq!(png.get_pixel(1, 0).0, [255; 3]);
        let reopened =
            MasterTilesetDocument::open_in_project(root, "demo", &MasterGraphicsOptions::default())
                .unwrap();
        assert_eq!(reopened.tileset().tiles(), doc.tileset().tiles());
    }
}

#[test]
fn duplicate_compound_outputs_and_nonlocal_names_are_rejected_without_writes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    setup(root);
    write(root, "gfx/tilesets/demo.shared,shared.2bpp", [255; 16 * 16]);
    write(root, "gfx/tilesets/shared.2bpp", [255; 16 * 16]);
    write(
        root,
        "gfx/tilesets/demo_palette_map.asm",
        "\ttilepal 0, GRAY\n".repeat(48),
    );
    let options = MasterGraphicsOptions {
        allow_256: true,
        ..Default::default()
    };
    let mut doc =
        MasterTilesetDocument::open_in_project(root, "demo.shared,shared", &options).unwrap();
    edit(&mut doc);
    assert!(matches!(
        doc.preview_save(&Blockset::from_bytes(&[0; 16]).unwrap()),
        Err(ProjectError::Format { .. })
    ));
    assert!(!root.join("gfx/tilesets/shared.png").exists());
    assert!(!root.join("gfx/tilesets/demo.shared,shared.png").exists());
    for name in ["C:escape", "../escape", "demo.C:escape", "demo.,../escape"] {
        assert!(matches!(
            MasterTilesetDocument::open_in_project(root, name, &options),
            Err(ProjectError::Format { .. })
        ));
    }
}

#[test]
fn preview_then_save_creates_png_preserves_import_and_assembly_and_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    setup(root);
    let source = fs::read(root.join("gfx/tilesets/demo.2bpp")).unwrap();
    let blocks = Blockset::from_bytes(&[2; 16]).unwrap();
    let mut doc =
        MasterTilesetDocument::open_in_project(root, "demo", &MasterGraphicsOptions::default())
            .unwrap();
    let before_hash = doc.content_hash(&blocks);
    edit(&mut doc);
    let preview = doc.preview_save(&blocks).unwrap();
    assert_eq!(preview.len(), 2);
    assert!(!root.join("gfx/tilesets/demo.png").exists());
    assert_eq!(doc.content_hash(&blocks), before_hash);
    doc.save(&blocks).unwrap();
    assert!(!doc.tileset().is_modified());
    assert_eq!(
        fs::read(root.join("gfx/tilesets/demo.2bpp")).unwrap(),
        source
    );
    let raw = fs::read_to_string(root.join("gfx/tilesets/demo_palette_map.asm")).unwrap();
    assert_eq!(
        raw,
        "; header\r\nTable:\r\n\ttilepal 0, WATER, RED, GREEN, TEXT ; retain\r\nFooter:\r\n\tdb 42\r\n"
    );
    let encoded = fs::read(root.join("gfx/tilesets/demo.png")).unwrap();
    // IHDR bit depth and color type match master's Image::write_image.
    assert_eq!(&encoded[24..26], &[2, 0]);
    let png = image::open(root.join("gfx/tilesets/demo.png"))
        .unwrap()
        .to_rgb8();
    assert_eq!(png.dimensions(), (128, 8));
    assert_eq!(png.get_pixel(0, 0).0, [0; 3]);
    assert_eq!(png.get_pixel(1, 0).0, [85; 3]);
    let reopened =
        MasterTilesetDocument::open_in_project(root, "demo", &MasterGraphicsOptions::default())
            .unwrap();
    assert_eq!(doc.content_hash(&blocks), reopened.content_hash(&blocks));
    assert_eq!(
        &doc.tileset().tiles()[..3],
        &reopened.tileset().tiles()[..3]
    );
    assert_eq!(&reopened.tileset().attributes()[..4], &[3, 1, 2, 7]);
    assert!(doc.preview_save(&blocks).unwrap().is_empty());
    edit(&mut doc);
    doc.save(&blocks).unwrap(); // updated source/output observations
}

#[test]
fn every_source_and_destination_conflict_prevents_all_writes() {
    for changed in ["demo.2bpp", "demo.png", "demo_palette_map.asm"] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        setup(root);
        let blocks = Blockset::from_bytes(&[2; 16]).unwrap();
        let mut doc =
            MasterTilesetDocument::open_in_project(root, "demo", &MasterGraphicsOptions::default())
                .unwrap();
        edit(&mut doc);
        let path = root.join("gfx/tilesets").join(changed);
        fs::write(&path, "external change").unwrap();
        assert!(matches!(
            doc.preview_save(&blocks),
            Err(ProjectError::Conflict(_))
        ));
        assert!(matches!(doc.save(&blocks), Err(ProjectError::Conflict(_))));
        assert!(doc.tileset().is_modified());
        assert_eq!(fs::read(path).unwrap(), b"external change");
        if changed != "demo.png" {
            assert!(!root.join("gfx/tilesets/demo.png").exists());
        }
    }
}

#[test]
fn compound_png_outputs_reopen_in_the_same_order_and_unsafe_layout_never_writes() {
    for (allow_256, before_count) in [(true, 16), (false, 128)] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            root,
            "gfx/tilesets/demo.before,after.2bpp",
            [0, 255].repeat(16 * 8),
        );
        write(
            root,
            "gfx/tilesets/before.2bpp",
            [255, 0].repeat(before_count * 8),
        );
        write(root, "gfx/tilesets/after.2bpp", [255; 16 * 16]);
        write(
            root,
            "gfx/tilesets/demo_palette_map.asm",
            "\ttilepal 0, GRAY\n".repeat(before_count + 32),
        );
        let options = MasterGraphicsOptions {
            allow_256,
            ..Default::default()
        };
        let mut doc =
            MasterTilesetDocument::open_in_project(root, "demo.before,after", &options).unwrap();
        let blocks = Blockset::from_bytes(&[0; 16]).unwrap();
        edit(&mut doc);
        if allow_256 {
            assert_eq!(doc.preview_save(&blocks).unwrap().len(), 4);
            doc.save(&blocks).unwrap();
            let reopened =
                MasterTilesetDocument::open_in_project(root, "demo.before,after", &options)
                    .unwrap();
            assert_eq!(doc.tileset().tiles(), reopened.tileset().tiles());
            assert_eq!(doc.tileset().attributes(), reopened.tileset().attributes());
            assert_eq!(doc.content_hash(&blocks), reopened.content_hash(&blocks));
        } else {
            assert!(matches!(doc.save(&blocks), Err(ProjectError::Tileset(_))));
            for name in ["demo.before,after.png", "before.png", "after.png"] {
                assert!(!root.join("gfx/tilesets").join(name).exists());
            }
        }
    }
}

#[test]
fn roof_overlay_does_not_replace_base_pixels_in_saved_png() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "gfx/tilesets/demo.2bpp", [0; 16 * 16]);
    write(root, "gfx/tilesets/roofs/test.2bpp", [255; 9 * 16]);
    write(
        root,
        "gfx/tilesets/demo_palette_map.asm",
        "\ttilepal 0, GRAY\n".repeat(16),
    );
    let options = MasterGraphicsOptions {
        roof: Some("test".into()),
        ..Default::default()
    };
    let mut doc = MasterTilesetDocument::open_in_project(root, "demo", &options).unwrap();
    let blocks = Blockset::from_bytes(&[0x0a; 16]).unwrap();
    let before = doc.graphics(&blocks).unwrap().render_block(0).unwrap();
    doc.apply_batch(&[TileCommand::Paint {
        tile: 0x0a,
        points: vec![Point { x: 0, y: 0 }],
        hue: 2,
    }])
    .unwrap();
    assert_eq!(
        doc.graphics(&blocks).unwrap().render_block(0).unwrap(),
        before
    );
    doc.save(&blocks).unwrap();
    let base =
        MasterTilesetDocument::open_in_project(root, "demo", &MasterGraphicsOptions::default())
            .unwrap();
    assert_eq!(base.tileset().tiles()[0x0a].hues()[0], 2);
    assert_eq!(
        fs::read(root.join("gfx/tilesets/roofs/test.2bpp")).unwrap(),
        [255; 9 * 16]
    );
}

#[test]
fn resizing_palette_rows_preserves_comments_but_refuses_ambiguous_assembly() {
    for ambiguous in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        setup(root);
        let raw = format!(
            "; header\n\ttilepal 0, GRAY ; first\n{}\ttilepal 0, RED, GREEN, TEXT ; second\nFooter:\n\tdb 42\n",
            if ambiguous {
                "UnrelatedLabel:\n"
            } else {
                "; between\n"
            }
        );
        write(root, "gfx/tilesets/demo_palette_map.asm", &raw);
        let mut doc =
            MasterTilesetDocument::open_in_project(root, "demo", &MasterGraphicsOptions::default())
                .unwrap();
        doc.apply_batch(&[TileCommand::Delete { tile: 2 }]).unwrap();
        let blocks = Blockset::from_bytes(&[0; 16]).unwrap();
        if ambiguous {
            assert!(matches!(
                doc.preview_save(&blocks),
                Err(ProjectError::Format { .. })
            ));
            assert_eq!(
                fs::read_to_string(root.join("gfx/tilesets/demo_palette_map.asm")).unwrap(),
                raw
            );
            assert!(!root.join("gfx/tilesets/demo.png").exists());
        } else {
            doc.save(&blocks).unwrap();
            let saved = fs::read_to_string(root.join("gfx/tilesets/demo_palette_map.asm")).unwrap();
            for retained in [
                "; header",
                "; first",
                "; between",
                "; second",
                "Footer:\n\tdb 42\n",
            ] {
                assert!(saved.contains(retained));
            }
        }
    }
}

#[test]
fn priority_updates_preserve_unrelated_asm_and_resize_owned_bank_gap() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "gfx/tilesets/demo.2bpp", [255; 192 * 16]);
    let raw = format!(
        "{}\nrept 16\n\tdb $ff\nendr\n\n{}Tail:\n\tdb 42\n",
        "\ttilepal 0, GRAY\n".repeat(96),
        "\ttilepal 1, RED\n".repeat(96)
    );
    write(root, "gfx/tilesets/demo_palette_map.asm", &raw);
    let mut doc =
        MasterTilesetDocument::open_in_project(root, "demo", &MasterGraphicsOptions::default())
            .unwrap();
    doc.set_allow_priority(true);
    doc.apply_batch(&[TileCommand::Palette {
        tile: 0,
        palette: 0x80,
    }])
    .unwrap();
    doc.save(&Blockset::from_bytes(&[0; 16]).unwrap()).unwrap();
    let saved = fs::read_to_string(root.join("gfx/tilesets/demo_palette_map.asm")).unwrap();
    assert_eq!(
        saved,
        raw.replacen(", GRAY", ", PRIORITY_GRAY", 1)
            .replacen("rept 16", "rept 32", 1)
    );
    assert!(doc.allow_priority());
}

#[test]
fn explicit_priority_option_preserves_bits_and_changes_owned_gap_even_for_pixel_only_edits() {
    for enabled in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(root, "gfx/tilesets/demo.2bpp", [255; 192 * 16]);
        let raw = format!(
            "{}\nrept 32\n\tdb $ff\nendr\n\n{}Tail:\n\tdb 42\n",
            "\ttilepal 0, PRIORITY_GRAY\n".repeat(96),
            "\ttilepal 1, RED\n".repeat(96)
        );
        write(root, "gfx/tilesets/demo_palette_map.asm", &raw);
        let mut doc =
            MasterTilesetDocument::open_in_project(root, "demo", &MasterGraphicsOptions::default())
                .unwrap();
        let blocks = Blockset::from_bytes(&[0; 16]).unwrap();
        assert!(
            doc.allow_priority(),
            "reading priority names enables the option"
        );
        let hash = doc.content_hash(&blocks);
        doc.set_allow_priority(enabled);
        assert_eq!(doc.allow_priority(), enabled);
        assert_eq!(
            doc.content_hash(&blocks) == hash,
            enabled,
            "output-mode changes invalidate preflight hashes"
        );
        assert!(!doc.tileset().is_modified());
        assert!(doc.preview_save(&blocks).unwrap().is_empty());
        doc.save(&blocks).unwrap();
        assert_eq!(
            fs::read_to_string(root.join("gfx/tilesets/demo_palette_map.asm")).unwrap(),
            raw
        );
        doc.apply_batch(&[TileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 0,
        }])
        .unwrap();
        doc.save(&blocks).unwrap();
        let saved = fs::read_to_string(root.join("gfx/tilesets/demo_palette_map.asm")).unwrap();
        assert_eq!(
            saved,
            if enabled {
                raw
            } else {
                raw.replacen("rept 32", "rept 16", 1)
            }
        );
        assert_eq!(doc.tileset().attributes()[0], 0x80);
        assert_eq!(
            doc.allow_priority(),
            enabled,
            "saving does not silently enable the option"
        );
        let reopened =
            MasterTilesetDocument::open_in_project(root, "demo", &MasterGraphicsOptions::default())
                .unwrap();
        assert!(
            reopened.allow_priority(),
            "reading priority names enables it again"
        );
    }
    let dir = tempfile::tempdir().unwrap();
    setup(dir.path());
    for enabled in [false, true] {
        let doc = MasterTilesetDocument::open_in_project(
            dir.path(),
            "demo",
            &MasterGraphicsOptions {
                allow_priority: enabled,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            doc.allow_priority(),
            enabled,
            "plain palette maps retain the session option"
        );
    }
}
