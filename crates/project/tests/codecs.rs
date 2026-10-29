use polished_map_project::codecs::*;

#[test]
fn plane_weighting_uses_upstream_order_not_conventional_indices() {
    let mut bytes = [0; 16];
    bytes[0] = 0b1010_0000;
    bytes[1] = 0b0110_0000;
    let tiles = decode_2bpp(&bytes).unwrap();
    assert_eq!(&tiles[0].hues()[..4], &[2, 1, 3, 0]);
    assert_eq!(encode_2bpp(&tiles), bytes);
}

#[test]
fn all_byte_values_round_trip() {
    let bytes: Vec<_> = (0..=255).collect();
    assert_eq!(encode_2bpp(&decode_2bpp(&bytes).unwrap()), bytes);
    assert!(decode_2bpp(&[]).unwrap().is_empty());
    for n in [1, 15, 17, 4097] {
        assert!(decode_2bpp(&vec![0; n]).is_err());
    }
}

#[test]
fn lz_vectors_cover_every_short_command_and_long_form() {
    assert_eq!(decompress_lz(&[2, 1, 2, 3, 0xff], 32).unwrap(), [1, 2, 3]);
    assert_eq!(decompress_lz(&[0x22, 7, 0xff], 32).unwrap(), [7, 7, 7]);
    assert_eq!(
        decompress_lz(&[0x44, 1, 2, 0xff], 32).unwrap(),
        [1, 2, 1, 2, 1]
    );
    assert_eq!(decompress_lz(&[0x62, 0xff], 32).unwrap(), [0, 0, 0]);
    assert_eq!(
        decompress_lz(&[0, 5, 0x83, 0x80, 0xff], 32).unwrap(),
        [5, 5, 5, 5, 5]
    );
    assert_eq!(
        decompress_lz(&[0, 1, 0xa0, 0, 0, 0xff], 32).unwrap(),
        [1, 128]
    );
    assert_eq!(
        decompress_lz(&[2, 1, 2, 3, 0xc2, 0, 2, 0xff], 32).unwrap(),
        [1, 2, 3, 3, 2, 1]
    );
    assert_eq!(decompress_lz(&[0xec, 39, 0xff], 64).unwrap(), vec![0; 40]);
    assert_eq!(decode_lz_tiles(&[0xff]).unwrap().len(), 256);
}

#[test]
fn lz_malformed_data_is_bounded_and_never_panics() {
    for input in [
        &[][..],
        &[0],
        &[0x80, 0x80, 0xff],
        &[0xfc, 0, 0xff],
        &[0xe0],
    ] {
        assert!(decompress_lz(input, 4096).is_err());
    }
    assert!(decompress_lz(&[0x7f, 0xff], 31).is_err());
    for a in 0..=255 {
        for b in 0..=255 {
            let _ = decompress_lz(&[a, b], 64);
        }
    }
}

#[test]
fn lz_truncated_sequences_and_invalid_output_ranges_return_errors() {
    // All seven commands plus a long header. None of the incomplete prefixes
    // contains an end marker at a command boundary.
    let valid = [
        2, 0x12, 0x34, 0x56, 0x22, 0xab, 0x44, 1, 2, 0x62, 0x83, 0x80, 0xa2, 0x82, 0xc2, 0x82,
        0xec, 39, 0xff,
    ];
    assert_eq!(decompress_lz(&valid, 4096).unwrap().len(), 64);
    for end in 0..valid.len() {
        assert!(decompress_lz(&valid[..end], 4096).is_err(), "prefix {end}");
    }

    // Original code can read/write outside initialized data on these inputs;
    // never send them to the C++ differential harness.
    for input in [
        &[0x80, 0x80, 0xff][..],   // Relative address before the start.
        &[0, 7, 0x80, 0, 1, 0xff], // Absolute address in the unwritten tail.
        &[0, 7, 0xc1, 0, 0, 0xff], // Reverse copy crosses the start.
    ] {
        assert!(matches!(
            decompress_lz(input, 4096),
            Err(CodecError::BackReference(_))
        ));
    }
    let mut full = vec![0xef, 0xff, 0xef, 0xff, 0xef, 0xff, 0xef, 0xfe];
    full.extend([0x61, 0xff]); // 4095 initialized bytes plus a two-byte command.
    assert_eq!(decompress_lz(&full, 4096), Err(CodecError::Limit(4096)));
    assert!(decompress_lz(&[0xff], 0).unwrap().is_empty());
    assert_eq!(decompress_lz(&[0x60, 0xff], 0), Err(CodecError::Limit(0)));
}

#[test]
fn assembly_respects_indentation_numbers_comments_and_little_endian() {
    let text = "label:\n\tdb $12, %1010, 0FFh ; comment\r\n DW $1234, 256\ndb 99\n";
    assert_eq!(parse_asm(text).unwrap(), [18, 10, 255, 52, 18, 0, 1]);
    assert!(parse_asm("\tdb 256").is_err());
    assert!(parse_asm("\tdb unknown").is_err());
}

#[test]
fn assembly_rejects_missing_operands_without_shifting_later_bytes() {
    for text in ["\tdb 1,,2", "\tdw ,1", "\tdb 1, ,2", "\tdw 0,\t,1"] {
        assert!(
            matches!(parse_asm(text), Err(CodecError::Assembly { line: 1, value }) if value.is_empty())
        );
    }
    assert_eq!(parse_asm("\tdb 1,2,").unwrap(), [1, 2]);
    assert!(parse_asm("\tdb ; no operands\n\tdw\n").unwrap().is_empty());
}

#[test]
fn palette_names_preserve_priority_and_ignore_bank() {
    assert_eq!(
        parse_palette_map("\ttilepal 1, GREEN, PRIORITY_WATER ; x\n").unwrap(),
        [2, 131]
    );
    assert!(parse_palette_map("\ttilepal 0, CYAN").is_err());
}
