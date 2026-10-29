//! Valid-stream differential against unchanged master LZ and 2bpp loader functions.
use polished_map_project::codecs::{CodecError, decode_lz_tiles, decompress_lz};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    process::Command,
};

struct Stream {
    bytes: Vec<u8>,
    length: usize,
}

impl Stream {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            length: 0,
        }
    }

    fn command(&mut self, command: u8, length: usize, long: bool, payload: &[u8]) {
        assert!(command < 7 && (1..=1024).contains(&length));
        if long {
            self.bytes.extend([
                0xe0 | command << 2 | ((length - 1) >> 8) as u8,
                (length - 1) as u8,
            ]);
        } else {
            assert!(length <= 32);
            self.bytes.push(command << 5 | (length - 1) as u8);
        }
        self.bytes.extend(payload);
        self.length += length;
        assert!(self.length <= 4096);
    }

    fn seed(length: usize) -> Self {
        let mut stream = Self::new();
        while stream.length < length {
            let n = (length - stream.length).min(1024);
            let bytes: Vec<_> = (stream.length..stream.length + n)
                .map(|i| (i * 37 + i / 256) as u8)
                .collect();
            stream.command(0, n, true, &bytes);
        }
        stream
    }

    fn copy(&mut self, command: u8, length: usize, long: bool, start: usize, relative: bool) {
        assert!((4..=6).contains(&command) && start < self.length);
        if command == 6 {
            assert!(length <= start + 1);
        }
        let offset = if relative {
            let distance = self.length - start;
            assert!((1..=128).contains(&distance));
            vec![0x80 | (distance - 1) as u8]
        } else {
            vec![(start >> 8) as u8, start as u8]
        };
        self.command(command, length, long, &offset);
    }

    fn finish(mut self, label: String) -> Case {
        self.bytes.push(0xff);
        Case {
            bytes: self.bytes,
            length: self.length,
            label,
            error: None,
        }
    }
}

struct Case {
    bytes: Vec<u8>,
    length: usize,
    label: String,
    error: Option<(u32, CodecError)>,
}

fn corpus() -> Vec<Case> {
    let mut cases = vec![Stream::new().finish("empty terminator".into())];
    for command in 0..=6 {
        for long in [false, true] {
            for length in 1..=if long { 1024 } else { 32 } {
                if command < 4 {
                    let mut stream = Stream::new();
                    let payload = match command {
                        0 => (0..length).map(|i| (i * 37 + length) as u8).collect(),
                        1 => vec![length as u8],
                        2 => vec![length as u8, !(length as u8)],
                        _ => Vec::new(),
                    };
                    stream.command(command, length, long, &payload);
                    cases.push(
                        stream.finish(format!("command={command} long={long} length={length}")),
                    );
                } else {
                    for relative in [false, true] {
                        let mut stream = Stream::seed(2048);
                        let start = if relative || command == 6 { 2047 } else { 0 };
                        stream.copy(command, length, long, start, relative);
                        cases.push(stream.finish(format!(
                            "command={command} long={long} length={length} relative={relative}"
                        )));
                    }
                }
            }
        }
    }
    // Every encodable relative distance; flip/repeat include overlap expansion.
    for command in 4..=6 {
        for distance in 1..=128 {
            for length in [1, 32, 33, 1024] {
                let mut stream = Stream::seed(2048);
                stream.copy(command, length, length > 32, 2048 - distance, true);
                cases.push(stream.finish(format!(
                    "relative command={command} distance={distance} length={length}"
                )));
            }
        }
        for start in [0, 1, 127, 128, 255, 256, 1023, 1024, 2047, 2048, 4094] {
            let mut stream = Stream::seed(4095);
            stream.copy(command, 1, false, start, false);
            cases.push(stream.finish(format!(
                "absolute command={command} start={start}, full output"
            )));
        }
    }
    // Deterministic mixed sequences: generator tracks only lengths/valid ranges,
    // never implements a second decoder or supplies expected decoded bytes.
    for seed in 0..256_u32 {
        let mut random = seed + 1;
        let mut next = || {
            random = random.wrapping_mul(1664525).wrapping_add(1013904223);
            random as usize
        };
        let mut stream = Stream::seed(32);
        for _ in 0..64 {
            if stream.length == 4096 {
                break;
            }
            let command = (next() % 7) as u8;
            let mut length = (next() % 128 + 1).min(4096 - stream.length);
            let long = length > 32 || next() % 2 == 0;
            if command >= 4 {
                let relative = next() % 2 == 0;
                let start = if relative {
                    stream.length - 1 - next() % stream.length.min(128)
                } else {
                    next() % stream.length
                };
                if command == 6 {
                    length = length.min(start + 1);
                }
                stream.copy(command, length, long, start, relative);
            } else {
                let n = match command {
                    0 => length,
                    1 => 1,
                    2 => 2,
                    _ => 0,
                };
                let payload: Vec<_> = (0..n).map(|_| (next() >> 16) as u8).collect();
                stream.command(command, length, long, &payload);
            }
        }
        cases.push(stream.finish(format!("mixed seed={seed}")));
    }
    cases.push(Stream::seed(4096).finish("exact 4096-byte literal output".into()));
    let mut case = Stream::seed(16).finish("terminator ignores trailing data".into());
    case.bytes.extend([0xfc, 0x00, 0x80, 0xff]);
    cases.push(case);
    // These two malformed cases are safe in the original: both return before
    // any unchecked operand access or output write. All other malformed input
    // belongs exclusively in the Rust safety tests.
    cases.push(Case {
        bytes: vec![0xfc, 0, 0xff],
        length: 0,
        label: "reserved command".into(),
        error: Some((7, CodecError::LzCommand(0))),
    });
    let mut stream = Stream::seed(4096);
    stream.bytes.extend([0x60, 0xff]);
    cases.push(Case {
        bytes: stream.bytes,
        length: 4096,
        label: "command after full output".into(),
        error: Some((5, CodecError::Limit(4096))),
    });
    cases
}

#[test]
#[ignore = "requires POLISHED_MAP_LZ_ORACLE; run in the reference CI job"]
fn lz_loader_matches_original_commands_lengths_offsets_and_padding() {
    let oracle = std::env::var_os("POLISHED_MAP_LZ_ORACLE").expect("set POLISHED_MAP_LZ_ORACLE");
    let cases = corpus();
    assert_eq!(cases.len(), 12_390);
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("cases.bin");
    let output = dir.path().join("hues.bin");
    let scratch = dir.path().join("stream.lz");
    let mut file = BufWriter::new(File::create(&input).unwrap());
    file.write_all(&(cases.len() as u32).to_le_bytes()).unwrap();
    for case in &cases {
        file.write_all(&(case.bytes.len() as u32).to_le_bytes())
            .unwrap();
        file.write_all(&case.bytes).unwrap();
    }
    file.flush().unwrap();
    drop(file);
    assert!(
        Command::new(oracle)
            .args([&input, &output, &scratch])
            .status()
            .unwrap()
            .success()
    );
    let mut file = BufReader::new(File::open(output).unwrap());
    let mut compared = 0;
    for case in &cases {
        let mut bytes = [0; 4];
        file.read_exact(&mut bytes).unwrap();
        let status = u32::from_le_bytes(bytes);
        file.read_exact(&mut bytes).unwrap();
        let tiles = u32::from_le_bytes(bytes);
        if let Some((expected, error)) = &case.error {
            assert_eq!((status, tiles), (*expected, 0), "{}", case.label);
            assert_eq!(
                &decompress_lz(&case.bytes, 4096).unwrap_err(),
                error,
                "{}",
                case.label
            );
            continue;
        }
        assert_eq!((status, tiles), (0, 256), "{}", case.label);
        let decoded = decompress_lz(&case.bytes, 4096).unwrap();
        assert_eq!(decoded.len(), case.length, "{}", case.label);
        let actual: Vec<_> = decode_lz_tiles(&case.bytes)
            .unwrap()
            .iter()
            .flat_map(|tile| tile.hues().iter().copied())
            .collect();
        let mut original = vec![0; 256 * 64];
        file.read_exact(&mut original).unwrap();
        assert_eq!(actual.len(), original.len(), "{}", case.label);
        if let Some(index) = actual.iter().zip(&original).position(|(a, b)| a != b) {
            panic!(
                "{}: hue {index}: Rust={}, original={}",
                case.label, actual[index], original[index]
            );
        }
        compared += original.len();
    }
    assert_eq!(file.read(&mut [0]).unwrap(), 0);
    println!(
        "Compared {} LZ cases and {compared} original tile hues",
        cases.len()
    );
}
