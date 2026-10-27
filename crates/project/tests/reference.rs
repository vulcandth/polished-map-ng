//! Optional differential run; requires the pinned C++ oracle built by tools/reference_oracle.py.
use polished_map_core::{PaletteBank, PaletteSet};
use polished_map_project::codecs::decode_2bpp;
use std::{fs, process::Command};

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
