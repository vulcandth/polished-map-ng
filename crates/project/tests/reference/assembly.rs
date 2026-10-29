//! Unchanged Parsed_Asm and CHR loader differential, with explicit safety differences.
use polished_map_project::codecs::{decode_2bpp, parse_asm};
use std::{
    fs,
    io::{Cursor, Read},
    process::Command,
};

struct Case {
    name: String,
    text: String,
    bytes: Vec<u8>,
    result: u32,
    strict_error: bool,
}

fn number(n: u16, style: usize) -> String {
    match style {
        0 => format!("${n:X}"),
        1 => format!("%{n:b}"),
        2 => format!("{n:04X}h"),
        _ => n.to_string(),
    }
}

fn corpus() -> Vec<Case> {
    let mut cases = Vec::new();
    let mut valid = |name: String, text: String, bytes: Vec<u8>| {
        cases.push(Case {
            name,
            text,
            bytes,
            result: 0,
            strict_error: false,
        })
    };
    valid("empty".into(), String::new(), vec![]);
    valid(
        "ignored statements".into(),
        "Label:\ndb 42\nLabel: db 90\n\tld a, 5\n\tdb$10\n\tdb\n\tdw ; no operands\n; comment\n"
            .into(),
        vec![],
    );
    valid(
        "signs, case and C whitespace".into(),
        " \u{b}DB\u{c}+1, +2, $+03, %+100, +5H\r\n\tDw $1234\t, %100000000\r\n".into(),
        vec![1, 2, 3, 4, 5, 0x34, 0x12, 0, 1],
    );
    valid("terminal comma".into(), "\tdb 1,2,".into(), vec![1, 2]);
    for style in 0..4 {
        let mut text = String::new();
        for first in (0..256).step_by(16) {
            text.push_str(["\tdb ", " DB\t", " \tDb ", "\t dB "][style]);
            text.push_str(
                &(first..first + 16)
                    .map(|n| number(n, style))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            text.push_str(" ; all byte values\r\n");
        }
        valid(
            format!("all bytes style {style}"),
            text,
            (0..=255).collect(),
        );
        for first in (0..65536u32).step_by(2048) {
            let mut text = String::new();
            let mut bytes = Vec::new();
            for start in (first..first + 2048).step_by(8) {
                text.push_str(["\tdw ", " DW\t", " \tDw ", "\t dW "][style]);
                text.push_str(
                    &(start..start + 8)
                        .map(|n| number(n as u16, style))
                        .collect::<Vec<_>>()
                        .join(","),
                );
                text.push_str(" ; little endian\n");
                for n in start..start + 8 {
                    bytes.extend_from_slice(&(n as u16).to_le_bytes());
                }
            }
            valid(format!("words {first} style {style}"), text, bytes);
        }
    }
    for length in [1, 15, 16, 17, 4095, 4096, 4097, 4112] {
        let bytes: Vec<u8> = (0..length).map(|i| (i * 37) as u8).collect();
        let text = bytes
            .chunks(16)
            .map(|row| {
                format!(
                    "\tdb {}\n",
                    row.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
                )
            })
            .collect();
        valid(format!("length {length}"), text, bytes);
    }
    for (name, text, bytes, result) in [
        ("byte overflow", "\tdb 1,256", vec![1], 2),
        ("word overflow", "\tdw 1,65536", vec![1, 0], 2),
        ("negative", "\tdb -1", vec![], 2),
        ("unknown number", "\tdb unknown", vec![0], 0),
        ("partial number", "\tdb 12tail", vec![12], 0),
        ("empty interior operand", "\tdb 1,,2", vec![1, 0, 2], 0),
        ("empty initial operand", "\tdw ,1", vec![0, 0, 1, 0], 0),
    ] {
        cases.push(Case {
            name: name.into(),
            text: text.into(),
            bytes,
            result,
            strict_error: true,
        });
    }
    cases
}

fn u32_at(input: &mut Cursor<Vec<u8>>) -> u32 {
    let mut bytes = [0; 4];
    input.read_exact(&mut bytes).unwrap();
    u32::from_le_bytes(bytes)
}

#[test]
#[ignore = "requires POLISHED_MAP_ASM_ORACLE; run in the reference CI job"]
fn assembly_numbers_and_chr_tiles_match_original_with_explicit_safety_errors() {
    let oracle = std::env::var_os("POLISHED_MAP_ASM_ORACLE").expect("set POLISHED_MAP_ASM_ORACLE");
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input.bin");
    let output = temp.path().join("output.bin");
    let scratch = temp.path().join("case.CHR");
    fs::create_dir_all(temp.path().join("gfx/tilesets")).unwrap();
    let tiles_path = temp.path().join("gfx/tilesets/demo.CHR");
    let cases = corpus();
    let mut packet = (cases.len() as u32).to_le_bytes().to_vec();
    for case in &cases {
        packet.extend_from_slice(&(case.text.len() as u32).to_le_bytes());
        packet.extend_from_slice(case.text.as_bytes());
    }
    fs::write(&input, packet).unwrap();
    assert!(
        Command::new(oracle)
            .args([&input, &output, &scratch])
            .status()
            .unwrap()
            .success()
    );
    let mut input = Cursor::new(fs::read(output).unwrap());
    let mut hues_compared = 0;
    for case in &cases {
        assert_eq!(
            u32_at(&mut input),
            case.result,
            "{}: parser status",
            case.name
        );
        let length = u32_at(&mut input) as usize;
        assert!(length <= 8192);
        let mut bytes = vec![0; length];
        input.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, case.bytes, "{}: independent byte values", case.name);
        let status = u32_at(&mut input);
        let count = u32_at(&mut input) as usize;
        assert!(count <= 256);
        let mut hues = vec![0; count * 64];
        input.read_exact(&mut hues).unwrap();
        let expected_status = if case.result != 0 {
            1
        } else if !length.is_multiple_of(16) {
            3
        } else if length > 4096 {
            5
        } else {
            0
        };
        assert_eq!(status, expected_status, "{}: CHR status", case.name);
        if case.strict_error {
            assert!(
                parse_asm(&case.text).is_err(),
                "{}: reject silent substitution",
                case.name
            );
            continue;
        }
        let rust = parse_asm(&case.text).unwrap();
        assert_eq!(rust, bytes, "{}: Rust parsed bytes", case.name);
        if status == 0 {
            let tiles = decode_2bpp(&rust).unwrap_or_else(|e| panic!("{}: {e}", case.name));
            assert_eq!(tiles.len(), count, "{}: tile count", case.name);
            assert_eq!(
                tiles
                    .iter()
                    .flat_map(|t| t.hues().iter().copied())
                    .collect::<Vec<_>>(),
                hues,
                "{}: every hue",
                case.name
            );
            hues_compared += hues.len();
            fs::write(&tiles_path, &case.text).unwrap();
            let document = polished_map_project::MasterTilesetDocument::open_in_project(
                temp.path(),
                "demo",
                &polished_map_project::MasterGraphicsOptions {
                    allow_256: true,
                    monochrome: true,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(
                &document.tileset().tiles()[..count],
                tiles,
                "{}: public CHR route",
                case.name
            );
        } else {
            assert!(
                decode_2bpp(&rust).is_err(),
                "{}: invalid dimensions",
                case.name
            );
        }
    }
    assert_eq!(input.position() as usize, input.get_ref().len());
    println!(
        "{} assembly cases; {hues_compared} hues compared",
        cases.len()
    );
}
