//! Compare composition with unchanged original Print functions and decoded PNGs.
//! The synthetic coverage masks intentionally do not assert native font fidelity.
use polished_map_core::{
    EventDefinition, EventKind, Events, Graphics, Map, PrintFont, PrintOptions, PrintStyle,
    PrintTextMask, Tile,
};
use std::{fs, path::PathBuf, process::Command};

fn definition(kind: EventKind, x: i16, y: i16, hex_coordinates: bool) -> EventDefinition {
    EventDefinition {
        kind,
        x,
        y,
        hex_coordinates,
        prefix: kind.has_prefix().then(|| "0,".into()),
        suffix: Some(", 1, DESTINATION".into()),
        line: 1,
    }
}

#[test]
#[ignore = "requires POLISHED_MAP_PRINT_ORACLE; run in the reference CI job"]
fn print_composition_matches_compiled_original_for_all_options() {
    let oracle = PathBuf::from(
        std::env::var_os("POLISHED_MAP_PRINT_ORACLE").expect("set POLISHED_MAP_PRINT_ORACLE"),
    );
    let temp = tempfile::tempdir().unwrap();
    let assets = temp.path().join("assets.rgba");
    let input = temp.path().join("events.txt");
    let output = temp.path().join("print.rgb");
    let mut rgba = Vec::new();
    for (name, dimensions) in [
        ("texture_red16", (16, 16)),
        ("texture_green16", (16, 16)),
        ("texture_blue16", (16, 16)),
        ("texture_purple16", (16, 16)),
        ("texture_orange16", (16, 16)),
        ("texture_azure16", (16, 16)),
        ("warp_digits", (80, 7)),
        ("small_priority", (8, 8)),
    ] {
        let png = fs::read(oracle.with_extension("assets").join(format!("{name}.png"))).unwrap();
        let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(decoded.dimensions(), dimensions);
        rgba.extend_from_slice(decoded.as_raw());
    }
    fs::write(&assets, rgba).unwrap();
    // Off-map warps still advance the numbering; the first visible warp is 16.
    let mut definitions: Vec<_> = (0..15)
        .map(|_| definition(EventKind::WarpEvent, -6, -6, false))
        .collect();
    for (index, kind) in EventKind::ALL.into_iter().enumerate() {
        definitions.push(definition(
            kind,
            (index % 4) as i16,
            (index / 4 % 4) as i16,
            index % 2 == 0,
        ));
    }
    // Overlaps, exact edges, and ink overhanging a nominally off-map rectangle.
    for (x, y) in [(-1, 0), (0, -1), (4, 0), (0, 4), (3, 3), (0, 0)] {
        definitions.push(definition(EventKind::WarpEvent, x, y, true));
    }
    fs::write(
        &input,
        definitions
            .iter()
            .map(|d| {
                format!(
                    "{} {} {} {}\n",
                    d.kind.macro_name(),
                    d.x,
                    d.y,
                    u8::from(d.hex_coordinates)
                )
            })
            .collect::<String>(),
    )
    .unwrap();
    assert!(
        Command::new(&oracle)
            .args([&assets, &input, &output])
            .status()
            .unwrap()
            .success()
    );
    let expected = fs::read(output).unwrap();
    assert_eq!(expected.len(), 256 * 64 * 64 * 3);
    let events = Events::new(definitions).unwrap();
    let map = Map::from_bytes(2, 2, vec![0, 1, 255, 1]).unwrap();
    let graphics = Graphics::new(
        vec![Tile::new([0; 64]).unwrap(); 3],
        vec![[[20, 40, 60]; 4], [[200, 100, 50]; 4], [[100, 180, 220]; 4]],
        vec![
            [0; 16],
            std::array::from_fn(|i| if i % 2 == 0 { 1 } else { 2 }),
        ],
    )
    .unwrap()
    .with_tile_priorities(vec![false, false, true])
    .unwrap();
    let mut offset = 0;
    for large in [false, true] {
        for grid in [85, 38] {
            for hexadecimal_ids in [false, true] {
                for flags in 0..32 {
                    let options = PrintOptions {
                        grid: flags & 1 != 0,
                        ids: flags & 2 != 0,
                        priority: flags & 4 != 0,
                        events: flags & 8 != 0,
                        warp_ids: flags & 16 != 0,
                    };
                    let actual = graphics
                        .render_print(
                            &map,
                            Some(&events),
                            options,
                            PrintStyle {
                                hexadecimal_ids,
                                grid_rgb: [grid; 3],
                            },
                            |font, text| {
                                let width = if large {
                                    if font == PrintFont::CourierBold14 {
                                        40
                                    } else {
                                        35
                                    }
                                } else {
                                    text.len() * 3
                                };
                                let height = if large { 35 } else { 5 };
                                let coverage = (0..width * height)
                                    .map(|i| {
                                        [0, 64, 128, 255][(i % width
                                            + i / width
                                            + usize::from(text.as_bytes()[i % width % text.len()]))
                                            % 4]
                                    })
                                    .collect();
                                PrintTextMask::new(width as u16, height as u16, coverage)
                            },
                        )
                        .unwrap();
                    let end = offset + actual.pixels.len();
                    if actual.pixels != expected[offset..end] {
                        let at = actual
                            .pixels
                            .iter()
                            .zip(&expected[offset..end])
                            .position(|(a, b)| a != b)
                            .unwrap();
                        panic!(
                            "large={large} grid={grid} hex={hexadecimal_ids} flags={flags}: first difference at ({},{}), channel {}: Rust {}, original {}",
                            at / 3 % 64,
                            at / 3 / 64,
                            at % 3,
                            actual.pixels[at],
                            expected[offset + at]
                        );
                    }
                    offset = end;
                }
            }
        }
    }
    assert_eq!(offset, expected.len());
    assert_eq!(map.blocks(), &[0, 1, 255, 1]);
    assert!(!events.is_modified());
}
