use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};
use polished_map_project::{
    OpenOptions, ProjectError, RoofPaletteLayout, plusplus::*, prepare_palette_export,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn write(root: &Path, name: &str, data: impl AsRef<[u8]>) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, data).unwrap();
    path
}
fn rgb_file(count: usize) -> String {
    (0..count)
        .map(|p| {
            let values = (0..4)
                .flat_map(|h| (0..3).map(move |c| ((p * 7 + h * 5 + c * 11) % 32).to_string()))
                .collect::<Vec<_>>();
            format!("\tRGB {}\n", values.join(","))
        })
        .collect()
}
fn slot(palette: u8) -> ColorSlot {
    ColorSlot {
        set: PaletteSet::Day,
        palette,
        hue: 2,
    }
}
fn edit() -> PaletteCommand {
    PaletteCommand::SetRgb {
        slot: slot(0),
        rgb: [7, 123, 249],
    }
}
fn bank_bytes(doc: &PaletteDocument) -> Vec<u8> {
    PaletteSet::ALL
        .into_iter()
        .flat_map(|s| doc.colors(s).into_iter().flatten().flatten())
        .collect()
}

#[test]
fn eight_palette_boundary_and_batch_failures_are_atomic() {
    for command in [
        PaletteCommand::SetRgb {
            slot: slot(8),
            rgb: [0; 3],
        },
        PaletteCommand::Channel {
            slot: slot(8),
            channel: 0,
            value: 1,
        },
        PaletteCommand::Hex {
            slot: slot(8),
            value: "ABCDEF".into(),
        },
        PaletteCommand::Copy {
            from: slot(8),
            to: slot(0),
        },
        PaletteCommand::Copy {
            from: slot(0),
            to: slot(8),
        },
        PaletteCommand::Swap {
            a: slot(8),
            b: slot(0),
        },
        PaletteCommand::Swap {
            a: slot(0),
            b: slot(8),
        },
        PaletteCommand::Channel {
            slot: slot(0),
            channel: 3,
            value: 1,
        },
        PaletteCommand::SetRgb {
            slot: ColorSlot { hue: 4, ..slot(0) },
            rgb: [0; 3],
        },
    ] {
        let mut doc = PaletteDocument::default();
        let hash = doc.content_hash();
        assert!(doc.apply_batch(&[edit(), command]).is_err());
        assert_eq!(doc.content_hash(), hash);
        assert!(!doc.is_modified());
    }
    let doc = PaletteDocument::default();
    assert!(doc.color(slot(8)).is_err());
    assert!(doc.copy_color(slot(8)).is_err());
}

#[test]
fn imports_exports_and_conflicts_preserve_session_boundaries() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "colors.pal", rgb_file(48));
    let mut doc = PaletteDocument::default();
    doc.load(&path).unwrap();
    doc.apply_batch(&[edit()]).unwrap();
    assert_eq!(doc.copy_color(slot(0)).unwrap(), [0, 123, 255]);
    let colors = bank_bytes(&doc);
    let target = prepare_palette_export(&path).unwrap();
    doc.export(&target).unwrap();
    assert_eq!(bank_bytes(&doc), colors);
    assert!(!doc.is_modified());
    doc.verify_source().unwrap();
    let mut reopened = PaletteDocument::default();
    reopened.load(&path).unwrap();
    assert_eq!(reopened.color(slot(0)).unwrap(), [0, 123, 255]);
    assert_eq!(
        fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter(|s| s.starts_with('\t'))
            .count(),
        48
    );
    // The original no-op layout still accepts the import and clears edits.
    for count in 1..=6 {
        let mono = write(dir.path(), "mono.pal", rgb_file(count));
        doc.apply_batch(&[edit()]).unwrap();
        doc.select(PaletteSet::Nite);
        let before = bank_bytes(&doc);
        doc.load(&mono).unwrap();
        assert_eq!(bank_bytes(&doc), before);
        assert_eq!(doc.selected(), PaletteSet::Nite);
        assert!(!doc.is_modified());
    }
    let bad = write(dir.path(), "bad.pal", "\tRGB 32,0,0\n");
    doc.mark_modified();
    let hash = doc.content_hash();
    assert!(doc.load(&bad).is_err());
    assert_eq!(doc.content_hash(), hash);
    assert!(doc.is_modified());
    let dest = prepare_palette_export(&dir.path().join("out")).unwrap();
    fs::write(dest.path(), "external").unwrap();
    assert!(matches!(doc.export(&dest), Err(ProjectError::Conflict(_))));
    assert!(doc.is_modified());
    let other = prepare_palette_export(&dir.path().join("other")).unwrap();
    fs::write(dir.path().join("mono.pal"), "changed").unwrap();
    assert!(matches!(doc.export(&other), Err(ProjectError::Conflict(_))));
    assert!(!other.path().exists());
}

fn map(root: &Path, environment: &str, palette: &str) -> MapDocument {
    write(root, "main.asm", "");
    let path = write(root, "maps/Town.ablk", [0]);
    write(
        root,
        "constants/map_constants.asm",
        "\tnewgroup\n\tmap_const TOWN, 1, 1\n",
    );
    write(
        root,
        "data/maps/maps.asm",
        format!("map Town, TILESET_DEMO, {environment}, LANDMARK, MUSIC, FALSE, {palette}\n"),
    );
    MapDocument::open(&path, &OpenOptions::default()).unwrap()
}

#[test]
fn palette_plan_paths_consent_context_and_roof_failure_are_independent() {
    let dir = tempfile::tempdir().unwrap();
    let canonical = fs::canonicalize(dir.path()).unwrap();
    let root = canonical.as_path();
    let doc = map(root, "TOWN", "PALETTE_NITE");
    let preferred = write(root, "gfx/tilesets/bg_tiles.pal", rgb_file(16));
    write(root, "tilesets/bg.pal", rgb_file(8));
    write(root, "MAPDATA/landmark.pal", rgb_file(8));
    let candidates = [
        "maps/Town.pal",
        "gfx/tilesets/landmark.pal",
        "gfx/tilesets/demo.extra.pal",
        "gfx/tilesets/demo.pal",
    ];
    for name in candidates {
        write(root, name, rgb_file(8));
    }
    for name in candidates {
        let mut plan =
            MapPalettePlan::for_map(&doc, "demo.extra", &PaletteOptions::default()).unwrap();
        let mut colors = PaletteDocument::default();
        colors.mark_modified();
        assert_eq!(plan.next_import_path(), Some(preferred.as_path()));
        plan.advance(&mut colors, false).unwrap();
        assert!(colors.is_modified());
        assert_eq!(plan.next_import_path(), Some(root.join(name).as_path()));
        plan.advance(&mut colors, true).unwrap();
        assert!(!colors.is_modified());
        assert_eq!(colors.selected(), PaletteSet::Custom);
        plan.advance(&mut colors, false).unwrap();
        assert_eq!(colors.selected(), PaletteSet::Custom);
        assert!(plan.next_is_roof_colors());
        let hash = colors.content_hash();
        let next = plan.next_import_path().unwrap().to_path_buf();
        assert!(plan.advance(&mut colors, true).is_err());
        assert_eq!(colors.content_hash(), hash);
        assert_eq!(plan.next_import_path(), Some(next.as_path()));
        colors.mark_modified();
        write(root, "tilesets/roof.pal", rgb_file(2));
        plan.advance(&mut colors, true).unwrap();
        assert!(colors.is_modified());
        assert!(plan.is_finished());
        fs::remove_file(root.join(name)).unwrap();
        fs::remove_file(root.join("tilesets/roof.pal")).unwrap();
    }
    let mut plan = MapPalettePlan::for_map(&doc, "demo.extra", &PaletteOptions::default()).unwrap();
    let mut colors = PaletteDocument::default();
    plan.advance(&mut colors, true).unwrap();
    assert!(plan.next_import_path().is_none()); // MAPDATA ignored.
    plan.advance(&mut colors, true).unwrap();
    assert_eq!(colors.selected(), PaletteSet::Nite);
    fs::remove_file(preferred).unwrap();
    fs::remove_file(root.join("tilesets/bg.pal")).unwrap();
    colors.mark_modified();
    let new = MapDocument::new_map(root, 1, 1).unwrap();
    write(root, "NewMap.pal", rgb_file(8));
    let mut plan = MapPalettePlan::for_map(&new, "demo", &PaletteOptions::default()).unwrap();
    plan.advance(&mut colors, true).unwrap();
    assert!(!colors.is_modified());
    assert!(plan.next_import_path().is_none()); // Unsaved suggested path isn't imported.
    plan.advance(&mut colors, true).unwrap();
    assert!(plan.is_finished());
}

#[test]
fn unified_keeps_legacy_numeric_outdoor_roof_colors_without_changing_parity() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "Makefile", "");
    for environment in ["TOWN", "ROUTE", "1", "2", "INDOOR", "3"] {
        let doc = map(dir.path(), environment, "PALETTE_AUTO");
        for unified in [false, true] {
            let options = PaletteOptions::default();
            let mut plan = if unified {
                MapPalettePlan::for_unified_map(&doc, "demo", &options)
            } else {
                MapPalettePlan::for_map(&doc, "demo", &options)
            }
            .unwrap();
            let mut colors = PaletteDocument::default();
            plan.advance(&mut colors, false).unwrap(); // default import
            plan.advance(&mut colors, false).unwrap(); // contextual selection
            assert_eq!(
                plan.next_is_roof_colors(),
                matches!(environment, "TOWN" | "ROUTE")
                    || (unified && matches!(environment, "1" | "2")),
                "environment {environment}, unified {unified}"
            );
        }
        let classic_path = write(dir.path(), "maps/Town.blk", [0]);
        let classic =
            polished_map_project::MapDocument::open(&classic_path, &OpenOptions::default())
                .unwrap();
        let mut plan =
            polished_map_project::MapPalettePlan::for_map(&classic, "demo", &Default::default())
                .unwrap();
        let mut colors = polished_map_project::PaletteDocument::default();
        plan.advance(&mut colors, false).unwrap();
        plan.advance(&mut colors, false).unwrap();
        assert_eq!(
            plan.next_is_roof_colors(),
            matches!(environment, "TOWN" | "ROUTE" | "1" | "2")
        );
    }
}

#[test]
fn contextual_palette_order_and_explicit_selection_match_source_rules() {
    let dir = tempfile::tempdir().unwrap();
    for (environment, palette, dark, expected) in [
        ("TOWN", "PALETTE_AUTO", false, PaletteSet::Day),
        ("TOWN", "PALETTE_AUTO", true, PaletteSet::Nite),
        ("INDOOR", "PALETTE_NITE", false, PaletteSet::Nite),
        ("GATE", "PALETTE_MORN", false, PaletteSet::Indoor),
        ("6", "4", false, PaletteSet::Nite),
        ("TOWN", "3", false, PaletteSet::Morn),
    ] {
        let doc = map(dir.path(), environment, palette);
        for explicit in [None, Some(PaletteSet::Darkness)] {
            let mut plan = MapPalettePlan::for_map(
                &doc,
                "demo",
                &PaletteOptions {
                    dark_theme: dark,
                    palette: explicit,
                    ..Default::default()
                },
            )
            .unwrap();
            let mut colors = PaletteDocument::default();
            while !plan.is_finished() {
                plan.advance(&mut colors, false).unwrap();
            }
            assert_eq!(colors.selected(), explicit.unwrap_or(expected));
        }
    }
}

#[test]
#[ignore = "requires compiled pinned plusplus palette oracle"]
fn palette_layouts_roofs_and_exports_match_original_functions() {
    let oracle = std::env::var_os("POLISHED_MAP_PLUSPLUS_PALETTE_ORACLE")
        .expect("set POLISHED_MAP_PLUSPLUS_PALETTE_ORACLE");
    let dir = tempfile::tempdir().unwrap();
    let prefix = dir.path().join("reference");
    assert!(
        std::process::Command::new(oracle)
            .arg(&prefix)
            .status()
            .unwrap()
            .success()
    );
    let layouts = fs::read(dir.path().join("reference-layouts.bin")).unwrap();
    let (records, remainder) = layouts.as_chunks::<577>();
    assert!(remainder.is_empty());
    assert_eq!(records.len(), 65 * 6);
    for (i, record) in records.iter().enumerate() {
        let path = write(dir.path(), "input.pal", rgb_file(i / 6));
        let mut doc = PaletteDocument::default();
        doc.select(PaletteSet::ALL[i % 6]);
        doc.load(&path).unwrap();
        assert_eq!(
            doc.selected() as u8,
            record[0],
            "layout {}, selection {}",
            i / 6,
            i % 6
        );
        assert_eq!(
            bank_bytes(&doc),
            &record[1..],
            "layout {}, selection {}",
            i / 6,
            i % 6
        );
    }
    let roofs = fs::read(dir.path().join("reference-roofs.bin")).unwrap();
    let (records, remainder) = roofs.as_chunks::<577>();
    assert!(remainder.is_empty());
    let mut records = records.iter();
    for count in [0, 1, 2, 3, 4, 512] {
        for layout in [
            RoofPaletteLayout::Custom,
            RoofPaletteLayout::DayNite,
            RoofPaletteLayout::MornDayNite,
            RoofPaletteLayout::DayNiteCustom,
            RoofPaletteLayout::MornDayNiteCustom,
        ] {
            for group in [0, 1, 2, 255] {
                let record = records.next().unwrap();
                let path = write(dir.path(), "roof.pal", rgb_file(count));
                let mut doc = PaletteDocument::default();
                doc.mark_modified();
                assert_eq!(
                    doc.load_roof_colors(&path, group, layout).is_ok(),
                    record[0] != 0
                );
                assert_eq!(bank_bytes(&doc), &record[1..]);
                assert!(doc.is_modified());
                assert_eq!(doc.selected(), PaletteSet::Day);
            }
        }
    }
    assert!(records.next().is_none());
    for seed in 0..16 {
        let mut commands = Vec::new();
        for set in PaletteSet::ALL {
            for palette in 0..8 {
                for hue in 0..4 {
                    commands.push(PaletteCommand::SetRgb {
                        slot: ColorSlot { set, palette, hue },
                        rgb: std::array::from_fn(|c| {
                            ((set as usize * 71
                                + usize::from(palette) * 37
                                + usize::from(hue) * 19
                                + c * 13
                                + seed * 31)
                                & 255) as u8
                        }),
                    });
                }
            }
        }
        let mut doc = PaletteDocument::default();
        doc.apply_batch(&commands).unwrap();
        let target = prepare_palette_export(&dir.path().join("actual.pal")).unwrap();
        doc.export(&target).unwrap();
        assert_eq!(
            fs::read(target.path()).unwrap(),
            fs::read(dir.path().join(format!("reference-{seed}.pal"))).unwrap()
        );
    }
}
