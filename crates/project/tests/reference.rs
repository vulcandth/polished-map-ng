//! Optional differential run; requires the pinned C++ oracle built by tools/reference_oracle.py.
use polished_map_core::{PaletteBank, PaletteSet};
use polished_map_project::codecs::decode_2bpp;
use std::{fs, process::Command};

#[path = "reference/assembly.rs"]
mod assembly;
#[path = "reference/asset_paths.rs"]
mod asset_paths;
#[path = "reference/discovery.rs"]
mod discovery;
#[path = "reference/lz.rs"]
mod lz;
#[path = "reference/png_import.rs"]
mod png_import;
#[path = "reference/printing.rs"]
mod printing;

#[test]
#[ignore = "requires POLISHED_MAP_SIZES_ORACLE; run in the reference CI job"]
fn map_size_choices_match_original_for_every_representable_area() {
    use polished_map_project::MapSizeOptions;
    let oracle =
        std::env::var_os("POLISHED_MAP_SIZES_ORACLE").expect("set POLISHED_MAP_SIZES_ORACLE");
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("sizes.bin");
    assert!(
        Command::new(oracle)
            .arg(&output)
            .status()
            .unwrap()
            .success()
    );
    let bytes = fs::read(output).unwrap();
    let mut words = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_le_bytes([b[0], b[1]]));
    for count in 0..=65026 {
        let selected = words.next().unwrap();
        let length = words.next().unwrap();
        let pairs: Vec<_> = (0..length)
            .map(|_| (words.next().unwrap(), words.next().unwrap()))
            .collect();
        let options = MapSizeOptions::from_block_count(count);
        assert_eq!(options.choices(), pairs, "count {count}");
        assert_eq!(
            options.selected(),
            (selected != u16::MAX).then_some(usize::from(selected)),
            "count {count}"
        );
    }
    assert!(words.next().is_none());
    assert!(bytes.len().is_multiple_of(2));
}

#[test]
#[ignore = "requires POLISHED_MAP_PRIORITY_ORACLE; run in the reference CI job"]
fn priority_patterns_match_all_original_rgba_pixels() {
    use polished_map_core::PriorityPattern;
    let root = std::path::PathBuf::from(
        std::env::var_os("POLISHED_MAP_PRIORITY_ORACLE").expect("set POLISHED_MAP_PRIORITY_ORACLE"),
    );
    for (name, pattern) in ["small", "large", "chip"]
        .into_iter()
        .zip(PriorityPattern::ALL)
    {
        let png = image::load_from_memory(&fs::read(root.join(format!("{name}.png"))).unwrap())
            .unwrap()
            .to_rgba8();
        assert_eq!(
            png.dimensions(),
            (pattern.size() as u32, pattern.size() as u32)
        );
        assert_eq!(
            png.as_raw(),
            &pattern.rgba(),
            "{name}: every RGBA channel must match"
        );
    }
}

#[test]
#[ignore = "requires POLISHED_MAP_WARP_ID_ORACLE; run in the reference CI job"]
fn warp_digit_pixels_and_label_layout_match_original() {
    use polished_map_core::WarpIdLabel;
    let oracle = std::path::PathBuf::from(
        std::env::var_os("POLISHED_MAP_WARP_ID_ORACLE").expect("set POLISHED_MAP_WARP_ID_ORACLE"),
    );
    let png = image::load_from_memory(&fs::read(oracle.with_extension("png")).unwrap())
        .unwrap()
        .to_rgb8();
    assert_eq!(png.dimensions(), (80, 7));
    for value in 0..16 {
        let label = WarpIdLabel::new(if value == 0 { 16 } else { value }, true).unwrap();
        let digit = label.digits().next().unwrap();
        assert_eq!(usize::from(digit.value()), value);
        for y in 0..7 {
            for x in 0..5 {
                let channel = if digit.rows()[y] & (1 << (4 - x)) != 0 {
                    0
                } else {
                    255
                };
                assert_eq!(
                    png.get_pixel((value * 5 + x) as u32, y as u32).0,
                    [channel; 3],
                    "glyph {value:X} at {x},{y}"
                );
            }
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("labels.tsv");
    assert!(
        Command::new(&oracle)
            .arg(&output)
            .status()
            .unwrap()
            .success()
    );
    let expected = fs::read_to_string(output).unwrap();
    assert_eq!(expected.lines().count(), 8192);
    for line in expected.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 4);
        let id: usize = fields[0].parse().unwrap();
        let hex = fields[1] == "1";
        let label = WarpIdLabel::new(id, hex).unwrap();
        let calls: String = label
            .digits()
            .map(|d| format!("{},{},5,7,{},0;", d.x(), d.y(), u16::from(d.value()) * 5))
            .collect();
        assert_eq!(calls, fields[2], "ID {id} hex={hex}");
        assert_eq!(label.zoomed_text(), fields[3], "ID {id} hex={hex}");
    }
}

#[test]
#[ignore = "requires POLISHED_MAP_PALETTE_ORACLE; run in the reference CI job"]
fn palette_clipboard_and_export_match_compiled_source_with_corrected_labels() {
    use polished_map_core::{ColorSlot, PaletteCommand};
    let oracle =
        std::env::var_os("POLISHED_MAP_PALETTE_ORACLE").expect("set POLISHED_MAP_PALETTE_ORACLE");
    let dir = tempfile::tempdir().unwrap();
    let clipboard = dir.path().join("clipboard.bin");
    let prefix = dir.path().join("export");
    assert!(
        Command::new(oracle)
            .arg(&clipboard)
            .arg(&prefix)
            .status()
            .unwrap()
            .success()
    );
    let reference = fs::read(clipboard).unwrap();
    assert_eq!(reference.len(), 256 * 9);
    let a = ColorSlot {
        set: PaletteSet::Day,
        palette: 0,
        hue: 0,
    };
    let b = ColorSlot { palette: 1, ..a };
    for i in 0..256u16 {
        let mut bank = PaletteBank::default();
        bank.edit(&[PaletteCommand::SetRgb {
            slot: a,
            rgb: [i as u8, (i * 37) as u8, (i * 91) as u8],
        }])
        .unwrap();
        let snapshot = bank.copy_color(a).unwrap();
        bank.edit(&[
            PaletteCommand::SetRgb {
                slot: a,
                rgb: [i as u8 ^ 0x55, i as u8 ^ 0xaa, i as u8 ^ 0xcc],
            },
            PaletteCommand::SetRgb {
                slot: b,
                rgb: snapshot,
            },
        ])
        .unwrap();
        let mut actual = bank.color(b).unwrap().to_vec();
        bank.edit(&[PaletteCommand::Swap { a, b }]).unwrap();
        actual.extend(bank.color(a).unwrap());
        actual.extend(bank.color(b).unwrap());
        assert_eq!(
            actual,
            &reference[usize::from(i) * 9..usize::from(i + 1) * 9]
        );
    }
    for seed in 0..16 {
        let mut bank = PaletteBank::default();
        let mut edits = Vec::new();
        for (l, set) in PaletteSet::ALL.into_iter().enumerate() {
            for p in 0..9 {
                for h in 0..4 {
                    edits.push(PaletteCommand::SetRgb {
                        slot: ColorSlot {
                            set,
                            palette: p,
                            hue: h,
                        },
                        rgb: std::array::from_fn(|c| {
                            ((l * 71
                                + usize::from(p) * 37
                                + usize::from(h) * 19
                                + c * 13
                                + seed * 31)
                                & 255) as u8
                        }),
                    });
                }
            }
        }
        bank.edit(&edits).unwrap();
        for (mono, name) in [(false, "color"), (true, "mono")] {
            assert_eq!(
                bank.export_rgb(mono),
                fs::read_to_string(dir.path().join(format!("export-{seed}-{name}.pal"))).unwrap()
            );
        }
    }
    compare_roof_color_imports(dir.path());
}

fn compare_roof_color_imports(root: &std::path::Path) {
    use polished_map_core::{ColorSlot, PaletteCommand};
    use polished_map_project::{PaletteDocument, RoofPaletteLayout::*};
    let reference = fs::read(root.join("export-roofs.bin")).unwrap();
    let mut actual = Vec::new();
    let path = root.join("roof-input.pal");
    for count in [0, 1, 2, 3, 4, 512] {
        let table = (0..count * 4)
            .map(|i| {
                format!(
                    "\tRGB {},{},{}\n",
                    i * 7 % 32,
                    (i * 7 + 11) % 32,
                    (i * 7 + 22) % 32
                )
            })
            .collect::<String>();
        fs::write(&path, table).unwrap();
        for layout in [
            Custom,
            DayNite,
            MornDayNite,
            DayNiteCustom,
            MornDayNiteCustom,
        ] {
            for group in [0, 1, 2, 255] {
                let mut edits = Vec::new();
                for (l, set) in PaletteSet::ALL.into_iter().enumerate() {
                    for p in 0..9 {
                        for h in 0..4 {
                            edits.push(PaletteCommand::SetRgb {
                                slot: ColorSlot {
                                    set,
                                    palette: p,
                                    hue: h,
                                },
                                rgb: std::array::from_fn(|c| {
                                    ((l * 71 + usize::from(p) * 37 + usize::from(h) * 19 + c * 13)
                                        & 255) as u8
                                }),
                            });
                        }
                    }
                }
                let mut palettes = PaletteDocument::default();
                palettes.apply_batch(&edits).unwrap();
                actual.push(u8::from(
                    palettes.load_roof_colors(&path, group, layout).is_ok(),
                ));
                for set in PaletteSet::ALL {
                    for palette in palettes.bank().colors(set) {
                        for color in palette {
                            actual.extend(color);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(reference.len(), 120 * (1 + 6 * 9 * 4 * 3));
    assert_eq!(
        actual, reference,
        "120 original roof import outcomes and complete banks"
    );
}

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
#[test]
#[ignore = "requires POLISHED_MAP_EVENT_ORACLE; run in the reference CI job"]
fn event_coordinates_formatting_and_warps_match_compiled_original() {
    use polished_map_core::EventKind;
    use polished_map_project::EventScript;
    let oracle =
        std::env::var_os("POLISHED_MAP_EVENT_ORACLE").expect("set POLISHED_MAP_EVENT_ORACLE");
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("events.asm");
    let output = dir.path().join("events.tsv");
    let mut source = String::new();
    for kind in EventKind::ALL {
        for n in -6i32..=511 {
            for base in [10, 16, 2] {
                let number = |n: i32| match base {
                    16 => format!("${:x}", n as u32),
                    2 if n >= 0 => format!("%{n:b}"),
                    _ => n.to_string(),
                };
                let prefix = if kind.has_prefix() { "SPRITE_RED," } else { "" };
                let suffix = if kind.id_before_map() {
                    ", $2, ROUTE_1__2_B1F ; comment"
                } else {
                    ", ROUTE_1__2_B1F, $2 ; comment"
                };
                source.push_str(&format!(
                    "\t{} {prefix}{}, {}{suffix}\n",
                    kind.macro_name(),
                    number(n),
                    number(505 - n)
                ));
            }
        }
    }
    fs::write(&input, &source).unwrap();
    assert!(
        Command::new(oracle)
            .arg(&input)
            .arg(&output)
            .status()
            .unwrap()
            .success()
    );
    let expected = fs::read_to_string(&output).unwrap();
    let script = EventScript::parse(&source).unwrap();
    assert_eq!(script.events().entries().len(), 22 * 518 * 3);
    assert_eq!(expected.lines().count(), script.events().entries().len());
    for (line, event) in expected.lines().zip(script.events().entries()) {
        let d = event.definition();
        let (map, id) = event.warp_destination().unwrap_or_default();
        let actual = format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            d.x,
            d.y,
            u8::from(d.hex_coordinates),
            u8::from(d.kind.is_warp()),
            d.kind.symbol(),
            event.text(),
            map,
            id
        );
        assert_eq!(actual, line, "line {}", d.line);
    }
}
