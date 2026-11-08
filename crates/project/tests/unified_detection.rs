//! Pinned external projects; never copy game graphics into this repository.
use polished_map_core::{Command, Point};
use polished_map_project::{FormatDetectionError, ProjectFormat, detect_map_format};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn check_project(root: &Path, extension: &str, format: ProjectFormat) -> (usize, usize) {
    let mut count = 0;
    let mut needs_tileset = 0;
    let copies = tempfile::tempdir().unwrap();
    for entry in fs::read_dir(root.join("maps")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != extension) {
            continue;
        }
        let original = fs::read(&path).unwrap();
        let detected = match detect_map_format(&path, None, None) {
            Ok(detected) => detected,
            Err(FormatDetectionError::ContextRequired { .. }) => {
                assert_eq!(format, ProjectFormat::Master);
                needs_tileset += 1;
                continue;
            }
            Err(error) => panic!("{}: {error}", path.display()),
        };
        assert_eq!(detected.format(), format, "{}", path.display());
        let Some(tileset) = detected.metadata().tileset.as_deref() else {
            // An .ablk identifies storage but shared maps can still lack a
            // unique owning header. The GUI must request actual tileset context.
            needs_tileset += 1;
            continue;
        };
        // Resolve the same real companion files the actual loaders consume.
        match format {
            ProjectFormat::Master => {
                verify_master_map(root, &path, tileset, copies.path());
            }
            ProjectFormat::PlusPlus => {
                polished_map_project::plusplus::BlocksetDocument::open_in_project(root, tileset)
                    .unwrap();
            }
        }
        assert_eq!(fs::read(&path).unwrap(), original);
        count += 1;
    }
    assert!(
        count > 200,
        "expected the complete pinned project, got {count} maps"
    );
    (count, needs_tileset)
}

fn verify_master_map(root: &Path, path: &Path, tileset: &str, copies: &Path) {
    use polished_map_project::{
        MapDocument, MasterGraphicsOptions, OpenOptions, load_master_map_graphics, prepare_save_as,
    };
    let metadata = polished_map_project::discover_map(path, Some(root)).unwrap();
    let mut options = OpenOptions::default();
    if let Some((width, height)) = metadata.dimensions
        && usize::from(width) * usize::from(height) != fs::read(path).unwrap().len()
    {
        // Both pinned Gen 1 constants explicitly document this shortened map.
        // Upstream's default open fails too; the user must choose its real size.
        assert_eq!(path.file_name().unwrap(), "UndergroundPathNorthSouth.blk");
        assert_eq!((width, height), (4, 24));
        assert_eq!(fs::read(path).unwrap().len(), 92);
        assert!(MapDocument::open_in_project(path, &options, Some(root)).is_err());
        options = OpenOptions {
            width: Some(4),
            height: Some(23),
        };
    }
    let mut document = MapDocument::open_in_project(path, &options, Some(root))
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let graphics = load_master_map_graphics(&document, tileset, &MasterGraphicsOptions::default())
        .unwrap_or_else(|error| panic!("{} ({tileset}): {error}", path.display()));
    let image = graphics.render_map(document.map()).unwrap();
    assert_eq!(
        image.pixels.len(),
        usize::from(document.map().width()) * usize::from(document.map().height()) * 32 * 32 * 3
    );
    let block = document.map().blocks()[0] ^ 1;
    document
        .map_mut()
        .apply(&Command::Paint {
            points: vec![Point { x: 0, y: 0 }],
            block,
        })
        .unwrap();
    let expected = document.map().blocks().to_vec();
    let target = prepare_save_as(&copies.join(path.file_name().unwrap())).unwrap();
    document.save_as(&target).unwrap();
    assert_eq!(fs::read(target.path()).unwrap(), expected);
    let reopened = MapDocument::open_in_project(target.path(), &options, Some(root)).unwrap();
    assert_eq!(reopened.map().blocks(), expected);
}

#[test]
#[ignore = "requires pinned POLISHED_MAP_POKECRYSTAL and POLISHED_CRYSTAL_FIXTURE"]
fn both_pinned_projects_detect_using_actual_map_context() {
    let master = PathBuf::from(
        std::env::var_os("POLISHED_MAP_POKECRYSTAL").expect("set POLISHED_MAP_POKECRYSTAL"),
    );
    let plusplus = PathBuf::from(
        std::env::var_os("POLISHED_CRYSTAL_FIXTURE").expect("set POLISHED_CRYSTAL_FIXTURE"),
    );
    let master_count = check_project(&master, "blk", ProjectFormat::Master);
    let plusplus_count = check_project(&plusplus, "ablk", ProjectFormat::PlusPlus);
    assert_eq!(master_count, (236, 18));
    assert_eq!(plusplus_count, (418, 34));
    eprintln!(
        "detected (ready, needs tileset) {master_count:?} master maps and {plusplus_count:?} plusplus maps"
    );
}

#[test]
#[ignore = "requires pinned POLISHED_MAP_POKERED, POLISHED_MAP_POKEYELLOW and POLISHED_MAP_POKEGOLD"]
fn red_yellow_gold_detect_using_actual_map_context() {
    for (variable, expected) in [
        ("POLISHED_MAP_POKERED", (202, 23)),
        ("POLISHED_MAP_POKEYELLOW", (203, 23)),
        ("POLISHED_MAP_POKEGOLD", (205, 34)),
    ] {
        let root = PathBuf::from(std::env::var_os(variable).expect("set pinned project fixture"));
        let counts = check_project(&root, "blk", ProjectFormat::Master);
        assert_eq!(counts, expected);
        eprintln!("{variable}: (ready, needs tileset) = {counts:?}");
    }
}
