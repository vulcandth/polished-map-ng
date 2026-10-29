use polished_map_project::{
    MapDocument, MapSizeOptions, OpenOptions, discover_map, inspect_map_sizes,
};
use std::fs;

#[test]
fn map_size_choices_preserve_source_order_preference_and_boundaries() {
    let options = MapSizeOptions::from_block_count(90);
    assert_eq!(
        options.choices(),
        &[
            (90, 1),
            (45, 2),
            (30, 3),
            (18, 5),
            (15, 6),
            (10, 9),
            (9, 10),
            (6, 15),
            (5, 18),
            (3, 30),
            (2, 45),
            (1, 90)
        ]
    );
    assert_eq!(options.dimensions(), (10, 9));
    assert_eq!(MapSizeOptions::from_block_count(360).dimensions(), (20, 18));
    assert_eq!(MapSizeOptions::from_block_count(6).dimensions(), (3, 2));
    assert_eq!(
        MapSizeOptions::from_block_count(255 * 255).choices(),
        &[(255, 255)]
    );
    for count in [0, 257, 255 * 255 + 1, usize::MAX] {
        let options = MapSizeOptions::from_block_count(count);
        assert!(options.choices().is_empty());
        assert_eq!(options.selected(), None);
        assert_eq!(options.dimensions(), (10, 9));
    }
}

#[test]
fn map_size_metadata_selects_existing_or_appends_mismatched_dimensions_without_writes() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Makefile"), "").unwrap();
    fs::create_dir(dir.path().join("constants")).unwrap();
    fs::write(
        dir.path().join("constants/map_constants.asm"),
        "\tmap_const EXAMPLE, 2, 2\n",
    )
    .unwrap();
    let path = dir.path().join("Example.blk");
    fs::write(&path, [1, 2, 3, 4, 5, 6]).unwrap();
    let metadata = discover_map(&path, None).unwrap();
    let options = inspect_map_sizes(&path, metadata.dimensions).unwrap();
    assert_eq!(options.choices(), &[(6, 1), (3, 2), (2, 3), (1, 6), (2, 2)]);
    assert_eq!(options.dimensions(), (2, 2));
    let options = options.with_preferred(3, 2).unwrap();
    assert_eq!(options.selected(), Some(1));
    assert_eq!(options.choices().len(), 5);
    assert!(options.clone().with_preferred(0, 1).is_err());
    assert!(options.with_preferred(256, 1).is_err());
    assert_eq!(fs::read(&path).unwrap(), [1, 2, 3, 4, 5, 6]);
    let renamed = dir.path().join("Example.1x6.blk");
    fs::copy(path, &renamed).unwrap();
    let metadata = discover_map(&renamed, None).unwrap();
    assert_eq!(
        inspect_map_sizes(&renamed, metadata.dimensions)
            .unwrap()
            .dimensions(),
        (1, 6)
    );
}

#[test]
fn map_size_asm_uses_decoded_bytes_and_headless_open_keeps_ambiguity_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Shared.MAP");
    let source = "Label:\n\tdb 1, 2 ; two bytes\n\tdw $0403, $0605\n";
    fs::write(&path, source).unwrap();
    let options = inspect_map_sizes(&path, None).unwrap();
    assert_eq!(options.block_count(), 6);
    assert_eq!(options.dimensions(), (3, 2));
    assert!(matches!(
        MapDocument::open(&path, &OpenOptions::default()),
        Err(polished_map_project::ProjectError::DimensionsRequired)
    ));
    let document = MapDocument::open(
        &path,
        &OpenOptions {
            width: Some(3),
            height: Some(2),
        },
    )
    .unwrap();
    assert_eq!(document.map().blocks(), &[1, 2, 3, 4, 5, 6]);
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    fs::write(&path, "\tdb unknown\n").unwrap();
    assert!(inspect_map_sizes(&path, None).is_err());
    assert!(inspect_map_sizes(&dir.path().join("missing.blk"), None).is_err());
}
