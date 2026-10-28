use polished_map_core::{Blockset, Point, TileCommand};
use polished_map_project::{
    MasterGraphicsOptions, MasterTilesetDocument, ProjectError, RoofDocument,
};
use std::{fs, path::Path};

fn fixture(root: &Path) {
    fs::create_dir_all(root.join("gfx/tilesets/roofs")).unwrap();
    fs::write(root.join("gfx/tilesets/demo.2bpp"), [0; 32 * 16]).unwrap();
    fs::write(
        root.join("gfx/tilesets/demo_palette_map.asm"),
        "\ttilepal 0, GRAY\n".repeat(32),
    )
    .unwrap();
    fs::write(root.join("gfx/tilesets/roofs/test.2bpp"), [255; 9 * 16]).unwrap();
}
fn options() -> MasterGraphicsOptions {
    MasterGraphicsOptions {
        roof: Some("test".into()),
        ..Default::default()
    }
}
fn paint(tile: u8, hue: u8) -> TileCommand {
    TileCommand::Paint {
        tile,
        points: vec![Point { x: 0, y: 0 }],
        hue,
    }
}

#[test]
fn roof_save_preserves_base_files_and_dirty_state_and_reopens_in_file_order() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fixture(root);
    let mut doc = MasterTilesetDocument::open_in_project(root, "demo", &options()).unwrap();
    let palette_before = fs::read(root.join("gfx/tilesets/demo_palette_map.asm")).unwrap();
    doc.apply_batch(&[paint(0, 2)]).unwrap();
    let roof = doc.roof_mut().unwrap();
    assert!(!roof.roof().tileset().is_modified());
    roof.roof_mut().apply_batch(&[paint(6, 0)]).unwrap();
    let before_hash = roof.content_hash();
    let preview = roof.preview_save().unwrap().unwrap();
    assert!(preview.previous_hash.is_none());
    assert!(!root.join("gfx/tilesets/roofs/test.png").exists());
    roof.save().unwrap();
    assert_ne!(roof.content_hash(), before_hash);
    let reopened = RoofDocument::open_in_project(root, "test", [0; 9]).unwrap();
    assert_eq!(reopened.content_hash(), roof.content_hash());
    assert_eq!(reopened.roof().tileset().tiles()[6].hues()[0], 0);
    assert_eq!(reopened.roof().tileset().tiles()[0].hues()[0], 3);
    let image = image::open(root.join("gfx/tilesets/roofs/test.png"))
        .unwrap()
        .to_rgb8();
    assert_eq!(image.dimensions(), (24, 24));
    assert_eq!(image.get_pixel(0, 16).0, [255; 3]); // GUI top-left, file bottom-left
    assert_eq!(
        &fs::read(root.join("gfx/tilesets/roofs/test.png")).unwrap()[24..26],
        &[2, 0]
    );
    assert!(doc.tileset().is_modified());
    assert!(!doc.roof().unwrap().roof().tileset().is_modified());
    assert!(!root.join("gfx/tilesets/demo.png").exists());
    assert_eq!(
        fs::read(root.join("gfx/tilesets/demo_palette_map.asm")).unwrap(),
        palette_before
    );
    assert_eq!(
        fs::read(root.join("gfx/tilesets/roofs/test.2bpp")).unwrap(),
        [255; 9 * 16]
    );
    // Saving base graphics leaves a newly dirty roof untouched.
    doc.roof_mut()
        .unwrap()
        .roof_mut()
        .apply_batch(&[paint(0, 1)])
        .unwrap();
    doc.save(&Blockset::from_bytes(&[0; 16]).unwrap()).unwrap();
    assert!(doc.roof().unwrap().roof().tileset().is_modified());
    assert_eq!(
        RoofDocument::open_in_project(root, "test", [0; 9])
            .unwrap()
            .roof()
            .tileset()
            .tiles()[0]
            .hues()[0],
        3
    );
    assert_eq!(
        doc.graphics(&Blockset::from_bytes(&[0x10; 16]).unwrap())
            .unwrap()
            .render_block(0)
            .unwrap()
            .pixels[..3],
        polished_map_core::day_palettes()[0][0]
    );
}

#[test]
fn roof_sources_and_outputs_are_conflict_checked_independently() {
    for changed in ["test.2bpp", "test.png"] {
        let dir = tempfile::tempdir().unwrap();
        fixture(dir.path());
        let mut doc = RoofDocument::open_in_project(dir.path(), "test", [0; 9]).unwrap();
        doc.roof_mut().apply_batch(&[paint(6, 0)]).unwrap();
        let path = dir.path().join("gfx/tilesets/roofs").join(changed);
        fs::write(&path, b"external").unwrap();
        assert!(matches!(doc.preview_save(), Err(ProjectError::Conflict(_))));
        assert!(matches!(doc.save(), Err(ProjectError::Conflict(_))));
        assert!(doc.roof().tileset().is_modified());
        assert_eq!(fs::read(path).unwrap(), b"external");
    }
}
