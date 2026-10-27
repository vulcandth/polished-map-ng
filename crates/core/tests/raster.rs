use polished_map_core::{Graphics, Map, Tile, day_palettes};

#[test]
fn block_and_map_pixels_use_native_internal_hue_order() {
    let hues = std::array::from_fn(|i| (i % 4) as u8);
    let palette = [[255; 3], [85; 3], [170; 3], [0; 3]];
    let graphics =
        Graphics::new(vec![Tile::new(hues).unwrap()], vec![palette], vec![[0; 16]]).unwrap();
    let raster = graphics
        .render_map(&Map::from_bytes(2, 1, vec![0, 0]).unwrap())
        .unwrap();
    assert_eq!((raster.width, raster.height), (64, 32));
    assert_eq!(
        &raster.pixels[..12],
        &[255, 255, 255, 85, 85, 85, 170, 170, 170, 0, 0, 0]
    );
    assert_eq!(&raster.pixels[..96], &raster.pixels[96..192]);
}

#[test]
fn original_five_bit_color_expansion_is_exact() {
    assert_eq!(day_palettes()[0][0], [222, 255, 222]);
    assert_eq!(day_palettes()[0][1], [107; 3]);
}

#[test]
fn invalid_references_and_hues_are_errors() {
    assert!(Tile::new([4; 64]).is_err());
    let tile = Tile::new([0; 64]).unwrap();
    assert!(Graphics::new(vec![tile.clone()], vec![day_palettes()[0]], vec![[1; 16]]).is_err());
    let g = Graphics::new(vec![tile], vec![day_palettes()[0]], vec![[0; 16]]).unwrap();
    let missing = g
        .render_map(&Map::from_bytes(1, 1, vec![255]).unwrap())
        .unwrap();
    assert_eq!(missing.pixels, [0xab, 0xcd, 0xef].repeat(32 * 32));
}
