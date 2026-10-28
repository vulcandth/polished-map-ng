//! Optional differential run; requires the pinned C++ oracle built by tools/reference_oracle.py.
use polished_map_core::{PaletteBank, PaletteSet};
use polished_map_project::codecs::decode_2bpp;
use std::{fs, process::Command};

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
