use polished_map_core::{EventCommand, EventKind};
use polished_map_project::{EventDocument, EventScript, ProjectError, event_script_path};
use std::fs;

#[test]
fn all_macros_coordinate_orders_prefixes_and_warp_numbering() {
    let mut source = String::from("Label:\nwarp_event 1, 2, IGNORED, 1\n\twarp MAP_SCRIPT, 1, 2\n");
    for kind in EventKind::ALL {
        source.push_str(&format!(
            "\t{} {}$fe, %11, {}, {} ; retained\n",
            kind.macro_name(),
            if kind.has_prefix() { "SPRITE_RED," } else { "" },
            if kind.id_before_map() {
                "2"
            } else {
                "ROUTE_1_2_B1F"
            },
            if kind.id_before_map() {
                "ROUTE_1_2_B1F"
            } else {
                "2"
            }
        ));
    }
    let script = EventScript::parse(&source).unwrap();
    assert_eq!(script.render(), source);
    assert_eq!(script.events().entries().len(), 22);
    let mut warp_id = 0;
    for (index, event) in script.events().entries().iter().enumerate() {
        let d = event.definition();
        assert_eq!(d.line, index + 4);
        assert_eq!((d.x, d.y), if d.kind.y_first() { (3, -2) } else { (-2, 3) });
        assert!(d.hex_coordinates);
        if d.kind.is_warp() {
            warp_id += 1;
            assert_eq!(event.warp_id(), Some(warp_id));
            assert_eq!(event.warp_destination(), Some(("Route1_2B1F".into(), 2)));
        } else {
            assert_eq!(event.warp_id(), None);
            assert_eq!(event.warp_destination(), None);
        }
    }
}

#[test]
fn edits_preserve_unowned_assembly_and_batch_failure_is_atomic() {
    let source = "; mixed endings\r\nLabel:\n  person_event SPRITE_RED, $ff, 3, FLAGS ; keep\r\n\tdb 4\n\twarp_event 4,5, LOCAL_MAP,1";
    let mut script = EventScript::parse(source).unwrap();
    assert_eq!(script.render(), source);
    let before = script.events().clone();
    assert!(
        script
            .apply(&[
                EventCommand::Move {
                    index: 0,
                    x: 9,
                    y: 8
                },
                EventCommand::Move {
                    index: 2,
                    x: 1,
                    y: 1
                }
            ])
            .is_err()
    );
    assert_eq!(script.events(), &before);
    script
        .apply(&[EventCommand::Move {
            index: 0,
            x: 12,
            y: -6,
        }])
        .unwrap();
    assert_eq!(
        script.render(),
        "; mixed endings\r\nLabel:\n  person_event SPRITE_RED, $fffffffa, $c, FLAGS ; keep\r\n\tdb 4\n\twarp_event 4,5, LOCAL_MAP,1"
    );
    let reopened = EventScript::parse(&script.render()).unwrap();
    assert_eq!(reopened.events().entries()[0].definition().y, -6);
    assert_eq!(reopened.events().entries()[0].definition().x, 12);
    assert_eq!(script.events().local_warp_target(1, "LocalMap"), Some(1));
    assert_eq!(script.events().local_warp_target(1, "AnotherMap"), None);
}

#[test]
fn options_normalize_fields_and_resize_translates_clamps_without_history() {
    let mut script =
        EventScript::parse("\tobject SPRITE_RED, 0, 249, FLAGS\n\tsign 5,5 ; hint\n").unwrap();
    script
        .apply(&[EventCommand::Edit {
            index: 0,
            x: -6,
            y: 9,
            prefix: Some(" SPRITE_BLUE ".into()),
            suffix: Some(" A,\n B ".into()),
        }])
        .unwrap();
    assert_eq!(
        script.events().entries()[0].text(),
        "object SPRITE_BLUE, -6,  9, A,  B"
    );
    script
        .apply(&[EventCommand::Translate {
            block_dx: -10,
            block_dy: i16::MAX,
        }])
        .unwrap();
    for event in script.events().entries() {
        assert_eq!((event.definition().x, event.definition().y), (-6, 249));
    }
    assert!(script.events().is_modified());
    for (width, expected) in [(1, 7), (10, 25), (122, 249), (255, 249)] {
        assert_eq!(polished_map_core::Events::drag_max(width), expected);
    }
    let before = script.render();
    assert!(
        script
            .apply(&[EventCommand::Edit {
                index: 1,
                x: 0,
                y: 0,
                prefix: Some("invalid".into()),
                suffix: None
            }])
            .is_err()
    );
    assert_eq!(script.render(), before);
}

#[test]
fn unsafe_or_unsupported_input_is_rejected_without_rewriting() {
    for args in [
        "WIDTH - 1, 2",
        "1+1, 2",
        "1",
        "1,",
        "1, $",
        "1, 2147483647",
        "1, -7",
        "1, 9999999999999999999999",
    ] {
        let error = EventScript::parse(&format!("\tbg_event {args}\n")).unwrap_err();
        assert!(error.to_string().contains("line 1"), "{error}");
    }
    assert!(EventScript::parse("\tbg_event 1, 2\0").is_err());
    let script = EventScript::parse("\twarp_event $ff, 256, MAP, -1\n").unwrap();
    assert_eq!(
        (
            script.events().entries()[0].definition().x,
            script.events().entries()[0].definition().y
        ),
        (-1, 0)
    );
    assert_eq!(
        script.events().entries()[0].warp_destination(),
        Some(("Map".into(), -1))
    );
}

#[test]
fn field_edits_cannot_shift_coordinates_or_hide_a_warp_from_the_parser() {
    let mut script =
        EventScript::parse("\tobject SPRITE, 1, 2, FLAGS\n\twarp 1, 2, 3, MAP\n").unwrap();
    let before = script.render();
    for prefix in ["SPRITE, 3", "SPRITE ; comment"] {
        assert!(
            script
                .apply(&[EventCommand::Edit {
                    index: 0,
                    x: 4,
                    y: 5,
                    prefix: Some(prefix.into()),
                    suffix: None
                }])
                .is_err()
        );
        assert_eq!(script.render(), before);
    }
    assert!(
        script
            .apply(&[EventCommand::Edit {
                index: 1,
                x: 4,
                y: 5,
                prefix: None,
                suffix: Some("3, MAP, EXTRA".into())
            }])
            .is_err()
    );
    assert_eq!(script.render(), before);
    let mut source = String::new();
    for _ in 0..70 {
        source.push_str("\tbg_event 1, 2, FLAGS\n");
    }
    let mut script = EventScript::parse(&source).unwrap();
    let commands: Vec<_> = (0..70)
        .map(|index| EventCommand::Edit {
            index,
            x: 1,
            y: 2,
            prefix: None,
            suffix: Some("x".repeat(65530)),
        })
        .collect();
    assert!(script.apply(&commands).is_err());
    assert_eq!(script.render(), source);
}

#[test]
fn discovery_and_persistence_preserve_bytes_and_check_external_changes() {
    let dir = tempfile::tempdir().unwrap();
    let dirs = ["data/mapObjects", "data/maps/objects", "SXY", "maps"];
    let original = "Header:\r\n\twarp_event 1, 2, TOWN, 1\r\n\tdb 1\r\n";
    for d in dirs {
        fs::create_dir_all(dir.path().join(d)).unwrap();
        let path = dir.path().join(d).join("Town.asm");
        fs::write(&path, original).unwrap();
        assert_eq!(event_script_path(dir.path(), "Town").unwrap(), path);
    }
    assert!(event_script_path(dir.path(), "../Town").is_err());
    let path = event_script_path(dir.path(), "Town").unwrap();
    let mut doc = EventDocument::open(&path).unwrap();
    doc.save().unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    doc.apply(&[EventCommand::Move {
        index: 0,
        x: 8,
        y: 9,
    }])
    .unwrap();
    let planned = doc.preview_save().unwrap().unwrap();
    fs::write(&path, "external edit").unwrap();
    assert!(matches!(doc.save(), Err(ProjectError::Conflict(_))));
    assert!(doc.script().events().is_modified());
    assert_eq!(fs::read_to_string(&path).unwrap(), "external edit");
    fs::write(&path, original).unwrap();
    doc.save().unwrap();
    assert_eq!(
        doc.source().content_hash().as_deref(),
        Some(planned.content_hash.as_str())
    );
    assert!(!doc.script().events().is_modified());
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "Header:\r\n\twarp_event  8,  9, TOWN, 1\r\n\tdb 1\r\n"
    );
    doc.apply(&[EventCommand::Move {
        index: 0,
        x: 10,
        y: 11,
    }])
    .unwrap();
    doc.save().unwrap();
    assert_eq!(
        EventDocument::open(&path)
            .unwrap()
            .script()
            .events()
            .entries()[0]
            .definition()
            .x,
        10
    );
}
