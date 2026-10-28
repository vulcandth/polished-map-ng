use polished_map_core::{
    Blockset, Point, Tile, TileCommand, TileSnapshot, Tileset, TilesetError, day_palettes,
};

fn sheet(n: usize) -> Tileset {
    Tileset::new(vec![Tile::new([0; 64]).unwrap(); n], vec![0; n]).unwrap()
}

#[test]
fn tile_snapshots_include_priority_and_swap_uses_live_values() {
    let mut s = sheet(3);
    s.apply(&TileCommand::Set {
        tile: 0,
        value: TileSnapshot {
            pixels: vec![2; 64],
            palette: 0x85,
        },
    })
    .unwrap();
    let copied = s.snapshot(0).unwrap();
    s.apply(&TileCommand::Delete { tile: 0 }).unwrap();
    s.apply(&TileCommand::Set {
        tile: 1,
        value: copied,
    })
    .unwrap();
    s.apply(&TileCommand::Swap { a: 0, b: 1 }).unwrap();
    assert_eq!(s.attributes(), [0x85, 9, 0]);
    assert_eq!(s.tiles()[0].hues(), &[2; 64]);
    assert_eq!(s.tiles()[1].hues(), &[0; 64]);
    s.apply(&TileCommand::Copy { from: 0, to: 2 }).unwrap();
    assert_eq!(s.snapshot(0).unwrap(), s.snapshot(2).unwrap());
    assert_eq!(
        s.render_tile(0, &day_palettes()).unwrap().pixels,
        day_palettes()[5][2].repeat(64)
    );
    s.mark_saved();
    s.apply(&TileCommand::Palette {
        tile: 0,
        palette: 0x85,
    })
    .unwrap();
    assert!(s.is_modified());
}

#[test]
fn invalid_commands_and_batches_never_modify_their_valid_prefix() {
    let mut s = sheet(1);
    let before = s.clone();
    for command in [
        TileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }, Point { x: 8, y: 0 }],
            hue: 2,
        },
        TileCommand::Fill {
            tile: 0,
            at: Point { x: 0, y: 0 },
            hue: 4,
        },
        TileCommand::Copy { from: 0, to: 1 },
        TileCommand::Set {
            tile: 0,
            value: TileSnapshot {
                pixels: vec![0; 63],
                palette: 0,
            },
        },
        TileCommand::Palette {
            tile: 0,
            palette: 0x89,
        },
        TileCommand::ImportRgb {
            tile: 0,
            width: 9,
            height: 8,
            pixels: vec![0; 9 * 8 * 3],
            allow_256: true,
        },
        TileCommand::ImportRgb {
            tile: 0,
            width: 8,
            height: 8,
            pixels: vec![0; 2],
            allow_256: true,
        },
    ] {
        assert!(s.apply(&command).is_err());
        assert_eq!(s, before);
        assert!(
            s.apply_batch(&[TileCommand::Delete { tile: 0 }, command])
                .is_err()
        );
        assert_eq!(s, before);
    }
    assert_eq!(Tileset::new(vec![], vec![]), Err(TilesetError::Length));
}

#[test]
fn fill_is_four_connected_replace_is_global_and_swap_is_simultaneous() {
    let pixels = std::array::from_fn(|i| if (i / 8 + i % 8) % 2 == 0 { 1 } else { 0 });
    let mut s = Tileset::new(vec![Tile::new(pixels).unwrap()], vec![0]).unwrap();
    s.apply(&TileCommand::Fill {
        tile: 0,
        at: Point { x: 0, y: 0 },
        hue: 2,
    })
    .unwrap();
    assert_eq!(s.tiles()[0].hues().iter().filter(|&&h| h == 2).count(), 1);
    s.apply(&TileCommand::Replace {
        tile: 0,
        from: 1,
        to: 3,
    })
    .unwrap();
    assert_eq!(s.tiles()[0].hues().iter().filter(|&&h| h == 3).count(), 31);
    s.apply(&TileCommand::SwapHues {
        tile: 0,
        a: 2,
        b: 3,
    })
    .unwrap();
    assert_eq!(s.tiles()[0].hues()[0], 3);
    assert_eq!(s.tiles()[0].hues().iter().filter(|&&h| h == 2).count(), 31);
}

#[test]
fn rgb_paste_skips_reserved_rows_clips_and_defines_only_touched_tiles() {
    let mut s = Tileset::new(vec![Tile::new([0; 64]).unwrap(); 256], vec![9; 256]).unwrap();
    s.apply(&TileCommand::ImportRgb {
        tile: 0x5f,
        width: 16,
        height: 16,
        pixels: vec![0; 16 * 16 * 3],
        allow_256: false,
    })
    .unwrap();
    assert_eq!(s.tiles()[0x5f].hues(), &[3; 64]);
    assert_eq!(s.tiles()[0x8f].hues(), &[3; 64]);
    assert_eq!(s.tiles()[0x6f].hues(), &[0; 64]);
    assert_eq!(s.attributes().iter().filter(|&&p| p == 0).count(), 2);
    s.apply(&TileCommand::ImportRgb {
        tile: 255,
        width: 16,
        height: 16,
        pixels: [0, 255, 0].repeat(16 * 16),
        allow_256: true,
    })
    .unwrap();
    assert_eq!(s.tiles()[255].hues(), &[2; 64]); // weighted gray 155 -> LIGHT
}

#[test]
fn trim_retains_used_blank_tiles_and_never_changes_pixels_or_dirty_state() {
    let mut s = sheet(256);
    s.apply(&TileCommand::Paint {
        tile: 7,
        points: vec![Point { x: 0, y: 0 }],
        hue: 1,
    })
    .unwrap();
    s.mark_saved();
    let before = s.tiles().to_vec();
    s.trim_unused(&Blockset::from_bytes(&[12; 16]).unwrap());
    assert_eq!(&s.attributes()[..13], &[0; 13]);
    assert_eq!(&s.attributes()[13..], &[9; 243]);
    assert_eq!(s.tiles(), before);
    assert!(!s.is_modified());
    s.trim_unused(&Blockset::from_bytes(&[0; 16]).unwrap());
    assert_eq!(&s.attributes()[..8], &[0; 8]);
    assert_eq!(&s.attributes()[8..], &[9; 248]);
}

#[test]
fn palette_serialization_preserves_banks_reserved_rows_and_partial_line_quirks() {
    let s = Tileset::new(vec![Tile::new([0; 64]).unwrap(); 3], vec![0x80, 2, 9]).unwrap();
    assert_eq!(
        s.palette_map_asm(true, false).unwrap(),
        "\ttilepal 0, PRIORITY_GRAY, GREEN\n"
    );
    assert_eq!(
        s.palette_map_asm(true, true).unwrap(),
        "\ttilepal 0, PRIORITY_GRAY, GREEN"
    );
    let s = Tileset::new(vec![Tile::new([0; 64]).unwrap(); 3], vec![0, 9, 3]).unwrap();
    assert_eq!(
        s.palette_map_asm(true, false).unwrap(),
        "\ttilepal 0, GRAY, TEXT, WATER, TEXT\n"
    );
    let mut s = sheet(256);
    for (priority, repeats) in [(false, 16), (true, 32)] {
        let text = s.palette_map_asm(false, priority).unwrap();
        assert_eq!(text.matches(&format!("rept {repeats}\n")).count(), 2);
        assert_eq!(text.matches(", GRAY").count(), 192);
        assert_eq!(text.matches("tilepal 0").count(), 12);
        assert_eq!(text.matches("tilepal 1").count(), 12);
    }
    for tile in 0..=255 {
        s.apply(&TileCommand::Delete { tile }).unwrap();
    }
    assert_eq!(
        s.palette_map_asm(true, false),
        Err(TilesetError::ExportLayout)
    );
}

#[test]
fn grayscale_export_keeps_hue_order_padding_gaps_and_rejects_unsafe_offsets() {
    let tiles = (0..256)
        .map(|i| Tile::new([(i % 4) as u8; 64]).unwrap())
        .collect();
    let s = Tileset::new(tiles, vec![0; 256]).unwrap();
    let image = s.export_region(0, Some(2), true).unwrap();
    assert_eq!((image.width, image.height), (128, 8));
    assert_eq!(&image.pixels[8 * 3..9 * 3], &[85; 3]);
    assert_eq!(&image.pixels[16 * 3..17 * 3], &[255; 3]);
    let image = s.export_region(0, Some(224), false).unwrap();
    assert_eq!((image.width, image.height), (128, 96));
    assert_eq!(image.pixels.len(), 128 * 96 * 3);
    assert_eq!(
        &image.pixels[(48 * 128 + 8) * 3..(48 * 128 + 9) * 3],
        &[85; 3]
    );
    assert_eq!(
        s.export_region(128, Some(16), false),
        Err(TilesetError::ExportLayout)
    );
    assert_eq!(
        s.export_region(0, Some(97), false),
        Err(TilesetError::ExportLayout)
    );
    assert_eq!(
        s.export_region(usize::MAX, None, false),
        Err(TilesetError::ExportLayout)
    );
    assert_eq!(
        s.export_region(0, Some(usize::MAX), true),
        Err(TilesetError::ExportLayout)
    );
    assert_eq!(
        s.export_region(0, Some(0), true),
        Err(TilesetError::ExportLayout)
    );
}
