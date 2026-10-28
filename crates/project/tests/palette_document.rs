use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};
use polished_map_project::{PaletteDocument, ProjectError, prepare_palette_export};
use std::fs;
fn edit(doc: &mut PaletteDocument, rgb: [u8; 3]) {
    doc.apply_batch(&[PaletteCommand::SetRgb {
        slot: ColorSlot {
            set: PaletteSet::Day,
            palette: 0,
            hue: 2,
        },
        rgb,
    }])
    .unwrap();
}

#[test]
fn export_quantizes_without_mutating_draft_and_reload_keeps_all_six_sets() {
    let dir = tempfile::tempdir().unwrap();
    let target = prepare_palette_export(&dir.path().join("colors")).unwrap();
    let mut doc = PaletteDocument::default();
    edit(&mut doc, [3, 127, 249]);
    let expected = doc.bank().clone();
    doc.export(&target, false).unwrap();
    assert_eq!(target.path().extension().unwrap(), "pal");
    assert!(!doc.is_modified());
    assert_eq!(doc.bank(), &expected);
    let mut reopened = PaletteDocument::default();
    reopened.load(target.path()).unwrap();
    for set in PaletteSet::ALL {
        for palette in 0..9 {
            for hue in 0..4 {
                let slot = ColorSlot { set, palette, hue };
                assert_eq!(
                    reopened.bank().color(slot).unwrap(),
                    expected.copy_color(slot).unwrap()
                );
            }
        }
    }
    let mono = prepare_palette_export(&dir.path().join("mono.pal")).unwrap();
    reopened.export(&mono, true).unwrap();
    assert_eq!(fs::read_to_string(mono.path()).unwrap().lines().count(), 6);
}

#[test]
fn import_overlays_sets_and_failure_or_external_changes_preserve_draft_and_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.pal");
    fs::write(&path, "\tRGB 00,01,02, 03,04,05, 06,07,08, 09,10,11\n").unwrap();
    let mut doc = PaletteDocument::default();
    edit(&mut doc, [100; 3]);
    let before = doc.bank().colors(PaletteSet::Day).to_owned();
    doc.load(&path).unwrap();
    assert_eq!(doc.selected(), PaletteSet::Custom);
    assert!(!doc.is_modified());
    assert_eq!(*doc.bank().colors(PaletteSet::Day), before);
    assert_eq!(doc.bank().colors(PaletteSet::Custom)[8][2], [24, 33, 41]);
    let hash = doc.content_hash();
    let bad = dir.path().join("bad.pal");
    fs::write(&bad, "\tRGB 32,0,0\n").unwrap();
    assert!(doc.load(&bad).is_err());
    assert_eq!(doc.content_hash(), hash);
    edit(&mut doc, [0; 3]);
    let target = prepare_palette_export(&dir.path().join("output.pal")).unwrap();
    fs::write(target.path(), "external").unwrap();
    assert!(matches!(
        doc.export(&target, false),
        Err(ProjectError::Conflict(_))
    ));
    assert!(doc.is_modified());
    assert_eq!(fs::read_to_string(target.path()).unwrap(), "external");
    let other = prepare_palette_export(&dir.path().join("other.pal")).unwrap();
    fs::write(&path, "external").unwrap();
    assert!(matches!(
        doc.export(&other, false),
        Err(ProjectError::Conflict(_))
    ));
    assert!(!other.path().exists());
}
