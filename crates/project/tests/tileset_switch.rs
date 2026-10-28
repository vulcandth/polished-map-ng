use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet, Point, TileCommand};
use polished_map_project::{MasterGraphicsOptions, MasterTilesetDocument, tileset_choices};
use std::fs;

#[test]
fn replacement_reloads_assets_and_keeps_session_palettes_roof_and_modes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for folder in ["gfx/tilesets/roofs", "data/tilesets", "constants"] {
        fs::create_dir_all(root.join(folder)).unwrap();
    }
    for (name, hue, blocks) in [("demo", 0, 2), ("01", 255, 1)] {
        fs::write(
            root.join(format!("gfx/tilesets/{name}.2bpp")),
            [hue; 32 * 16],
        )
        .unwrap();
        fs::write(
            root.join(format!("gfx/tilesets/{name}_palette_map.asm")),
            format!(
                "\ttilepal 0, {}\n",
                if name == "demo" {
                    "PRIORITY_RED"
                } else {
                    "RED"
                }
            )
            .repeat(32),
        )
        .unwrap();
        fs::write(
            root.join(format!("data/tilesets/{name}_metatiles.bin")),
            vec![0x0a; blocks * 16],
        )
        .unwrap();
    }
    fs::write(
        root.join("constants/tileset_constants.asm"),
        "\tconst TILESET_First\n",
    )
    .unwrap();
    fs::write(
        root.join("data/tilesets/01_collision.asm"),
        "\ttilecoll WALL, FLOOR, WATER, GRASS\n",
    )
    .unwrap();
    fs::write(root.join("gfx/tilesets/01.colored.png"), b"ignored").unwrap();
    fs::write(root.join("gfx/tilesets/roofs/test.2bpp"), [0; 9 * 16]).unwrap();
    let choices = tileset_choices(root).unwrap();
    assert_eq!((&*choices[0].name, &*choices[0].label), ("01", "01: First"));
    assert_eq!(choices.len(), 2);
    let mut doc = MasterTilesetDocument::open_in_project(
        root,
        "demo",
        &MasterGraphicsOptions {
            allow_256: true,
            roof: Some("test".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let paint = TileCommand::Paint {
        tile: 0,
        points: vec![Point { x: 0, y: 0 }],
        hue: 2,
    };
    doc.apply_batch(std::slice::from_ref(&paint)).unwrap();
    doc.roof_mut()
        .unwrap()
        .roof_mut()
        .apply_batch(&[paint])
        .unwrap();
    doc.palettes_mut()
        .apply_batch(&[PaletteCommand::SetRgb {
            slot: ColorSlot {
                set: PaletteSet::Nite,
                palette: 1,
                hue: 0,
            },
            rgb: [17, 34, 51],
        }])
        .unwrap();
    doc.palettes_mut().select(PaletteSet::Nite);
    let before = doc.palettes().content_hash();
    let (new, blocks) = doc.prepare_replacement("01").unwrap();
    assert_eq!(new.name(), "01");
    assert_eq!(new.tileset().tiles()[0].hues(), &[3; 64]);
    assert_eq!(blocks.blockset().blocks().len(), 1);
    assert_eq!(
        blocks.blockset().collisions().unwrap()[0],
        polished_map_core::CollisionRow::Asm(
            ["WALL", "FLOOR", "WATER", "GRASS"].map(str::to_owned)
        )
    );
    assert!(!blocks.blockset().is_modified());
    assert!(!new.tileset().is_modified());
    assert!(new.allow_256());
    assert!(
        new.allow_priority(),
        "priority mode remains enabled across replacement"
    );
    assert_eq!(new.palettes().content_hash(), before);
    assert!(new.palettes().is_modified());
    assert_eq!(new.roof().unwrap().name(), "test");
    assert_eq!(
        new.roof().unwrap().roof().tileset().tiles()[0].hues(),
        &[0; 64]
    );
    assert!(!new.roof().unwrap().roof().tileset().is_modified());
    assert_eq!(
        &new.graphics(blocks.blockset())
            .unwrap()
            .render_block(0)
            .unwrap()
            .pixels[..3],
        &[17, 34, 51]
    );
    // Successful preparation and malformed/missing assets never mutate the source session.
    fs::write(root.join("data/tilesets/01_metatiles.bin"), [0; 15]).unwrap();
    assert!(doc.prepare_replacement("01").is_err());
    assert!(doc.prepare_replacement("missing").is_err());
    assert_eq!(doc.name(), "demo");
    assert_eq!(doc.tileset().tiles()[0].hues()[0], 2);
    assert_eq!(doc.roof().unwrap().roof().tileset().tiles()[0].hues()[0], 2);
    assert!(doc.tileset().is_modified());
    assert_eq!(doc.palettes().content_hash(), before);
    fs::write(root.join("data/tilesets/01_metatiles.bin"), [0x0a; 16]).unwrap();
    fs::remove_file(root.join("gfx/tilesets/01_palette_map.asm")).unwrap();
    let (mono, _) = doc.prepare_replacement("01").unwrap();
    assert!(mono.monochrome());
    let (still_mono, _) = mono.prepare_replacement("demo").unwrap();
    assert!(
        still_mono.monochrome(),
        "upstream monochrome setting persists even when the next tileset has a palette map"
    );
}
