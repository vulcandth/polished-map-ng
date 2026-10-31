use polished_map_core::{
    BlockBrush, Point,
    plusplus::{AttributeBlockset, AttributeCommand, BankOrder, TileReference},
};

fn output(mode: &str) -> Vec<u8> {
    let oracle =
        std::env::var_os("POLISHED_MAP_PLUSPLUS_BLOCK_ORACLE").expect("set compiled oracle path");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("output.bin");
    assert!(
        std::process::Command::new(oracle)
            .arg(mode)
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    std::fs::read(path).unwrap()
}

#[test]
#[ignore = "requires compiled pinned plusplus source"]
fn original_tile_address_and_attribute_roundtrip() {
    let bytes = output("address");
    let mut actual = Vec::new();
    for order in [BankOrder::Bank1First, BankOrder::Bank0First] {
        for offset in 0..=255 {
            for attr in 0..=255 {
                let t = TileReference::from_bytes(offset, attr);
                actual.extend_from_slice(&t.index(order).to_le_bytes());
                actual.push(t.attribute());
            }
        }
    }
    assert_eq!(actual.len(), bytes.len());
    assert!(actual == bytes, "original tile address/attribute mismatch");
}

#[test]
#[ignore = "requires compiled pinned plusplus source"]
fn original_brush_all_tiles_modifiers_flips_and_edges() {
    let bytes = output("brush");
    let mut cursor = 0;
    for order in [BankOrder::Bank1First, BankOrder::Bank0First] {
        for mods in 0..8 {
            for tile in 0..512 {
                for at in 0..16 {
                    for flips in 0..4 {
                        let mut blockset =
                            AttributeBlockset::from_bytes(&[0; 16], &[0; 16]).unwrap();
                        blockset
                            .apply(&AttributeCommand::Paint {
                                block: 0,
                                at: Point {
                                    x: at % 4,
                                    y: at / 4,
                                },
                                tile,
                                palette: (tile & 7) as u8,
                                x_flip: flips & 1 != 0,
                                y_flip: flips & 2 != 0,
                                priority: tile & 8 != 0,
                                order,
                                brush: BlockBrush {
                                    shift: mods & 1 != 0,
                                    ctrl: mods & 2 != 0,
                                    alt: mods & 4 != 0,
                                },
                            })
                            .unwrap();
                        let actual: Vec<_> = blockset
                            .block(0)
                            .unwrap()
                            .into_iter()
                            .flat_map(|t| [t.offset(), t.attribute()])
                            .collect();
                        assert_eq!(
                            &actual,
                            &bytes[cursor..cursor + 32],
                            "order={order:?}, mods={mods}, tile={tile}, at={at}, flips={flips}"
                        );
                        cursor += 32;
                    }
                }
            }
        }
    }
    assert_eq!(cursor, bytes.len());
}

#[test]
#[ignore = "requires compiled pinned plusplus source"]
fn original_render_strides_match_banks_palettes_flips_undefined_tiles_and_roofs() {
    use polished_map_core::{
        Palette, Tile,
        plusplus::{Graphics, RawTileset},
    };
    let bytes = output("render");
    let raw = RawTileset::new(
        (0..512)
            .map(|i| {
                Tile::new(std::array::from_fn(|p| {
                    ((i + p % 8 / 2 + p / 8 / 3) % 4) as u8
                }))
                .unwrap()
            })
            .collect(),
        (0..512).map(|i| i % 11 != 0).collect(),
    )
    .unwrap();
    let roof_tiles: [Tile; 9] = std::array::from_fn(|i| {
        Tile::new(std::array::from_fn(|p| {
            ((i + p % 8 + 3 * (p / 8)) % 4) as u8
        }))
        .unwrap()
    });
    let palettes: [Palette; 8] = std::array::from_fn(|p| {
        std::array::from_fn(|h| {
            [
                (p * 23 + h * 3) as u8,
                (p * 7 + h * 29) as u8,
                (250 - p * 9 - h * 17) as u8,
            ]
        })
    });
    let mut cursor = 0;
    for order in [BankOrder::Bank1First, BankOrder::Bank0First] {
        for roof in [false, true] {
            for base in 0..16 {
                for attr in 0..256 {
                    let offsets: [u8; 16] = std::array::from_fn(|i| (base + i * 17) as u8);
                    let attrs: [u8; 16] = std::array::from_fn(|i| (attr + i) as u8);
                    let blocks = AttributeBlockset::from_bytes(&offsets, &attrs).unwrap();
                    let g = Graphics::new(raw.clone(), blocks, palettes, order);
                    let g = if roof {
                        g.with_roof(roof_tiles.clone())
                    } else {
                        g
                    };
                    let image = g.render_block(0).unwrap();
                    assert!(
                        image.pixels == bytes[cursor..cursor + 3072],
                        "order={order:?}, roof={roof}, base={base}, attr={attr}"
                    );
                    cursor += 3072;
                }
            }
        }
    }
    assert_eq!(cursor, bytes.len());
}
