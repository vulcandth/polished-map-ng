use polished_map_core::{Point, Roof, Tile, TileCommand, TilesetError};

fn roof() -> Roof {
    Roof::new(
        std::array::from_fn(|i| Tile::new([i as u8 % 4; 64]).unwrap()),
        [0, 1, 2, 3, 4, 5, 6, 7, 9],
    )
    .unwrap()
}

#[test]
fn roof_commands_are_atomic_and_keep_palettes_with_copied_pixels() {
    let mut roof = roof();
    let original = roof.tileset().tiles().to_vec();
    assert!(matches!(
        roof.apply_batch(&[
            TileCommand::Copy { from: 1, to: 0 },
            TileCommand::Paint {
                tile: 9,
                points: vec![Point { x: 0, y: 0 }],
                hue: 0
            }
        ]),
        Err(TilesetError::Tile(9))
    ));
    assert_eq!(roof.tileset().tiles(), original);
    assert!(!roof.tileset().is_modified());
    for command in [
        TileCommand::Delete { tile: 0 },
        TileCommand::Palette {
            tile: 0,
            palette: 2,
        },
    ] {
        assert_eq!(roof.apply_batch(&[command]), Err(TilesetError::RoofCommand));
    }
    roof.apply_batch(&[
        TileCommand::Copy { from: 1, to: 0 },
        TileCommand::Swap { a: 0, b: 2 },
    ])
    .unwrap();
    assert_eq!(&roof.tileset().attributes()[..3], &[2, 1, 1]);
    assert_eq!(roof.tileset().tiles()[0].hues(), &[2; 64]);
    roof.mark_saved();
    roof.synchronize_attributes([3; 9]).unwrap();
    assert!(!roof.tileset().is_modified());
    assert_eq!(roof.tileset().attributes(), [3; 9]);
}

#[test]
fn roof_export_is_file_order_not_the_reversed_display_rows() {
    let image = roof().export();
    assert_eq!((image.width, image.height), (24, 24));
    for id in 0..9 {
        for y in 0..8 {
            for x in 0..8 {
                let p = ((id / 3 * 8 + y) * 24 + id % 3 * 8 + x) * 3;
                assert_eq!(&image.pixels[p..p + 3], &[[255, 85, 170, 0][id % 4]; 3]);
            }
        }
    }
}

#[test]
fn roof_image_paste_clips_to_one_tile_and_preserves_uncovered_pixels() {
    for (width, height) in [(1, 2), (8, 8), (9, 12)] {
        let mut roof = roof();
        roof.apply_batch(&[TileCommand::ImportRgb {
            tile: 7,
            width,
            height,
            pixels: vec![255; usize::from(width) * usize::from(height) * 3],
            allow_256: false,
        }])
        .unwrap();
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(
                    roof.tileset().tiles()[7].hues()[y * 8 + x],
                    if x < usize::from(width) && y < usize::from(height) {
                        0
                    } else {
                        3
                    }
                );
            }
        }
        assert_eq!(roof.tileset().tiles()[8].hues(), &[0; 64]);
        let before = roof.tileset().tiles().to_vec();
        assert!(
            roof.apply_batch(&[TileCommand::ImportRgb {
                tile: 0,
                width: 2,
                height: 2,
                pixels: vec![0; 3],
                allow_256: false
            }])
            .is_err()
        );
        assert_eq!(roof.tileset().tiles(), before);
    }
}
