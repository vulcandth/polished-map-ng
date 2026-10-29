//! Source-derived master Map::read_blocks/read_asm_blocks prefix semantics.
use polished_map_core::{Command, MapError, Point};
use polished_map_project::{MapDocument, OpenOptions, ProjectError, prepare_save_as};
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn prefix_open_reports_decoded_trailing_blocks_and_hashes_the_complete_source() {
    let dir = tempfile::tempdir().unwrap();
    let options = OpenOptions {
        width: Some(2),
        height: Some(1),
    };
    for (name, source) in [
        ("Map.blk", vec![1, 2, 3, 4]),
        (
            "Map.MAP",
            b"\tdb $01, $02, $03, $04 ; keep this source\n".to_vec(),
        ),
    ] {
        let path = dir.path().join(name);
        fs::write(&path, &source).unwrap();
        assert!(matches!(
            MapDocument::open(&path, &options),
            Err(ProjectError::Map(MapError::Length {
                expected: 2,
                actual: 4
            }))
        ));
        let report = MapDocument::open_in_project_with_report(&path, &options, None).unwrap();
        assert_eq!(report.trailing_blocks, 2);
        let mut document = report.document;
        assert_eq!(document.map().blocks(), [1, 2]);
        assert!(!document.map().is_modified());
        assert!(!document.map().can_undo() && !document.map().can_redo());
        assert_eq!(
            document.content_hash(),
            Some(format!("{:x}", Sha256::digest(&source)))
        );
        assert_eq!(fs::read(&path).unwrap(), source);
        document
            .map_mut()
            .apply(&Command::Paint {
                points: vec![Point { x: 0, y: 0 }],
                block: 9,
            })
            .unwrap();
        let target = prepare_save_as(&dir.path().join(format!("{name}.copy.blk"))).unwrap();
        document.save_as(&target).unwrap();
        assert_eq!(fs::read(&path).unwrap(), source);
        assert_eq!(
            MapDocument::open(target.path(), &options)
                .unwrap()
                .map()
                .blocks(),
            [9, 2]
        );
    }
}

#[test]
fn prefix_open_keeps_strict_validation_and_full_source_conflict_detection() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Map.2x1.blk");
    for bytes in [vec![1], vec![1, 2], vec![1, 2, 3]] {
        fs::write(&path, &bytes).unwrap();
        let report = MapDocument::open_in_project_with_report(&path, &OpenOptions::default(), None);
        if bytes.len() < 2 {
            assert!(matches!(
                report,
                Err(ProjectError::Map(MapError::Length {
                    expected: 2,
                    actual: 1
                }))
            ));
        } else {
            let report = report.unwrap();
            assert_eq!(report.trailing_blocks, bytes.len() - 2);
            assert_eq!(report.document.map().blocks(), [1, 2]);
        }
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    let mut document =
        MapDocument::open_in_project_with_report(&path, &OpenOptions::default(), None)
            .unwrap()
            .document;
    document
        .map_mut()
        .apply(&Command::Replace { from: 1, to: 9 })
        .unwrap();
    // Even a change outside the loaded prefix is a conflict.
    fs::write(&path, [1, 2, 4]).unwrap();
    assert!(matches!(document.save(), Err(ProjectError::Conflict(_))));
    assert_eq!(fs::read(&path).unwrap(), [1, 2, 4]);
    assert!(document.map().is_modified());
    let mut document =
        MapDocument::open_in_project_with_report(&path, &OpenOptions::default(), None)
            .unwrap()
            .document;
    document
        .map_mut()
        .apply(&Command::Replace { from: 1, to: 9 })
        .unwrap();
    document.save().unwrap();
    assert_eq!(fs::read(&path).unwrap(), [9, 2]);
    for width in [0, 256, u16::MAX] {
        assert!(matches!(
            MapDocument::open_in_project_with_report(
                &path,
                &OpenOptions {
                    width: Some(width),
                    height: Some(1)
                },
                None
            ),
            Err(ProjectError::Map(MapError::Dimensions { .. }))
        ));
    }
    fs::write(&path, vec![0; 4 * 1024 * 1024 + 1]).unwrap();
    assert!(matches!(
        MapDocument::open_in_project_with_report(&path, &OpenOptions::default(), None),
        Err(ProjectError::Format { .. })
    ));
}

#[test]
fn prefix_open_supports_maximum_dimensions_and_rejects_malformed_assembly() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Largest.255x255.blk");
    fs::write(&path, vec![7; 255 * 255 + 1]).unwrap();
    let report =
        MapDocument::open_in_project_with_report(&path, &OpenOptions::default(), None).unwrap();
    assert_eq!(report.trailing_blocks, 1);
    assert_eq!(report.document.map().blocks(), vec![7; 255 * 255]);
    let path = dir.path().join("Malformed.2x1.map");
    fs::write(&path, "\tdb $01, unknown_expression\n").unwrap();
    assert!(matches!(
        MapDocument::open_in_project_with_report(
            &path,
            &OpenOptions {
                width: Some(2),
                height: Some(1)
            },
            None
        ),
        Err(ProjectError::Format { .. })
    ));
}
