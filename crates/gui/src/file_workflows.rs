//! master Main_Window::{close_cb,exit_cb,modified_filename} and Modal_Dialog.
//! LGPL-3.0-only. Closing a document retains session palettes/view preferences.
use super::*;

impl Editor {
    pub(super) fn request_close_map(&mut self) {
        self.pending_recent = None;
        if self.loaded.is_none() {
            return;
        }
        self.pending_close = true;
        self.pending_exit = false;
        self.pending_new = false;
        self.event_ui.warp_path = None;
        if self.is_modified() {
            self.confirm_discard = true;
        } else {
            self.close_map();
        }
    }

    pub(super) fn request_exit(&mut self, ctx: &egui::Context) {
        self.pending_recent = None;
        self.pending_close = false;
        self.pending_new = false;
        self.event_ui.warp_path = None;
        if self.is_modified() {
            self.pending_exit = true;
            self.confirm_discard = true;
        } else {
            self.pending_exit = false;
            self.exit_approved = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    pub(super) fn close_map(&mut self) {
        // Upstream colors and selected set are global, outside Tileset::clear.
        self.unloaded_palettes = self.palette_document().clone();
        self.loaded = None;
        self.textures.clear();
        self.selected = 0;
        self.block_clipboard = None;
        self.sidebar_drag = None;
        self.paste_shortcut_handled = false;
        self.stroke.clear();
        self.reset_sidebar_scroll = true;
        self.reset_canvas_scroll = true;
        self.canvas_scroll = Vec2::ZERO;
        self.ruler_scroll = Vec2::ZERO;
        self.ruler_cursor = None;
        self.event_ui.clear_document_state();
        self.pending_close = false;
        self.confirm_discard = false;
        self.status = "Map closed".into();
    }

    fn modified_filename(&self) -> String {
        let Some(loaded) = &self.loaded else {
            return "New Map".into();
        };
        let path = if loaded.document.map().is_modified() {
            if !loaded.document.has_backing_file() {
                return "New Map".into();
            }
            loaded.document.path()
        } else if let Some(events) = &loaded.events
            && events.script().events().is_modified()
        {
            events.source().path()
        } else if let Some(tileset) = &loaded.tileset
            && tileset.tileset().is_modified()
        {
            // The first observed graphics file is the main part, before companions.
            return tileset
                .source_files()
                .first()
                .map(|s| basename(s.path()))
                .unwrap_or_else(|| tileset.name().into());
        } else if let Some(roof) = loaded.tileset.as_ref().and_then(|t| t.roof())
            && roof.roof().tileset().is_modified()
        {
            roof.source().path()
        } else if let Some(blockset) = &loaded.blockset {
            blockset.path()
        } else {
            loaded.document.path()
        };
        basename(path)
    }

    pub(super) fn discard_dialog(&mut self, ctx: &egui::Context) {
        let question = if self.pending_exit {
            "Exit anyway?".into()
        } else if self.pending_close {
            "Close it anyway?".into()
        } else if self.pending_new {
            "Create a new map anyway?".into()
        } else if let Some(path) = &self.event_ui.warp_path {
            format!("Warp to {} anyway?", basename(path))
        } else {
            "Open another map anyway?".into()
        };
        let message = format!(
            "{} has unsaved changes!\n\n{question}",
            self.modified_filename()
        );
        let mut accept = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::unique("unsaved")).show(ctx, |ui| {
            ui.heading("Warning");
            ui.label(message);
            ui.horizontal(|ui| {
                for label in if self.discard_windows_buttons {
                    ["OK", "Cancel"]
                } else {
                    ["Cancel", "OK"]
                } {
                    if ui.button(label).clicked() {
                        if label == "OK" {
                            accept = true;
                        } else {
                            cancel = true;
                        }
                    }
                }
            });
            accept |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        });
        if cancel {
            self.pending_recent = None;
            self.confirm_discard = false;
            self.pending_close = false;
            self.pending_exit = false;
            self.pending_new = false;
            self.event_ui.warp_path = None;
        } else if accept {
            self.confirm_discard = false;
            self.continue_after_discard(ctx);
        }
    }
}

fn basename(path: &std::path::Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;
    use polished_map_core::{
        BlockCommand, ColorSlot, EventCommand, PaletteCommand, PaletteSet, TileCommand,
    };
    use polished_map_project::EventDocument;
    use std::{fs, path::Path};

    fn dirty(editor: &mut Editor, root: &Path, domain: &str) {
        if domain == "map" {
            editor.apply(Command::Paint {
                points: vec![Point { x: 0, y: 0 }],
                block: 1,
            });
            return;
        }
        let loaded = editor.loaded.as_mut().unwrap();
        let paint = TileCommand::Paint {
            tile: 0,
            points: vec![Point { x: 0, y: 0 }],
            hue: 3,
        };
        match domain {
            "events" => {
                let path = root.join("Map.asm");
                fs::write(&path, "Map_Events:\n\twarp_event 0, 0, MAP, 1\n").unwrap();
                let mut events = EventDocument::open(&path).unwrap();
                events
                    .apply(&[EventCommand::Move {
                        index: 0,
                        x: 1,
                        y: 0,
                    }])
                    .unwrap();
                loaded.events = Some(events);
            }
            "tiles" => loaded
                .tileset
                .as_mut()
                .unwrap()
                .apply_batch(&[paint])
                .unwrap(),
            "roof" => loaded
                .tileset
                .as_mut()
                .unwrap()
                .roof_mut()
                .unwrap()
                .roof_mut()
                .apply_batch(&[paint])
                .unwrap(),
            "blocks" => loaded
                .blockset
                .as_mut()
                .unwrap()
                .blockset_mut()
                .apply(&BlockCommand::Set {
                    block: 0,
                    tiles: [1; 16],
                })
                .unwrap(),
            _ => panic!("unknown domain"),
        }
    }

    fn edit_palettes(editor: &mut Editor, ctx: &egui::Context) {
        let mut palettes = editor.palette_document().clone();
        palettes
            .apply_batch(&[PaletteCommand::SetRgb {
                slot: ColorSlot {
                    set: PaletteSet::Custom,
                    palette: 0,
                    hue: 0,
                },
                rgb: [17, 34, 51],
            }])
            .unwrap();
        palettes.select(PaletteSet::Custom);
        editor.commit_palettes(ctx, palettes).unwrap();
    }

    fn source_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        fn collect(path: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
            for entry in fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    collect(&path, files);
                } else {
                    files.push((path.clone(), fs::read(path).unwrap()));
                }
            }
        }
        let mut files = Vec::new();
        collect(root, &mut files);
        files.sort_by(|a, b| a.0.cmp(&b.0));
        files
    }

    #[test]
    fn close_clears_document_state_but_preserves_palettes_and_view_preferences() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, true);
        let before = source_bytes(dir.path());
        let ctx = h.ctx.clone();
        edit_palettes(h.state_mut(), &ctx);
        let palette_hash = h.state().palette_document().content_hash();
        let editor = h.state_mut();
        editor.selected = 1;
        editor.copy_block();
        editor.sidebar_drag = Some(1);
        editor.stroke.push(Point { x: 0, y: 0 });
        editor.canvas_scroll = Vec2::new(35.0, 47.0);
        editor.ruler_scroll = editor.canvas_scroll;
        editor.zoom = true;
        editor.rulers = true;
        editor.ids = true;
        editor.hex = true;
        editor.grid = false;
        editor.event_ui.mode = true;
        editor.event_ui.show = false;
        editor.event_ui.warp_ids = false;
        editor.event_ui.auto_load = false;
        editor.event_ui.game_boy = true;
        editor.event_ui.drag = Some(0);
        editor.event_ui.hovered_event = Some(0);
        assert!(
            !editor.is_modified(),
            "palette-only edits do not trigger source unsaved guard"
        );
        editor.request_close_map();
        assert!(editor.loaded.is_none());
        assert!(editor.textures.is_empty());
        assert!(editor.block_clipboard.is_none());
        assert!(editor.sidebar_drag.is_none());
        assert!(editor.stroke.is_empty());
        assert_eq!(editor.selected, 0);
        assert_eq!(editor.canvas_scroll, Vec2::ZERO);
        assert_eq!(editor.ruler_scroll, Vec2::ZERO);
        assert!(editor.reset_canvas_scroll && editor.reset_sidebar_scroll);
        assert!(editor.event_ui.drag.is_none() && editor.event_ui.hovered_event.is_none());
        assert!(editor.zoom && editor.rulers && editor.ids && editor.hex && !editor.grid);
        assert!(editor.event_ui.mode && editor.event_ui.game_boy);
        assert!(!editor.event_ui.show && !editor.event_ui.warp_ids && !editor.event_ui.auto_load);
        assert_eq!(editor.palette_document().content_hash(), palette_hash);
        assert!(editor.palette_document().is_modified() && editor.palette_monochrome());
        assert!(!editor.confirm_discard);
        editor.request_close_map(); // empty close is harmless and retains session state
        assert_eq!(editor.palette_document().content_hash(), palette_hash);
        h.run();
        assert_eq!(source_bytes(dir.path()), before);
        h.state_mut().prepare_palettes();
        h.run();
        assert!(h.state().palette_form.is_some());
    }

    #[test]
    fn every_dirty_domain_warns_and_cancel_preserves_then_accept_closes_without_writes() {
        for (domain, filename) in [
            ("map", "Map.2x1.blk"),
            ("events", "Map.asm"),
            ("tiles", "demo.2bpp"),
            ("roof", "demo.2bpp"),
            ("blocks", "demo_metatiles.bin"),
        ] {
            let (dir, mut h) = crate::roof_editor_tests::fixture();
            dirty(h.state_mut(), dir.path(), domain);
            let before = source_bytes(dir.path());
            assert_eq!(h.state().modified_filename(), filename);
            h.get_by_label("File").click();
            h.run();
            h.get_by_role_and_label(egui::Role::MenuItem, "Close")
                .click();
            h.run();
            h.get_by_label(&format!(
                "{filename} has unsaved changes!\n\nClose it anyway?"
            ));
            assert!(h.state().confirm_discard && h.state().pending_close);
            h.get_by_label("Cancel").click();
            h.run();
            assert!(h.state().loaded.is_some() && h.state().is_modified());
            assert!(!h.state().pending_close && !h.state().confirm_discard);
            h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
            h.run();
            h.get_by_label("OK").click();
            h.run();
            assert!(h.state().loaded.is_none());
            assert_eq!(source_bytes(dir.path()), before, "{domain}");
        }
    }

    #[test]
    fn save_map_writes_only_map_and_leaves_other_domains_dirty_with_history() {
        let (dir, mut h) = crate::roof_editor_tests::fixture();
        for domain in ["map", "events", "tiles", "roof", "blocks"] {
            dirty(h.state_mut(), dir.path(), domain);
        }
        let before = source_bytes(dir.path());
        // The warning reports the first modified domain in upstream priority order.
        assert_eq!(h.state().modified_filename(), "Map.2x1.blk");
        h.get_by_label("File").click();
        h.run();
        h.get_by_role_and_label(egui::Role::MenuItem, "Save Map")
            .click();
        h.run();
        assert!(h.state().error.is_none());
        assert_eq!(fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [1, 1]);
        let loaded = h.state().loaded.as_ref().unwrap();
        assert!(!loaded.document.map().is_modified());
        assert!(loaded.document.map().can_undo());
        assert!(
            loaded
                .events
                .as_ref()
                .unwrap()
                .script()
                .events()
                .is_modified()
        );
        assert!(loaded.tileset.as_ref().unwrap().tileset().is_modified());
        assert!(
            loaded
                .tileset
                .as_ref()
                .unwrap()
                .roof()
                .unwrap()
                .roof()
                .tileset()
                .is_modified()
        );
        assert!(loaded.blockset.as_ref().unwrap().blockset().is_modified());
        assert_eq!(h.state().modified_filename(), "Map.asm");
        for (path, bytes) in before {
            if path.file_name().unwrap() != "Map.2x1.blk" {
                assert_eq!(fs::read(path).unwrap(), bytes);
            }
        }
        // A concurrent map edit must not be overwritten by the dedicated action.
        h.state_mut().apply(Command::Paint {
            points: vec![Point { x: 1, y: 0 }],
            block: 0,
        });
        fs::write(dir.path().join("Map.2x1.blk"), [0, 0]).unwrap();
        h.state_mut().save_map();
        assert!(h.state().error.is_some());
        assert!(h.state().loaded.is_some());
        assert_eq!(fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [0, 0]);
    }

    #[test]
    fn discard_keyboard_buttons_and_exit_commands_follow_source() {
        #[cfg(feature = "visual-tests")]
        let mut snapshots = egui_kittest::SnapshotResults::new();
        for windows in [false, true] {
            let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
            dirty(h.state_mut(), dir.path(), "map");
            h.state_mut().discard_windows_buttons = windows;
            h.key_press_modifiers(egui::Modifiers::ALT, egui::Key::F4);
            h.run();
            assert!(h.state().pending_exit && !h.state().exit_approved);
            h.get_by_label("Map.2x1.blk has unsaved changes!\n\nExit anyway?");
            assert_eq!(
                h.get_by_label("OK").rect().left() < h.get_by_label("Cancel").rect().left(),
                windows
            );
            #[cfg(feature = "visual-tests")]
            {
                h.remove_cursor();
                h.run();
                h.snapshot(if windows {
                    "discard-exit-windows"
                } else {
                    "discard-exit-unix"
                });
            }
            h.key_press(egui::Key::Escape);
            h.run();
            assert!(!h.state().pending_exit && !h.state().exit_approved);
            assert!(h.state().loaded.is_some() && h.state().is_modified());
            h.get_by_label("File").click();
            h.run();
            h.get_by_role_and_label(egui::Role::MenuItem, "Exit")
                .click();
            h.run();
            // Inspect the key-down frame: key_press also runs a release frame,
            // whose FullOutput no longer contains the one-shot Close command.
            h.key_down(egui::Key::Enter);
            h.step();
            assert!(h.state().exit_approved);
            assert!(h.output().viewport_output.values().any(|v| {
                v.commands
                    .iter()
                    .any(|c| matches!(c, egui::ViewportCommand::Close))
            }));
            assert_eq!(fs::read(dir.path().join("Map.2x1.blk")).unwrap(), [0, 1]);
            #[cfg(feature = "visual-tests")]
            snapshots.extend_harness(&mut h);
        }
        let mut editor = Editor::default();
        let ctx = egui::Context::default();
        edit_palettes(&mut editor, &ctx);
        ctx.begin_pass(Default::default());
        editor.request_exit(&ctx);
        let mut output = ctx.end_pass();
        // No renderer is installed for this command-only context.
        output.textures_delta.clear();
        assert!(editor.exit_approved && !editor.confirm_discard);
        assert!(output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Close))
        }));
    }

    #[test]
    fn close_shortcut_respects_modal_guard_and_event_unload_binding() {
        let (dir, mut h) = crate::tileset_editor_tests::fixture(false, false);
        dirty(h.state_mut(), dir.path(), "events");
        h.state_mut()
            .loaded
            .as_mut()
            .unwrap()
            .events
            .as_mut()
            .unwrap()
            .save()
            .unwrap();
        h.key_press_modifiers(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::W,
        );
        h.run();
        assert!(h.state().loaded.as_ref().unwrap().events.is_none());
        h.state_mut().prepare_resize();
        h.run();
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
        h.run();
        assert!(h.state().loaded.is_some());
        assert!(!h.state().confirm_discard);
        h.get_by_label("Cancel").click();
        h.run();
        h.key_press(egui::Key::Escape);
        h.run();
        assert!(!h.state().exit_approved);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::W);
        h.run();
        assert!(h.state().loaded.is_none());
    }
}
