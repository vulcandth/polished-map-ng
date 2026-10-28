//! Optional differential run; requires the pinned C++ oracle built by tools/reference_oracle.py.
use polished_map_core::{PaletteBank, PaletteSet};
use polished_map_project::codecs::decode_2bpp;
use std::{fs, process::Command};

#[test]
#[ignore = "requires POLISHED_MAP_TILE_ORACLE; run in the reference CI job"]
fn tile_pixels_and_rgb_paste_match_compiled_upstream() {
    use polished_map_core::{Point, Tile, TileCommand, Tileset};
    let oracle =
        std::env::var_os("POLISHED_MAP_TILE_ORACLE").expect("set POLISHED_MAP_TILE_ORACLE");
    let dir = tempfile::tempdir().unwrap();
    let edits = dir.path().join("edits.bin");
    let images = dir.path().join("images.bin");
    let roofs = dir.path().join("roofs.bin");
    assert!(
        Command::new(oracle)
            .arg(&edits)
            .arg(&images)
            .arg(&roofs)
            .status()
            .unwrap()
            .success()
    );
    let reference = fs::read(edits).unwrap();
    assert_eq!(reference.len(), 40 * 3 * 64 * 4 * 64);
    let mut offset = 0;
    for seed in 0..40 {
        let pixels = std::array::from_fn(|i| {
            (if seed < 4 {
                seed
            } else if seed < 8 {
                (i / 8 + i % 8 + seed) % 4
            } else {
                (i * seed + i / 8 + (i % 8) * (i % 8)) % 4
            }) as u8
        });
        for op in 0..3 {
            for at in 0..64 {
                for to in 0..4 {
                    let mut sheet =
                        Tileset::new(vec![Tile::new(pixels).unwrap()], vec![0]).unwrap();
                    let command = match op {
                        0 => TileCommand::Fill {
                            tile: 0,
                            at: Point {
                                x: (at % 8) as u16,
                                y: (at / 8) as u16,
                            },
                            hue: to,
                        },
                        1 => TileCommand::Replace {
                            tile: 0,
                            from: pixels[at],
                            to,
                        },
                        _ => TileCommand::SwapHues {
                            tile: 0,
                            a: pixels[at],
                            b: to,
                        },
                    };
                    sheet.apply(&command).unwrap();
                    assert_eq!(
                        sheet.tiles()[0].hues(),
                        &reference[offset..offset + 64],
                        "seed={seed} op={op} at={at} to={to}"
                    );
                    offset += 64;
                }
            }
        }
    }
    let reference = fs::read(images).unwrap();
    assert_eq!(reference.len(), 2 * 11 * 4 * 256 * 65);
    let mut offset = 0;
    for allow_256 in [false, true] {
        for tile in [0, 15, 0x50, 0x5f, 0x60, 0x7f, 0x80, 0xd0, 0xdf, 0xe0, 0xff] {
            for (width, height) in [(1, 1), (9, 9), (128, 128), (200, 120)] {
                let pixels = (0..usize::from(width) * usize::from(height) * 3)
                    .map(|i| ((i * 37 + i / 11) & 255) as u8)
                    .collect();
                let mut sheet =
                    Tileset::new(vec![Tile::new([0; 64]).unwrap(); 256], vec![9; 256]).unwrap();
                sheet
                    .apply(&TileCommand::ImportRgb {
                        tile,
                        width,
                        height,
                        pixels,
                        allow_256,
                    })
                    .unwrap();
                for i in 0..256 {
                    assert_eq!(
                        sheet.tiles()[i].hues(),
                        &reference[offset..offset + 64],
                        "allow={allow_256} tile={tile} size={width}x{height} result={i}"
                    );
                    assert_eq!(sheet.attributes()[i], reference[offset + 64]);
                    offset += 65;
                }
            }
        }
    }
    let reference = fs::read(roofs).unwrap();
    assert_eq!(reference.len(), 16 * 9 * 2 * 9 * 65);
    let mut offset = 0;
    for seed in 0..16 {
        for tile in 0..9 {
            for palette in [0, 9] {
                let mut attributes = [9; 9];
                attributes[usize::from(tile)] = palette;
                let mut roof = polished_map_core::Roof::new(
                    std::array::from_fn(|_| Tile::new([0; 64]).unwrap()),
                    attributes,
                )
                .unwrap();
                roof.apply_batch(&[TileCommand::ImportRgb {
                    tile,
                    width: 8,
                    height: 8,
                    pixels: (0..8 * 8 * 3)
                        .map(|i| ((i * 37 + i / 11 + seed * 19) & 255) as u8)
                        .collect(),
                    allow_256: false,
                }])
                .unwrap();
                for i in 0..9 {
                    assert_eq!(
                        roof.tileset().tiles()[i].hues(),
                        &reference[offset..offset + 64],
                        "roof seed={seed} tile={tile} palette={palette} result={i}"
                    );
                    assert_eq!(roof.tileset().attributes()[i], reference[offset + 64]);
                    offset += 65;
                }
            }
        }
    }
}

#[test]
#[ignore = "requires POLISHED_MAP_BLOCK_ORACLE; run in the reference CI job"]
fn every_block_brush_case_matches_compiled_upstream_handler() {
    use polished_map_core::{BlockBrush, Point};
    let oracle =
        std::env::var_os("POLISHED_MAP_BLOCK_ORACLE").expect("set POLISHED_MAP_BLOCK_ORACLE");
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("brush.bin");
    assert!(
        Command::new(oracle)
            .arg(&output)
            .status()
            .unwrap()
            .success()
    );
    let reference = fs::read(output).unwrap();
    assert_eq!(reference.len(), 2 * 8 * 256 * 16 * 16);
    let mut offset = 0;
    for allow in [false, true] {
        for modifiers in 0..8 {
            let brush = BlockBrush {
                shift: modifiers & 1 != 0,
                ctrl: modifiers & 2 != 0,
                alt: modifiers & 4 != 0,
            };
            for tile in 0..=255 {
                for at in 0..16 {
                    let mut draft = [7; 16];
                    brush
                        .paint(
                            &mut draft,
                            Point {
                                x: at % 4,
                                y: at / 4,
                            },
                            tile,
                            allow,
                        )
                        .unwrap();
                    assert_eq!(
                        &draft,
                        &reference[offset..offset + 16],
                        "allow={allow} modifiers={modifiers} tile={tile} at={at}"
                    );
                    offset += 16;
                }
            }
        }
    }
}

#[test]
#[ignore = "requires POLISHED_MAP_2BPP_ORACLE; run in the reference CI job"]
fn every_bitplane_pair_and_default_palette_match_compiled_upstream() {
    let oracle =
        std::env::var_os("POLISHED_MAP_2BPP_ORACLE").expect("set POLISHED_MAP_2BPP_ORACLE");
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("hues.bin");
    let palettes = dir.path().join("palettes.bin");
    assert!(
        Command::new(oracle)
            .arg(&output)
            .arg(&palettes)
            .status()
            .unwrap()
            .success()
    );
    let reference = fs::read(output).unwrap();
    assert_eq!(reference.len(), 256 * 256 * 8);
    let palette_reference = fs::read(palettes).unwrap();
    let bank = PaletteBank::default();
    let actual: Vec<u8> = PaletteSet::ALL
        .into_iter()
        .flat_map(|set| bank.colors(set).iter().flatten().flatten().copied())
        .collect();
    assert_eq!(actual, palette_reference);
    for a in 0..=255u8 {
        for b in 0..=255u8 {
            let mut bytes = [0; 16];
            bytes[0] = a;
            bytes[1] = b;
            let tiles = decode_2bpp(&bytes).unwrap();
            let offset = (usize::from(a) * 256 + usize::from(b)) * 8;
            assert_eq!(
                &tiles[0].hues()[..8],
                &reference[offset..offset + 8],
                "planes {a:02x} {b:02x}"
            );
        }
    }
}
