use polished_map_core::{EventCommand, EventKind, plusplus::EVENT_KINDS};
use polished_map_project::{OpenOptions, ProjectError, plusplus::*};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn write(root: &Path, name: &str, data: impl AsRef<[u8]>) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, data).unwrap();
    path
}

#[test]
fn catalogs_remain_explicit_and_preserve_macro_metadata_and_warp_ids() {
    let mut source = String::from(
        "Label:\r\nwarp_event 1, 2, NOT_INDENTED, 1\n\twarp 1, 2, 1, IGNORED\n\tsign 1,2,IGNORED\n\tobject SPRITE,1,2,IGNORED\n\twarp_to 1,2,IGNORED\n",
    );
    for kind in EVENT_KINDS {
        source.push_str(&format!(
            "\t{} {}$fe, %11, {}, {} ; retained\r\n",
            kind.macro_name(),
            if kind.has_prefix() { "SPRITE_RED," } else { "" },
            if kind.id_before_map() {
                "2"
            } else {
                "LOCAL_MAP"
            },
            if kind.id_before_map() {
                "LOCAL_MAP"
            } else {
                "2"
            }
        ));
        assert_eq!(serde_json::to_value(kind).unwrap(), kind.macro_name());
        assert_eq!(
            serde_json::from_value::<EventKind>(serde_json::json!(kind.macro_name())).unwrap(),
            kind
        );
    }
    let script = parse_event_script(&source).unwrap();
    assert_eq!(script.render(), source);
    let events = script.events().entries();
    assert_eq!(events.len(), 26);
    let mut warp_id = 0;
    for (index, (kind, event)) in EVENT_KINDS.into_iter().zip(events).enumerate() {
        let d = event.definition();
        assert_eq!(d.kind, kind);
        assert_eq!(d.line, index + 7);
        assert_eq!((d.x, d.y), if kind.y_first() { (3, -2) } else { (-2, 3) });
        assert!(d.hex_coordinates);
        if kind.is_warp() {
            warp_id += 1;
            assert_eq!(event.warp_id(), Some(warp_id));
            assert_eq!(event.warp_destination(), Some(("LocalMap".into(), 2)));
            assert_eq!(
                script.events().local_warp_target(index, "LocalMap"),
                Some(4)
            );
        } else {
            assert!(event.warp_id().is_none());
            assert!(event.warp_destination().is_none());
        }
    }
    assert_eq!(warp_id, 3);
    let new = &EVENT_KINDS[19..];
    for (kind, symbol) in new.iter().zip(['C', 'B', 'V', '¿', '¡', '?', '!']) {
        assert_eq!(kind.symbol(), symbol);
        assert_eq!(kind.marker_rgb(), ([255, 128, 0], [205, 78, 0]));
    }
    assert_eq!(EventKind::PokemonEvent.symbol(), 'P');
    assert_eq!(
        EventKind::PokemonEvent.marker_rgb(),
        ([0, 128, 255], [0, 78, 205])
    );
    for kind in [
        EventKind::Warp,
        EventKind::Sign,
        EventKind::Object,
        EventKind::WarpTo,
    ] {
        assert!(polished_map_core::plusplus::event_kind_from_macro(kind.macro_name()).is_none());
    }
    for kind in EVENT_KINDS
        .into_iter()
        .filter(|k| !EventKind::ALL.contains(k))
    {
        assert!(EventKind::from_macro(kind.macro_name()).is_none());
        let text = format!("\t{} 1, 2, RETAINED\n", kind.macro_name());
        let master = polished_map_project::EventScript::parse(&text).unwrap();
        assert!(master.events().entries().is_empty());
        assert_eq!(master.render(), text);
    }
}

#[test]
fn edits_preserve_assembly_and_reject_partial_batches_and_malformed_fields() {
    let source = "; header\r\n  pokemon_event $ff, 3, PIKACHU ; keep\r\n\ttreebase_left_event_1 2, 4, OTHER\n\tdb 99\n\twarp 1,2,3,IGNORED";
    let mut script = parse_event_script(source).unwrap();
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
    assert_eq!(script.render(), source);
    assert!(!script.events().is_modified());
    script
        .apply(&[EventCommand::Edit {
            index: 0,
            x: -6,
            y: 249,
            prefix: None,
            suffix: Some(" RAICHU,\n FLAGS ; keep ".into()),
        }])
        .unwrap();
    assert_eq!(
        script.render(),
        "; header\r\n  pokemon_event $fffffffa, $f9, RAICHU,  FLAGS ; keep\r\n\ttreebase_left_event_1 2, 4, OTHER\n\tdb 99\n\twarp 1,2,3,IGNORED"
    );
    let changed = script.render();
    for command in [
        EventCommand::Move {
            index: 1,
            x: 250,
            y: 0,
        },
        EventCommand::Edit {
            index: 1,
            x: 0,
            y: 0,
            prefix: Some("PREFIX".into()),
            suffix: None,
        },
        EventCommand::Edit {
            index: 1,
            x: 0,
            y: 0,
            prefix: None,
            suffix: Some("\0".into()),
        },
    ] {
        assert!(script.apply(&[command]).is_err());
        assert_eq!(script.render(), changed);
    }
    script
        .apply(&[EventCommand::Translate {
            block_dx: i16::MIN,
            block_dy: i16::MAX,
        }])
        .unwrap();
    let reopened = parse_event_script(&script.render()).unwrap();
    for e in reopened.events().entries() {
        assert_eq!((e.definition().x, e.definition().y), (-6, 249));
    }
    for args in ["1", "1,", "$ff, $", "NAME+1, 2", "1, 2147483647", "1,-7"] {
        assert!(parse_event_script(&format!("\tpokemon_event {args}\n")).is_err());
    }
    assert!(parse_event_script("\tpokemon_event 1,2\0").is_err());
    for n in -6_i16..=-1 {
        let source = format!("\tpokemon_event ${:x}, 2, PIKACHU\n", i32::from(n) as u32);
        let mut script = parse_event_script(&source).unwrap();
        assert_eq!(script.events().entries()[0].definition().x, n);
        script
            .apply(&[EventCommand::Move {
                index: 0,
                x: n,
                y: n,
            }])
            .unwrap();
        let reopened = parse_event_script(&script.render()).unwrap();
        let event = reopened.events().entries()[0].definition();
        assert_eq!((event.x, event.y), (n, n));
    }
}

#[test]
fn discovery_fallbacks_and_warp_paths_stay_in_the_map_directory() {
    let dir = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(dir.path()).unwrap();
    write(&root, "main.asm", "");
    let path = write(&root, "custom/Town.1x1.demo.ablk", [0]);
    let map = MapDocument::open(&path, &OpenOptions::default()).unwrap();
    for folder in ["SXY", "data/maps/objects", "data/mapObjects"] {
        write(
            &root,
            &format!("{folder}/Town.asm"),
            "\twarp_event 1, 2, TOWN, 1\n",
        );
    }
    assert_eq!(
        event_script_path(&root, "Town").unwrap(),
        root.join("maps/Town.asm")
    );
    assert!(event_script_for_map(&map).unwrap().is_none());
    let fallback = path.with_extension("asm");
    fs::write(&fallback, "\tpokemon_event 1,2,PIKACHU\n").unwrap();
    assert_eq!(event_script_for_map(&map).unwrap(), Some(fallback.clone()));
    assert_eq!(
        EventDocument::open_for_map(&map)
            .unwrap()
            .unwrap()
            .script()
            .events()
            .entries()[0]
            .definition()
            .kind,
        EventKind::PokemonEvent
    );
    let preferred = write(&root, "maps/Town.asm", "\tcavebase_event 2,3,FLAGS\n");
    assert_eq!(event_script_for_map(&map).unwrap(), Some(preferred.clone()));
    fs::write(&preferred, "\tpokemon_event BAD, 2\n").unwrap();
    assert!(EventDocument::open_for_map(&map).is_err()); // No fallback around a malformed preferred file.
    let unsaved = MapDocument::new_map(&root, 1, 1).unwrap();
    write(&root, "NewMap.asm", "\tpokemon_event 1,2,X\n");
    assert!(EventDocument::open_for_map(&unsaved).unwrap().is_none());
    for ext in ["ablk", "blk", "ABLK"] {
        let current = root.join(format!("custom/Town.10x9.demo.{ext}"));
        assert_eq!(
            warp_map_path(&current, "Route1_2B1F").unwrap(),
            root.join(format!("custom/Route1_2B1F.{ext}"))
        );
    }
    for name in [
        "",
        "../Other",
        "C:Other",
        "Other/Map",
        "Other\\Map",
        "Other\0",
    ] {
        assert!(event_script_path(&root, name).is_err());
        assert!(warp_map_path(&path, name).is_err());
    }
}

#[test]
fn save_previews_and_conflicts_preserve_source_and_dirty_edits() {
    let dir = tempfile::tempdir().unwrap();
    let source = "Header:\r\n\tpokemon_event 1,2,PIKACHU ; keep\r\n\tdb 1\n";
    let path = write(dir.path(), "Town.asm", source);
    let mut doc = EventDocument::open(&path).unwrap();
    assert!(doc.preview_save().unwrap().is_none());
    doc.save().unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    doc.apply(&[EventCommand::Move {
        index: 0,
        x: 8,
        y: 9,
    }])
    .unwrap();
    let planned = doc.preview_save().unwrap().unwrap();
    fs::write(&path, "external").unwrap();
    assert!(matches!(doc.save(), Err(ProjectError::Conflict(_))));
    assert!(doc.script().events().is_modified());
    assert_eq!(fs::read(&path).unwrap(), b"external");
    fs::write(&path, source).unwrap();
    doc.save().unwrap();
    assert_eq!(
        doc.source().content_hash().as_deref(),
        Some(planned.content_hash.as_str())
    );
    assert!(!doc.script().events().is_modified());
    assert_eq!(
        EventDocument::open(&path)
            .unwrap()
            .script()
            .events()
            .entries()[0]
            .definition()
            .x,
        8
    );
    doc.apply(&[EventCommand::Move {
        index: 0,
        x: 10,
        y: 11,
    }])
    .unwrap();
    doc.save().unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "Header:\r\n\tpokemon_event 10, 11,PIKACHU ; keep\r\n\tdb 1\n"
    );
    fs::write(&path, [0xff]).unwrap();
    assert!(EventDocument::open(&path).is_err());
}

#[test]
fn all_event_symbols_print_through_the_shared_plusplus_compositor() {
    use polished_map_core::{
        Map, PrintOptions, PrintStyle, Tile,
        plusplus::{AttributeBlockset, BankOrder, Graphics, RawTileset},
    };
    let graphics = Graphics::new(
        RawTileset::new(vec![Tile::new([0; 64]).unwrap()], vec![true]).unwrap(),
        AttributeBlockset::from_bytes(&[0; 16], &[0; 16]).unwrap(),
        [[[255; 3]; 4]; 8],
        BankOrder::default(),
    );
    let map = Map::blank(13, 1).unwrap();
    let source: String = EVENT_KINDS
        .into_iter()
        .enumerate()
        .map(|(i, k)| {
            let (a, b) = if k.y_first() { (0, i) } else { (i, 0) };
            format!(
                "\t{} {}{a}, {b}, {}, {}\n",
                k.macro_name(),
                if k.has_prefix() { "SPRITE," } else { "" },
                if k.id_before_map() { "1" } else { "MAP" },
                if k.id_before_map() { "MAP" } else { "1" }
            )
        })
        .collect();
    let events = parse_event_script(&source).unwrap();
    let fonts = polished_map_project::PrintFonts::from_faces(
        polished_map_project::PrintFontFace::from_bytes(epaint_default_fonts::HACK_REGULAR, 0)
            .unwrap(),
        polished_map_project::PrintFontFace::from_bytes(epaint_default_fonts::UBUNTU_LIGHT, 0)
            .unwrap(),
    );
    let mut labels = std::collections::BTreeSet::new();
    let output = graphics
        .render_print(
            &map,
            Some(events.events()),
            PrintOptions {
                events: true,
                warp_ids: true,
                ..Default::default()
            },
            PrintStyle::default(),
            |font, text| {
                labels.insert(text.to_owned());
                fonts.rasterize(font, text)
            },
        )
        .unwrap();
    assert_ne!(output, graphics.render_map(&map).unwrap());
    for symbol in ['¿', '¡', '?', '!', 'P', 'V'] {
        assert!(labels.contains(&symbol.to_string()));
    }
}

#[test]
#[ignore = "requires POLISHED_MAP_PLUSPLUS_EVENT_ORACLE"]
fn events_match_original_plusplus_functions_and_metadata() {
    let oracle = std::env::var_os("POLISHED_MAP_PLUSPLUS_EVENT_ORACLE")
        .expect("set POLISHED_MAP_PLUSPLUS_EVENT_ORACLE");
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("events.asm");
    let output = dir.path().join("events.tsv");
    let mut source = String::new();
    for kind in EVENT_KINDS {
        for n in -6i32..=511 {
            for base in [10, 16, 2] {
                let number = |n: i32| match base {
                    // Keep oracle input within signed long on Windows. The
                    // source's own unsigned-looking negative output exceeds
                    // Win32 strtol and invokes signed overflow in parse_coord.
                    16 if n < 0 => format!("$-{positive:x}", positive = -n),
                    16 => format!("${n:x}"),
                    2 if n >= 0 => format!("%{n:b}"),
                    _ => n.to_string(),
                };
                source.push_str(&format!(
                    "\t{} {}{}, {}{}\n",
                    kind.macro_name(),
                    if kind.has_prefix() { "SPRITE_RED," } else { "" },
                    number(n),
                    number(505 - n),
                    if kind.id_before_map() {
                        ", $2, ROUTE_1__2_B1F ; comment"
                    } else {
                        ", ROUTE_1__2_B1F, $2 ; comment"
                    }
                ));
            }
        }
    }
    for kind in EVENT_KINDS.into_iter().filter(|kind| kind.is_warp()) {
        for map in [
            "TOWN",
            "ROUTE_1__2_B1F",
            "SS_ANNE_2F",
            "MAP2_GATE",
            "MAP__1___2",
            "",
        ] {
            for id in ["-1", "0", "1", "$ff", "%11", "SYMBOL", "2+3"] {
                let (a, b) = if kind.id_before_map() {
                    (id, map)
                } else {
                    (map, id)
                };
                source.push_str(&format!("\t{} 1, 2, {a}, {b}\n", kind.macro_name()));
            }
        }
    }
    fs::write(&input, &source).unwrap();
    assert!(
        std::process::Command::new(oracle)
            .arg(input)
            .arg(&output)
            .status()
            .unwrap()
            .success()
    );
    let expected = fs::read_to_string(output).unwrap();
    let script = parse_event_script(&source).unwrap();
    assert_eq!(script.events().entries().len(), 26 * 518 * 3 + 3 * 6 * 7);
    assert_eq!(expected.lines().count(), script.events().entries().len());
    for (line, event) in expected.lines().zip(script.events().entries()) {
        let d = event.definition();
        let (map, id) = event.warp_destination().unwrap_or_default();
        let texture = match d.kind.marker_rgb() {
            ([255, 0, 0], [205, 0, 0]) => 0,
            ([0, 160, 0], [0, 110, 0]) => 1,
            ([48, 48, 255], [0, 0, 205]) => 2,
            ([160, 0, 160], [110, 0, 110]) => 3,
            ([255, 128, 0], [205, 78, 0]) => 4,
            ([0, 128, 255], [0, 78, 205]) => 5,
            _ => panic!("unexpected texture"),
        };
        assert_eq!(
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                d.x,
                d.y,
                u8::from(d.hex_coordinates),
                u8::from(d.kind.is_warp()),
                d.kind.symbol() as u32,
                event.text(),
                map,
                id,
                texture
            ),
            line,
            "line {}",
            d.line
        );
    }
}

#[test]
#[ignore = "requires pinned POLISHED_CRYSTAL_FIXTURE; all writes are temporary"]
fn real_event_scripts_edit_save_reopen_without_changing_unowned_source() {
    let root = PathBuf::from(
        std::env::var_os("POLISHED_CRYSTAL_FIXTURE").expect("set POLISHED_CRYSTAL_FIXTURE"),
    );
    let temp = tempfile::tempdir().unwrap();
    let (mut scripts, mut events) = (0, 0);
    let mut kinds = BTreeMap::<String, usize>::new();
    for entry in fs::read_dir(root.join("maps")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "asm") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let parsed =
            parse_event_script(&source).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(parsed.render(), source);
        scripts += 1;
        events += parsed.events().entries().len();
        if parsed.events().entries().is_empty() {
            continue;
        }
        let commands: Vec<_> = parsed
            .events()
            .entries()
            .iter()
            .enumerate()
            .map(|(index, event)| {
                let d = event.definition();
                *kinds.entry(d.kind.macro_name().into()).or_default() += 1;
                EventCommand::Move {
                    index,
                    x: if d.x == 249 { 248 } else { d.x + 1 },
                    y: if d.y == -6 { -5 } else { d.y - 1 },
                }
            })
            .collect();
        let copy = temp.path().join(path.file_name().unwrap());
        fs::write(&copy, &source).unwrap();
        let mut document = EventDocument::open(&copy).unwrap();
        document.apply(&commands).unwrap();
        document.save().unwrap();
        let reopened = EventDocument::open(&copy).unwrap();
        assert_eq!(
            reopened.script().events().entries().len(),
            parsed.events().entries().len()
        );
        for (old, new) in parsed
            .events()
            .entries()
            .iter()
            .zip(reopened.script().events().entries())
        {
            let mut expected = old.definition().clone();
            expected.x += if expected.x == 249 { -1 } else { 1 };
            expected.y += if expected.y == -6 { 1 } else { -1 };
            assert_eq!(new.definition(), &expected, "{}", path.display());
            assert_eq!(new.warp_destination(), old.warp_destination());
            assert_eq!(new.warp_id(), old.warp_id());
        }
        let event_lines: std::collections::HashSet<_> = parsed
            .events()
            .entries()
            .iter()
            .map(|e| e.definition().line)
            .collect();
        let saved = fs::read_to_string(&copy).unwrap();
        assert_eq!(
            source.split_inclusive('\n').count(),
            saved.split_inclusive('\n').count()
        );
        for (index, (before, after)) in source
            .split_inclusive('\n')
            .zip(saved.split_inclusive('\n'))
            .enumerate()
        {
            if !event_lines.contains(&(index + 1)) {
                assert_eq!(after, before);
            }
        }
        assert_eq!(fs::read_to_string(&path).unwrap(), source);
    }
    eprintln!("verified {scripts} scripts, {events} events; kinds: {kinds:?}");
    assert_eq!((scripts, events), (607, 6518));
    assert_eq!(kinds.get("pokemon_event"), Some(&65));
}
