//! Unified catalog regressions: event syntax is independent of tileset storage.
use polished_map_core::{EventCommand, EventKind};
use polished_map_project::{EventDocument, EventScript, ProjectError, plusplus};
use std::{fs, path::PathBuf};

const MIXED: &str = "; mixed line endings\r\nTown_MapEvents:\n\twarp SCRIPT_MAP, 4, 5 ; script command\r\n\twarp 1, 2, 3, DEST_MAP ; legacy map warp\n\tpokemon_event 4, 5, PIKACHU, FLAGS ; modern object\r\n\ttreebase_left_event_1 6, 7, FLAGS\n\tobject SPRITE_RED, 8, 9, FLAGS\n\twarp_event 10, 11, OTHER_MAP, 1\n\tcustom_macro 12, 13, KEEP\n\tdb 77";

#[test]
fn combined_catalog_preserves_script_warps_and_both_parity_catalogs() {
    let script = EventScript::parse_unified(MIXED).unwrap();
    let entries = script.events().entries();
    assert_eq!(
        entries
            .iter()
            .map(|e| e.definition().kind)
            .collect::<Vec<_>>(),
        [
            EventKind::Warp,
            EventKind::PokemonEvent,
            EventKind::TreebaseLeftEvent1,
            EventKind::Object,
            EventKind::WarpEvent,
        ]
    );
    assert_eq!(entries[0].warp_destination(), Some(("DestMap".into(), 3)));
    assert_eq!(entries[4].warp_id(), Some(2));
    assert_eq!(script.render(), MIXED);
    // Adding the union entry point must not broaden either parity parser.
    assert_eq!(
        EventScript::parse(MIXED).unwrap().events().entries().len(),
        3
    );
    assert_eq!(
        plusplus::parse_event_script(MIXED)
            .unwrap()
            .events()
            .entries()
            .len(),
        3
    );
}

#[test]
fn mixed_macro_edits_save_only_owned_fields_and_keep_conflict_protection() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("Town.asm");
    fs::write(&path, MIXED).unwrap();
    let mut document = EventDocument::open_unified(&path).unwrap();
    document.save().unwrap();
    assert_eq!(fs::read(&path).unwrap(), MIXED.as_bytes());
    document
        .apply(&[
            EventCommand::Move {
                index: 0,
                x: 2,
                y: 3,
            },
            EventCommand::Move {
                index: 1,
                x: 5,
                y: 6,
            },
        ])
        .unwrap();
    document.save().unwrap();
    // The upstream serializer pads changed decimal coordinates to two
    // characters. Untouched records keep their original spacing and endings.
    let expected = MIXED
        .replace("warp 1, 2, 3,", "warp  2,  3, 3,")
        .replace("pokemon_event 4, 5,", "pokemon_event  5,  6,");
    assert_eq!(fs::read_to_string(&path).unwrap(), expected);
    let reopened = EventDocument::open_unified(&path).unwrap();
    assert_eq!(reopened.script().events().entries().len(), 5);
    document
        .apply(&[EventCommand::Move {
            index: 2,
            x: 7,
            y: 8,
        }])
        .unwrap();
    fs::write(&path, "external edit\n").unwrap();
    assert!(matches!(document.save(), Err(ProjectError::Conflict(_))));
    assert_eq!(fs::read(&path).unwrap(), b"external edit\n");
}

#[test]
#[ignore = "requires pinned POLISHED_MAP_POKECRYSTAL and POLISHED_CRYSTAL_FIXTURE"]
fn unified_catalog_reads_both_pinned_script_sets_without_changing_source() {
    for (variable, plus, expected_count) in [
        ("POLISHED_MAP_POKECRYSTAL", false, 388),
        ("POLISHED_CRYSTAL_FIXTURE", true, 607),
    ] {
        let root = PathBuf::from(std::env::var_os(variable).expect("set pinned project fixture"));
        let mut count = 0;
        for entry in fs::read_dir(root.join("maps")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "asm") {
                continue;
            }
            let source = fs::read_to_string(&path).unwrap();
            let parity = if plus {
                plusplus::parse_event_script(&source)
            } else {
                EventScript::parse(&source)
            }
            .unwrap();
            let unified = EventScript::parse_unified(&source)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert_eq!(unified.render(), source);
            for event in parity.events().entries() {
                assert!(
                    unified
                        .events()
                        .entries()
                        .iter()
                        .any(|candidate| candidate.definition() == event.definition()),
                    "{}: missing {:?}",
                    path.display(),
                    event.definition()
                );
            }
            count += 1;
        }
        assert_eq!(count, expected_count);
        eprintln!("{variable}: {count} scripts retain every parity event and exact source");
    }
}
