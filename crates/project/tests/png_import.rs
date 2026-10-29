use polished_map_project::{
    BlocksetDocument, MasterGraphicsOptions, MasterTilesetDocument, PngPlatform,
};
use std::{fs, path::Path};

fn encode(color: png::ColorType, depth: png::BitDepth, samples: &[u8], height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, 8, height);
    encoder.set_color(color);
    encoder.set_depth(depth);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(samples).unwrap();
    writer.finish().unwrap();
    bytes
}

fn open(root: &Path, name: &str, platform: PngPlatform) -> MasterTilesetDocument {
    MasterTilesetDocument::open_in_project(
        root,
        name,
        &MasterGraphicsOptions {
            png_platform: platform,
            allow_256: true,
            monochrome: true,
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn sixteen_bit_samples_discard_low_byte_at_every_hue_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let gfx = dir.path().join("gfx/tilesets");
    fs::create_dir_all(&gfx).unwrap();
    let row = [0x3fffu16, 0x4000, 0x7fff, 0x8000, 0xbfff, 0xc000, 0, 0xffff];
    let samples: Vec<u8> = row.repeat(8).iter().flat_map(|v| v.to_be_bytes()).collect();
    fs::write(
        gfx.join("demo.png"),
        encode(
            png::ColorType::Grayscale,
            png::BitDepth::Sixteen,
            &samples,
            8,
        ),
    )
    .unwrap();
    for platform in [PngPlatform::Unix, PngPlatform::Windows] {
        let doc = open(dir.path(), "demo", platform);
        assert_eq!(
            doc.tileset().tiles()[0].hues().as_slice(),
            [3, 1, 1, 2, 2, 0, 3, 0].repeat(8)
        );
    }
}

#[test]
fn windows_only_clears_fully_transparent_rgba_after_depth_conversion() {
    use png::{BitDepth::*, ColorType::*};
    let dir = tempfile::tempdir().unwrap();
    let gfx = dir.path().join("gfx/tilesets");
    fs::create_dir_all(&gfx).unwrap();
    for (color, depth, pixel, windows, unix) in [
        (Rgba, Eight, vec![255, 255, 255, 0], 3, 0),
        (Rgba, Eight, vec![255, 255, 255, 1], 0, 0),
        (GrayscaleAlpha, Eight, vec![255, 0], 0, 0),
        (
            Rgba,
            Sixteen,
            vec![255, 255, 255, 255, 255, 255, 0, 1],
            3,
            0,
        ),
        (
            Rgba,
            Sixteen,
            vec![255, 255, 255, 255, 255, 255, 1, 0],
            0,
            0,
        ),
        (GrayscaleAlpha, Sixteen, vec![255, 255, 0, 1], 0, 0),
    ] {
        fs::write(
            gfx.join("demo.png"),
            encode(color, depth, &pixel.repeat(64), 8),
        )
        .unwrap();
        for (platform, hue) in [(PngPlatform::Unix, unix), (PngPlatform::Windows, windows)] {
            assert_eq!(
                open(dir.path(), "demo", platform).tileset().tiles()[0].hues(),
                &[hue; 64],
                "{color:?} {depth:?} {platform:?}"
            );
        }
    }
}

#[test]
fn platform_policy_reaches_compound_parts_roof_and_reloads_and_binds_preflight_hash() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let gfx = root.join("gfx/tilesets");
    fs::create_dir_all(gfx.join("roofs")).unwrap();
    fs::create_dir_all(root.join("data/tilesets")).unwrap();
    let bytes = encode(
        png::ColorType::Rgba,
        png::BitDepth::Eight,
        &[255, 255, 255, 0].repeat(64),
        8,
    );
    for name in ["demo.before,after", "before", "after"] {
        fs::write(gfx.join(format!("{name}.png")), &bytes).unwrap();
    }
    fs::write(root.join("data/tilesets/demo_metatiles.bin"), [0; 16]).unwrap();
    let roof = encode(
        png::ColorType::Rgba,
        png::BitDepth::Eight,
        &[255, 255, 255, 0].repeat(64 * 9),
        72,
    );
    for name in ["one", "two"] {
        fs::write(gfx.join(format!("roofs/{name}.png")), &roof).unwrap();
    }
    let blocks = BlocksetDocument::open_in_project(root, "demo.before,after").unwrap();
    let mut hashes = Vec::new();
    let mut roof_hashes = Vec::new();
    for (platform, hue) in [(PngPlatform::Unix, 0), (PngPlatform::Windows, 3)] {
        let mut doc = MasterTilesetDocument::open_in_project(
            root,
            "demo.before,after",
            &MasterGraphicsOptions {
                png_platform: platform,
                allow_256: true,
                monochrome: true,
                roof: Some("one".into()),
                ..Default::default()
            },
        )
        .unwrap();
        hashes.push(doc.content_hash(blocks.blockset()));
        roof_hashes.push(doc.roof().unwrap().content_hash());
        doc.change_roof(Some("two")).unwrap();
        let (replacement, _) = doc.prepare_replacement("demo.before,after").unwrap();
        for state in [&doc, &replacement] {
            assert_eq!(state.png_platform(), platform);
            for slot in [0, 16, 32] {
                assert_eq!(state.tileset().tiles()[slot].hues(), &[hue; 64]);
            }
            let roof = state.roof().unwrap();
            assert_eq!(roof.png_platform(), platform);
            assert_eq!(roof.name(), "two");
            for tile in roof.roof().tileset().tiles() {
                assert_eq!(tile.hues(), &[hue; 64]);
            }
        }
    }
    assert_ne!(hashes[0], hashes[1]);
    assert_ne!(roof_hashes[0], roof_hashes[1]);
    assert_eq!(fs::read(gfx.join("demo.before,after.png")).unwrap(), bytes);
}

#[test]
fn truncated_pngs_are_rejected_without_publishing_partial_tiles() {
    let dir = tempfile::tempdir().unwrap();
    let gfx = dir.path().join("gfx/tilesets");
    fs::create_dir_all(&gfx).unwrap();
    let bytes = encode(
        png::ColorType::Rgba,
        png::BitDepth::Eight,
        &[255; 64 * 4],
        8,
    );
    for end in [0, 8, 29, bytes.len() / 2, bytes.len() - 12, bytes.len() - 1] {
        fs::write(gfx.join("demo.png"), &bytes[..end]).unwrap();
        assert!(
            MasterTilesetDocument::open_in_project(
                dir.path(),
                "demo",
                &MasterGraphicsOptions::default()
            )
            .is_err(),
            "accepted prefix {end}"
        );
    }
}
