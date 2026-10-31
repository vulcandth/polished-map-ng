use polished_map_core::{CollisionRow, plusplus::AttributeCommand};
use polished_map_project::{
    ProjectError,
    plusplus::{BlocksetDocument, prepare_binary_save_as},
    prepare_collision_save_as,
};
use std::fs;

#[test]
fn expanded_codecs_keep_master_limits_and_bound_allocation() {
    use polished_map_project::{PngPlatform, codecs, plusplus::read_tiles};
    let bytes = vec![0xaa; 8192];
    assert!(codecs::decode_2bpp(&bytes).is_err());
    let tiles = codecs::decode_plusplus_2bpp(&bytes).unwrap();
    assert_eq!(tiles.len(), 512);
    assert_eq!(codecs::encode_2bpp(&tiles), bytes);
    assert!(codecs::decode_plusplus_2bpp(&[0; 8193]).is_err());
    assert!(codecs::decode_plusplus_2bpp(&[0; 17]).is_err());
    let padded = codecs::decode_plusplus_lz_tiles(&[0, 255, 255]).unwrap();
    assert_eq!(padded.len(), 512);
    assert_eq!(&padded[0].hues()[..8], &[2; 8]);
    assert_eq!(*padded[511].hues(), [0; 64]);
    // Eight long blank runs exactly fill 8192 bytes; a ninth exceeds the cap.
    let mut lz = [0xef, 0xff].repeat(8);
    lz.push(255);
    assert_eq!(codecs::decode_plusplus_lz_tiles(&lz).unwrap().len(), 512);
    lz.pop();
    lz.extend_from_slice(&[0xef, 0xff, 255]);
    assert!(codecs::decode_plusplus_lz_tiles(&lz).is_err());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sheet.png");
    let raster = polished_map_core::RgbImage {
        width: 128,
        height: 256,
        pixels: vec![170; 128 * 256 * 3],
    };
    polished_map_project::write_png(&path, &raster).unwrap();
    let expanded = read_tiles(&path, PngPlatform::Unix).unwrap();
    assert_eq!(expanded.len(), 512);
    assert_eq!(*expanded[511].hues(), [2; 64]);
    for (width, height) in [(4096, 8), (8, 4096)] {
        let raster = polished_map_core::RgbImage {
            width,
            height,
            pixels: vec![255; width as usize * height as usize * 3],
        };
        polished_map_project::write_png(&path, &raster).unwrap();
        assert_eq!(read_tiles(&path, PngPlatform::Unix).unwrap().len(), 512);
    }
    let wrong = dir.path().join("sheet.chr");
    fs::write(&wrong, b"\tdb 0\n").unwrap();
    assert!(read_tiles(&wrong, PngPlatform::Unix).is_err());
    let raster = polished_map_core::RgbImage {
        width: 128,
        height: 264,
        pixels: vec![0; 128 * 264 * 3],
    };
    polished_map_project::write_png(&path, &raster).unwrap();
    assert!(read_tiles(&path, PngPlatform::Unix).is_err());
}

#[test]
fn save_reopens_attributes_offsets_and_preserves_unrelated_asm() {
    let dir = tempfile::tempdir().unwrap();
    let offsets = dir.path().join("offsets.bin");
    let attrs = dir.path().join("attrs.bin");
    let coll = dir.path().join("coll.asm");
    fs::write(&offsets, [0; 32]).unwrap();
    fs::write(&attrs, [0; 16]).unwrap();
    let original =
        "; header\r\n\ttilecoll A, B, C, D ; keep\r\n\ttilecoll E, F, G, H\r\nOther: db 7\r\n";
    fs::write(&coll, original).unwrap();
    let mut doc = BlocksetDocument::open(&offsets, &attrs, Some(&coll)).unwrap();
    doc.blockset_mut()
        .apply(&AttributeCommand::Set {
            block: 1,
            offsets: [0x80; 16],
            attributes: [0xff; 16],
        })
        .unwrap();
    doc.blockset_mut()
        .apply(&AttributeCommand::SetCollisions {
            block: 1,
            collision: CollisionRow::Asm(["I".into(), "J".into(), "K".into(), "L".into()]),
        })
        .unwrap();
    doc.save().unwrap();
    let saved = BlocksetDocument::open(&offsets, &attrs, Some(&coll)).unwrap();
    assert_eq!(saved.blockset().offsets().blocks()[1], [0x80; 16]);
    assert_eq!(saved.blockset().attributes()[1], [0xef; 16]);
    assert_eq!(
        fs::read_to_string(&coll).unwrap(),
        original.replace("E, F, G, H", "I, J, K, L")
    );
    assert!(!doc.blockset().is_modified());
    let o = prepare_binary_save_as(&dir.path().join("copy-offsets")).unwrap();
    let a = prepare_binary_save_as(&dir.path().join("copy-attrs")).unwrap();
    let c = prepare_collision_save_as(&dir.path().join("copy-coll"), false).unwrap();
    doc.save_as(&o, &a, Some(&c)).unwrap();
    assert_eq!(fs::read(o.path()).unwrap(), fs::read(offsets).unwrap());
    assert_eq!(fs::read(a.path()).unwrap(), fs::read(attrs).unwrap());
    assert_eq!(fs::read(c.path()).unwrap(), fs::read(coll).unwrap());
}

#[test]
fn conflict_in_any_domain_or_duplicate_destination_never_partially_saves() {
    for conflict in 0..3 {
        let dir = tempfile::tempdir().unwrap();
        let paths = ["offsets.bin", "attrs.bin", "coll.bin"].map(|p| dir.path().join(p));
        fs::write(&paths[0], [0; 16]).unwrap();
        fs::write(&paths[1], [0; 16]).unwrap();
        fs::write(&paths[2], [0; 4]).unwrap();
        let mut doc = BlocksetDocument::open(&paths[0], &paths[1], Some(&paths[2])).unwrap();
        doc.blockset_mut()
            .apply(&AttributeCommand::Set {
                block: 0,
                offsets: [1; 16],
                attributes: [0xa5; 16],
            })
            .unwrap();
        fs::write(&paths[conflict], b"external writer").unwrap();
        let before = paths.each_ref().map(|p| fs::read(p).unwrap());
        assert!(matches!(doc.save(), Err(ProjectError::Conflict(_))));
        assert!(doc.blockset().is_modified());
        assert_eq!(paths.each_ref().map(|p| fs::read(p).unwrap()), before);
        let target = prepare_binary_save_as(&dir.path().join("same.bin")).unwrap();
        let collision = prepare_collision_save_as(&dir.path().join("new-coll.bin"), true).unwrap();
        assert!(doc.save_as(&target, &target, Some(&collision)).is_err());
        assert!(!target.path().exists());
        assert!(!collision.path().exists());
    }
}

#[test]
fn discovery_uses_full_name_before_base_and_ignores_master_only_paths() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("data/tilesets")).unwrap();
    fs::create_dir_all(root.join("tilesets")).unwrap();
    for (name, value) in [("data/tilesets/demo", 1), ("tilesets/demo.extra", 2)] {
        fs::write(root.join(format!("{name}_metatiles.bin")), [value; 16]).unwrap();
        fs::write(root.join(format!("{name}_attributes.bin")), [value; 16]).unwrap();
    }
    fs::write(root.join("data/tilesets/demo.extra_collision.bin"), [8; 4]).unwrap();
    let doc = BlocksetDocument::open_in_project(root, "demo.extra").unwrap();
    assert_eq!(doc.blockset().offsets().blocks()[0], [2; 16]);
    assert_eq!(doc.blockset().attributes()[0], [2; 16]);
    assert!(doc.collision_path().is_none());
    assert!(BlocksetDocument::open_in_project(root, "../demo").is_err());
}

#[test]
#[ignore = "requires independently fetched pinned Polished Crystal project"]
fn real_polished_crystal_blocksets_roundtrip_in_temporary_copies() {
    let root = std::path::PathBuf::from(
        std::env::var_os("POLISHED_CRYSTAL_FIXTURE").expect("set pinned fixture path"),
    );
    let dir = tempfile::tempdir().unwrap();
    let mut count = 0;
    for entry in fs::read_dir(root.join("data/tilesets")).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_suffix("_metatiles.bin"))
        else {
            continue;
        };
        let original = BlocksetDocument::open_in_project(&root, name).unwrap();
        let o = dir.path().join(format!("{name}_metatiles.bin"));
        let a = dir.path().join(format!("{name}_attributes.bin"));
        fs::copy(original.path(), &o).unwrap();
        fs::copy(original.attribute_path(), &a).unwrap();
        let c = original.collision_path().map(|p| {
            let dest = dir.path().join(p.file_name().unwrap());
            fs::copy(p, &dest).unwrap();
            dest
        });
        let mut doc = BlocksetDocument::open(&o, &a, c.as_deref()).unwrap();
        let offsets = fs::read(&o).unwrap();
        let attrs = fs::read(&a).unwrap();
        let collisions = c.as_ref().map(|p| fs::read(p).unwrap());
        doc.blockset_mut()
            .apply(&AttributeCommand::Swap { a: 0, b: 1 })
            .unwrap();
        doc.save().unwrap();
        let mut doc = BlocksetDocument::open(&o, &a, c.as_deref()).unwrap();
        assert_eq!(
            doc.blockset().offsets().blocks()[0],
            original.blockset().offsets().blocks()[1]
        );
        assert_eq!(
            doc.blockset().attributes()[0],
            original.blockset().attributes()[1]
        );
        doc.blockset_mut()
            .apply(&AttributeCommand::Swap { a: 0, b: 1 })
            .unwrap();
        doc.save().unwrap();
        assert_eq!(fs::read(&o).unwrap(), offsets);
        assert_eq!(fs::read(&a).unwrap(), attrs);
        assert_eq!(c.as_ref().map(|p| fs::read(p).unwrap()), collisions);
        count += 1;
    }
    assert_eq!(count, 51, "fixture must match upstream.lock.toml");
}
