//! Complete PNG bytes through unchanged master/FLTK/libpng and Rust project loading.
use polished_map_project::{MasterGraphicsOptions, MasterTilesetDocument};
use std::{
    fs,
    io::{Cursor, Read},
    process::Command,
};

struct Case {
    name: String,
    png: Vec<u8>,
    status: u32,
}

fn encode(
    dimensions: (u32, u32),
    color: png::ColorType,
    depth: png::BitDepth,
    data: &[u8],
    palette: Option<Vec<u8>>,
    trns: Option<Vec<u8>>,
    gamma: bool,
) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, dimensions.0, dimensions.1);
    encoder.set_color(color);
    encoder.set_depth(depth);
    if let Some(palette) = palette {
        encoder.set_palette(palette);
    }
    if let Some(trns) = trns {
        encoder.set_trns(trns);
    }
    if gamma {
        encoder.set_source_gamma(png::ScaledFloat::new(0.45455));
    }
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(data).unwrap();
    writer.finish().unwrap();
    out
}

fn corpus() -> Vec<Case> {
    use png::{BitDepth::*, ColorType::*};
    let mut cases = Vec::new();
    for color in [Grayscale, GrayscaleAlpha, Rgb, Rgba] {
        for chunk in 0..4u32 {
            let mut data = Vec::new();
            for value in chunk * 16384..(chunk + 1) * 16384 {
                let v = value as u16;
                let alpha = [0u16, 1, 255, 256, 32768, 65535][value as usize % 6];
                let channels = match color {
                    Grayscale => vec![v],
                    GrayscaleAlpha => vec![v, alpha],
                    Rgb => vec![v, !v, v.wrapping_mul(173)],
                    Rgba => vec![v, !v, v.wrapping_mul(173), alpha],
                    _ => unreachable!(),
                };
                for sample in channels {
                    data.extend_from_slice(&sample.to_be_bytes());
                }
            }
            cases.push(Case {
                name: format!("16-bit {color:?} chunk {chunk}"),
                png: encode((128, 128), color, Sixteen, &data, None, None, false),
                status: 0,
            });
        }
    }
    for color in [Grayscale, GrayscaleAlpha, Rgb, Rgba] {
        for gamma in [false, true] {
            let mut data = Vec::new();
            for index in 0..16384u32 {
                let v = index as u8;
                let alpha = [0, 1, 127, 255][index as usize / 256 % 4];
                let pixel = match color {
                    Grayscale => vec![v],
                    GrayscaleAlpha => vec![v, alpha],
                    Rgb => vec![v, (index / 256) as u8 * 4, v.wrapping_mul(173)],
                    Rgba => vec![v, (index / 256) as u8 * 4, v.wrapping_mul(173), alpha],
                    _ => unreachable!(),
                };
                data.extend(pixel);
            }
            cases.push(Case {
                name: format!("8-bit {color:?} gamma={gamma}"),
                png: encode((128, 128), color, Eight, &data, None, None, gamma),
                status: 0,
            });
        }
    }
    for (depth, bits) in [(One, 1), (Two, 2), (Four, 4), (Eight, 8)] {
        let levels = 1usize << bits;
        let mut packed = vec![0u8; 256 * bits / 8];
        for i in 0..256 {
            packed[i * bits / 8] |= ((i % levels) as u8) << (8 - bits - i * bits % 8);
        }
        cases.push(Case {
            name: format!("packed gray {bits}"),
            png: encode((16, 16), Grayscale, depth, &packed, None, None, false),
            status: 0,
        });
        let palette: Vec<_> = (0..levels)
            .flat_map(|i| {
                [
                    255 - (i as u8),
                    (i as u8).wrapping_mul(57),
                    (i as u8).wrapping_mul(113),
                ]
            })
            .collect();
        for alpha in [false, true] {
            let trns = alpha.then(|| (0..levels).map(|i| [0, 1, 127, 255][i % 4]).collect());
            cases.push(Case {
                name: format!("indexed {bits} alpha={alpha}"),
                png: encode(
                    (16, 16),
                    Indexed,
                    depth,
                    &packed,
                    Some(palette.clone()),
                    trns,
                    false,
                ),
                status: 0,
            });
        }
    }
    for (color, data, trns) in [
        (Grayscale, vec![192; 256], vec![0, 192]),
        (Rgb, [192, 128, 64].repeat(256), vec![0, 192, 0, 128, 0, 64]),
    ] {
        cases.push(Case {
            name: format!("tRNS {color:?}"),
            png: encode((16, 16), color, Eight, &data, None, Some(trns), false),
            status: 0,
        });
    }
    for color in [Grayscale, Rgb] {
        let channels = if color == Rgb { 3 } else { 1 };
        let data: Vec<u8> = (0..256)
            .flat_map(|i| {
                // Same high byte, but only the exact 16-bit key is transparent.
                let sample = if i % 2 == 0 { 0xc0ffu16 } else { 0xc0fe };
                sample.to_be_bytes().repeat(channels)
            })
            .collect();
        cases.push(Case {
            name: format!("16-bit tRNS {color:?}"),
            png: encode(
                (16, 16),
                color,
                Sixteen,
                &data,
                None,
                Some(0xc0ffu16.to_be_bytes().repeat(channels)),
                false,
            ),
            status: 0,
        });
    }
    for (width, height, status) in [
        (7, 8, 3),
        (8, 7, 3),
        (136, 128, 5),
        (8, 2048, 0),
        (2048, 8, 0),
    ] {
        cases.push(Case {
            name: format!("dimensions {width}x{height}"),
            png: encode(
                (width, height),
                Grayscale,
                Eight,
                &vec![85; (width * height) as usize],
                None,
                None,
                false,
            ),
            status,
        });
    }
    cases
}

fn word(input: &mut Cursor<Vec<u8>>) -> u32 {
    let mut bytes = [0; 4];
    input.read_exact(&mut bytes).unwrap();
    u32::from_le_bytes(bytes)
}

#[test]
#[ignore = "requires POLISHED_MAP_PNG_ORACLE; run in reference CI"]
fn png_import_matches_original_depth_color_alpha_and_tile_order() {
    let oracle = std::env::var_os("POLISHED_MAP_PNG_ORACLE").expect("set POLISHED_MAP_PNG_ORACLE");
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input.bin");
    let output = temp.path().join("output.bin");
    let scratch = temp.path().join("case.png");
    fs::create_dir_all(temp.path().join("gfx/tilesets")).unwrap();
    let path = temp.path().join("gfx/tilesets/demo.png");
    let mut cases = corpus();
    let mut packet = (cases.len() as u32).to_le_bytes().to_vec();
    for case in &cases {
        packet.extend_from_slice(&(case.png.len() as u32).to_le_bytes());
        packet.extend_from_slice(&case.png);
    }
    fs::write(&input, &packet).unwrap();
    assert!(
        Command::new(&oracle)
            .args([&input, &output, &scratch])
            .arg("--interlace")
            .status()
            .unwrap()
            .success()
    );
    let mut interlaced = Cursor::new(fs::read(&output).unwrap());
    assert_eq!(word(&mut interlaced) as usize, cases.len());
    let originals = cases.len();
    for i in 0..originals {
        let size = word(&mut interlaced) as usize;
        assert!(size <= 1024 * 1024);
        let mut png = vec![0; size];
        interlaced.read_exact(&mut png).unwrap();
        assert_eq!(png[28], 1, "IHDR must declare Adam7");
        packet.extend_from_slice(&(size as u32).to_le_bytes());
        packet.extend_from_slice(&png);
        cases.push(Case {
            name: format!("Adam7 {}", cases[i].name),
            png,
            status: cases[i].status,
        });
    }
    assert_eq!(interlaced.position() as usize, interlaced.get_ref().len());
    packet[..4].copy_from_slice(&(cases.len() as u32).to_le_bytes());
    fs::write(&input, packet).unwrap();
    assert!(
        Command::new(oracle)
            .args([&input, &output, &scratch])
            .status()
            .unwrap()
            .success()
    );
    let mut input = Cursor::new(fs::read(output).unwrap());
    let mut pixels = 0;
    let mut original_hues = Vec::new();
    for (case_index, case) in cases.iter().enumerate() {
        fs::write(&path, &case.png).unwrap();
        for windows in [false, true] {
            let status = word(&mut input);
            let count = word(&mut input) as usize;
            let len = word(&mut input) as usize;
            assert!(len <= 16384);
            let mut hues = vec![0; len];
            input.read_exact(&mut hues).unwrap();
            if case_index < originals {
                original_hues.push(hues.clone());
            } else {
                assert_eq!(
                    hues,
                    original_hues[(case_index - originals) * 2 + usize::from(windows)],
                    "Adam7 repacking changed samples: {}",
                    case.name
                );
            }
            assert_eq!(status, case.status, "{} windows={windows}", case.name);
            let doc = MasterTilesetDocument::open_in_project(
                temp.path(),
                "demo",
                &MasterGraphicsOptions {
                    png_platform: if windows {
                        polished_map_project::PngPlatform::Windows
                    } else {
                        polished_map_project::PngPlatform::Unix
                    },
                    allow_256: true,
                    monochrome: true,
                    ..Default::default()
                },
            );
            if status != 0 {
                assert!(doc.is_err(), "{}", case.name);
                continue;
            }
            assert_eq!(len, count * 64);
            let doc = doc.unwrap();
            let actual: Vec<_> = doc.tileset().tiles()[..count]
                .iter()
                .flat_map(|t| t.hues().iter().copied())
                .collect();
            if let Some(i) = actual.iter().zip(&hues).position(|(a, b)| a != b) {
                panic!(
                    "{} windows={windows} hue {i}: Rust={} source={}",
                    case.name, actual[i], hues[i]
                );
            }
            pixels += len;
        }
    }
    assert_eq!(input.position() as usize, input.get_ref().len());
    println!("{} PNG cases; {pixels} hues compared", cases.len());
}
