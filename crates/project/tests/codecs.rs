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
    for n in [0, 1, 15, 17, 4097] {
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
fn assembly_respects_indentation_numbers_comments_and_little_endian() {
    let text = "label:\n\tdb $12, %1010, 0FFh ; comment\r\n DW $1234, 256\ndb 99\n";
    assert_eq!(parse_asm(text).unwrap(), [18, 10, 255, 52, 18, 0, 1]);
    assert!(parse_asm("\tdb 256").is_err());
    assert!(parse_asm("\tdb unknown").is_err());
}

#[test]
fn palette_names_preserve_priority_and_ignore_bank() {
    assert_eq!(
        parse_palette_map("\ttilepal 1, GREEN, PRIORITY_WATER ; x\n").unwrap(),
        [2, 131]
    );
    assert!(parse_palette_map("\ttilepal 0, CYAN").is_err());
}
