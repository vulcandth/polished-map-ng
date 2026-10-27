//! Optional differential run; requires the pinned C++ oracle built by tools/reference_oracle.py.
use polished_map_project::codecs::decode_2bpp;
use std::{fs, process::Command};

#[test]
#[ignore = "requires POLISHED_MAP_2BPP_ORACLE; run in the reference CI job"]
fn every_bitplane_pair_matches_unmodified_upstream_function() {
    let oracle =
        std::env::var_os("POLISHED_MAP_2BPP_ORACLE").expect("set POLISHED_MAP_2BPP_ORACLE");
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("hues.bin");
    assert!(
        Command::new(oracle)
            .arg(&output)
            .status()
            .unwrap()
            .success()
    );
    let reference = fs::read(output).unwrap();
    assert_eq!(reference.len(), 256 * 256 * 8);
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
