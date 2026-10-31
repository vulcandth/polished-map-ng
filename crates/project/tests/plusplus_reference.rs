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
