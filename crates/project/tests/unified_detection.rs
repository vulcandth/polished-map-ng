//! Pinned external projects; never copy game graphics into this repository.
use polished_map_project::{FormatDetectionError, ProjectFormat, detect_map_format};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn check_project(root: &Path, extension: &str, format: ProjectFormat) -> (usize, usize) {
    let mut count = 0;
    let mut needs_tileset = 0;
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
                polished_map_project::BlocksetDocument::open_in_project(root, tileset).unwrap();
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
