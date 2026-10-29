use super::*;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::{ColorSlot, PaletteCommand, PaletteSet};

fn toggle(h: &mut Harness<'_, Editor>) {
    h.get_by_label("Options").click();
    h.run();
    h.get_by_label("Drag and Drop").click();
    h.run();
    h.key_press(egui::Key::Escape);
    h.run();
}

fn click(h: &mut Harness<'_, Editor>, pos: Pos2) {
    h.hover_at(pos);
    for pressed in [true, false] {
        h.event_modifiers(
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Modifiers::NONE,
        );
    }
    h.run();
}

fn drag(h: &mut Harness<'_, Editor>, from: Pos2, to: Pos2, ctrl: bool) {
    h.hover_at(from);
    h.drag_at(from);
    h.run();
    h.hover_at(to);
    h.run();
    let modifiers = if ctrl {
        egui::Modifiers::CTRL
    } else {
        egui::Modifiers::NONE
    };
    h.event_modifiers(
        egui::Event::PointerButton {
            pos: to,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers,
        },
        modifiers,
    );
    h.run();
}

#[test]
fn drag_option_gates_sidebar_transfers_but_keeps_click_clipboard_and_map_painting() {
    for enabled in [false, true] {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        if !enabled {
            toggle(&mut h);
        }
        assert_eq!(h.state().drag_and_drop, enabled);
        drag(&mut h, Pos2::new(16.0, 63.0), Pos2::new(48.0, 63.0), false);
        let loaded = h.state().loaded.as_ref().unwrap();
        let blocks = loaded.blockset.as_ref().unwrap().blockset();
        assert_eq!(blocks.blocks()[0], [u8::from(enabled); 16]);
        assert_eq!(blocks.blocks()[1], [u8::from(!enabled); 16]);
        assert_eq!(blocks.is_modified(), enabled);
        assert!(!loaded.document.map().is_modified());
        assert!(h.state().sidebar_drag.is_none());
        if !enabled {
            click(&mut h, Pos2::new(48.0, 63.0));
            h.run();
            assert_eq!(h.state().selected, 1);
            h.input_mut().events.push(egui::Event::Copy);
            h.run();
            click(&mut h, Pos2::new(16.0, 63.0));
            h.run();
            h.input_mut().events.push(egui::Event::Paste(String::new()));
            h.run();
            assert_eq!(
                h.state()
                    .loaded
                    .as_ref()
                    .unwrap()
                    .blockset
                    .as_ref()
                    .unwrap()
                    .blockset()
                    .blocks()[0],
                [1; 16]
            );
            click(&mut h, Pos2::new(48.0, 63.0));
            h.run();
            drag(
                &mut h,
                Pos2::new(240.0, 143.0),
                Pos2::new(272.0, 143.0),
                false,
            );
            assert!(
                h.state()
                    .loaded
                    .as_ref()
                    .unwrap()
                    .document
                    .map()
                    .is_modified()
            );
        }
        h.state_mut().close_map();
        assert_eq!(h.state().drag_and_drop, enabled);
    }
}

fn pixels(h: &Harness<'_, Editor>, roof: bool, id: usize) -> [u8; 64] {
    let draft = &h.state().tileset_form.as_ref().unwrap().draft;
    *if roof {
        &draft.roof().unwrap().roof().tileset().tiles()[id]
    } else {
        &draft.tileset().tiles()[id]
    }
    .hues()
}

#[test]
fn drag_option_gates_base_and_roof_tiles_without_disabling_pixel_edits() {
    for roof in [false, true] {
        for enabled in [false, true] {
            let (_dir, mut h) = if roof {
                crate::roof_editor_tests::fixture()
            } else {
                crate::tileset_editor_tests::fixture(false, false)
            };
            if !enabled {
                toggle(&mut h);
            }
            if roof {
                h.state_mut().prepare_roof();
            } else {
                h.state_mut().prepare_tileset();
            }
            h.run();
            let labels = if roof {
                ["Roof tile $0A", "Roof tile $0B"]
            } else {
                ["Tileset tile $00", "Tileset tile $01"]
            };
            let from = h.get_by_label(labels[0]).rect().center();
            let to = h.get_by_label(labels[1]).rect().center();
            let a = pixels(&h, roof, 0);
            let b = pixels(&h, roof, 1);
            drag(&mut h, from, to, false);
            assert_eq!(pixels(&h, roof, 0), if enabled { b } else { a });
            assert_eq!(pixels(&h, roof, 1), if enabled { a } else { b });
            drag(&mut h, to, from, true);
            assert_eq!(pixels(&h, roof, 0), a);
            assert_eq!(pixels(&h, roof, 1), if enabled { a } else { b });
            if !enabled {
                h.get_by_label(labels[0]).click();
                h.run();
                h.get_by_label("Swatch 4").click();
                h.run();
                h.get_by_label("Pixel 0, 0").click();
                h.run();
                assert_eq!(pixels(&h, roof, 0)[0], 3);
            }
            h.get_by_label("Cancel").click();
            h.run();
            assert!(!h.state().is_modified());
        }
    }
}

fn slot(palette: u8, hue: u8) -> ColorSlot {
    ColorSlot {
        set: PaletteSet::Day,
        palette,
        hue,
    }
}
fn color(h: &Harness<'_, Editor>, slot: ColorSlot) -> [u8; 3] {
    h.state()
        .palette_form
        .as_ref()
        .unwrap()
        .draft
        .bank()
        .color(slot)
        .unwrap()
}

#[test]
fn palette_drag_swaps_or_copies_live_rgb8_and_cancels_outside_or_when_disabled() {
    for mono in [false, true] {
        let (_dir, mut h) = crate::tileset_editor_tests::fixture(false, mono);
        let p = if mono { 8 } else { 0 };
        let names = if mono {
            ["MONOCHROME color 1", "MONOCHROME color 2"]
        } else {
            ["GRAY color 1", "GRAY color 2"]
        };
        let bank = h.state().palette_document().bank().clone();
        h.state_mut().prepare_palettes();
        h.run();
        let (a, b) = (slot(p, 0), slot(p, 2));
        h.state_mut()
            .palette_form
            .as_mut()
            .unwrap()
            .draft
            .apply_batch(&[
                PaletteCommand::SetRgb {
                    slot: a,
                    rgb: [3, 127, 249],
                },
                PaletteCommand::SetRgb {
                    slot: b,
                    rgb: [17, 34, 51],
                },
            ])
            .unwrap();
        h.run();
        let from = h.get_by_label(names[0]).rect().center();
        let to = h.get_by_label(names[1]).rect().center();
        drag(&mut h, from, to, false);
        assert_eq!(color(&h, a), [17, 34, 51]);
        assert_eq!(color(&h, b), [3, 127, 249]);
        drag(&mut h, to, from, true);
        assert_eq!(
            color(&h, a),
            [3, 127, 249],
            "drag copy retains RGB8; keyboard Copy intentionally quantizes"
        );
        assert_eq!(color(&h, b), [3, 127, 249]);
        let hash = h
            .state()
            .palette_form
            .as_ref()
            .unwrap()
            .draft
            .content_hash();
        drag(&mut h, from, Pos2::new(799.0, 599.0), false);
        drag(&mut h, from, from, false);
        assert_eq!(
            h.state()
                .palette_form
                .as_ref()
                .unwrap()
                .draft
                .content_hash(),
            hash
        );
        h.get_by_label("Cancel").click();
        h.run();
        assert_eq!(h.state().palette_document().bank(), &bank);
        toggle(&mut h);
        h.state_mut().prepare_palettes();
        h.run();
        let before = h
            .state()
            .palette_form
            .as_ref()
            .unwrap()
            .draft
            .bank()
            .clone();
        let from = h.get_by_label(names[0]).rect().center();
        let to = h.get_by_label(names[1]).rect().center();
        for ctrl in [false, true] {
            drag(&mut h, from, to, ctrl);
        }
        assert_eq!(
            h.state().palette_form.as_ref().unwrap().draft.bank(),
            &before
        );
        h.get_by_label(names[1]).click();
        h.run();
        h.get_by_label("R channel").set_value_accesskit(1.0);
        h.run();
        assert_ne!(
            h.state().palette_form.as_ref().unwrap().draft.bank(),
            &before
        );
        h.get_by_label("OK").click();
        h.run();
        assert!(h.state().palette_document().is_modified());
        assert!(!h.state().is_modified());
    }
}

#[cfg(feature = "visual-tests")]
#[test]
fn asset_drag_feedback_matches_selection_border_geometry() {
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for kind in ["tileset", "roof", "palette"] {
        let (_dir, mut h) = if kind == "roof" {
            crate::roof_editor_tests::fixture()
        } else {
            crate::tileset_editor_tests::fixture(false, false)
        };
        let labels = match kind {
            "tileset" => {
                h.state_mut().prepare_tileset();
                ["Tileset tile $00", "Tileset tile $01"]
            }
            "roof" => {
                h.state_mut().prepare_roof();
                ["Roof tile $0A", "Roof tile $0B"]
            }
            _ => {
                h.state_mut().prepare_palettes();
                ["GRAY color 1", "GRAY color 2"]
            }
        };
        // These drag snapshots use Windows order on every host. The dedicated
        // form geometry tests cover both orders. CI 40816806957 differed only
        // in native-default OK/Cancel labels; do not relax pixel thresholds.
        if let Some(form) = h.state_mut().tileset_form.as_mut() {
            form.windows_buttons = true;
        }
        if let Some(form) = h.state_mut().palette_form.as_mut() {
            form.windows_buttons = true;
        }
        h.run();
        let from = h.get_by_label(labels[0]).rect();
        let to = h.get_by_label(labels[1]).rect();
        h.hover_at(from.center());
        h.drag_at(from.center());
        h.run();
        h.hover_at(to.center());
        h.run();
        let rendered = h.render().unwrap();
        for rect in [from, to] {
            let x = rect.min.x.round() as u32;
            let y = rect.min.y.round() as u32;
            for (inset, expected) in [
                (0, [0, 0, 0, 255]),
                (1, [255, 255, 255, 255]),
                (2, [0, 0, 0, 255]),
            ] {
                assert_eq!(rendered.get_pixel(x + inset, y + inset).0, expected);
            }
        }
        snapshots.add(h.try_snapshot(format!("{kind}-drag-borders")));
    }
    snapshots.unwrap();
}
