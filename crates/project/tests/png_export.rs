use polished_map_core::RgbImage;
use polished_map_project::{ProjectError, write_new_png};
use polished_map_project::{prepare_png_export, write_png_export};
use std::fs;

#[test]
fn png_creation_is_complete_and_never_clobbers_an_existing_destination() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("print.png");
    let image = RgbImage {
        width: 2,
        height: 1,
        pixels: vec![1, 2, 3, 252, 253, 254],
    };
    write_new_png(&path, &image).unwrap();
    let bytes = fs::read(&path).unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap().to_rgb8();
    assert_eq!(decoded.dimensions(), (2, 1));
    assert_eq!(decoded.as_raw(), &image.pixels);
    let other = RgbImage {
        width: 1,
        height: 1,
        pixels: vec![0; 3],
    };
    assert!(matches!(
        write_new_png(&path, &other),
        Err(ProjectError::Conflict(_))
    ));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn invalid_images_and_bad_paths_leave_no_partial_png() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("print.png");
    for image in [
        RgbImage {
            width: 0,
            height: 1,
            pixels: vec![],
        },
        RgbImage {
            width: 2,
            height: 1,
            pixels: vec![0; 3],
        },
        RgbImage {
            width: 8161,
            height: 1,
            pixels: vec![0; 8161 * 3],
        },
        RgbImage {
            width: u32::MAX,
            height: u32::MAX,
            pixels: vec![],
        },
    ] {
        assert!(write_new_png(&path, &image).is_err());
        assert!(!path.exists());
    }
    let image = RgbImage {
        width: 1,
        height: 1,
        pixels: vec![0; 3],
    };
    assert!(write_new_png(&dir.path().join("missing/print.png"), &image).is_err());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn observed_png_replacement_detects_changes_and_supports_large_screenshots() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("large.png");
    fs::write(&path, vec![0; 9 * 1024 * 1024]).unwrap();
    let target = prepare_png_export(&path).unwrap();
    assert!(target.exists());
    assert!(target.content_hash().is_some());
    let image = RgbImage {
        width: 1,
        height: 1,
        pixels: vec![12, 34, 56],
    };
    write_png_export(&target, &image).unwrap();
    assert_eq!(
        image::open(&path).unwrap().to_rgb8().as_raw(),
        &image.pixels
    );
    assert!(matches!(
        write_png_export(&target, &image),
        Err(ProjectError::Conflict(_))
    ));
    let target = prepare_png_export(&path).unwrap();
    fs::write(&path, b"external edit").unwrap();
    assert!(matches!(
        write_png_export(&target, &image),
        Err(ProjectError::Conflict(_))
    ));
    assert_eq!(fs::read(&path).unwrap(), b"external edit");
    let path = dir.path().join("new.png");
    let target = prepare_png_export(&path).unwrap();
    fs::write(&path, b"external creation").unwrap();
    assert!(matches!(
        write_png_export(&target, &image),
        Err(ProjectError::Conflict(_))
    ));
    assert_eq!(fs::read(&path).unwrap(), b"external creation");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn screenshot_suffix_matches_source_and_does_not_write_during_preparation() {
    let dir = tempfile::tempdir().unwrap();
    for (name, expected, added) in [
        ("shot", "shot.png", true),
        ("shot.PNG", "shot.PNG", false),
        ("shot.other", "shot.other", false),
        (".shot", ".shot", false),
    ] {
        let target = prepare_png_export(&dir.path().join(name)).unwrap();
        assert_eq!(target.path().file_name().unwrap(), expected);
        assert_eq!(target.extension_added(), added);
        assert!(!target.exists());
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}
