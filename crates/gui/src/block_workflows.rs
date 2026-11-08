//! Both pinned main-window.cpp copy/paste/swap, select_metatile_cb and add_sub_metatiles.
//! Dialog geometry: Option_Dialog::refresh and Add_Sub_Dialog::refresh_content.
use super::*;
use polished_map_core::plusplus::AttributeCommand;
use polished_map_core::{BlockCommand, CollisionRow};

pub(super) struct BlockClipboard {
    pub(super) id: u8,
    tiles: [u8; 16],
    attributes: Option<[u8; 16]>,
    collision: Option<CollisionRow>,
}

impl Editor {
    pub(super) fn copy_block(&mut self) {
        if let Some(assets) = self.loaded.as_ref().and_then(|l| l.plus_assets.as_ref()) {
            if let Ok(tiles) = assets.blocks.blockset().block(self.selected) {
                self.block_clipboard = Some(BlockClipboard {
                    id: self.selected,
                    tiles: tiles.map(|t| t.offset()),
                    attributes: Some(tiles.map(|t| t.attribute())),
                    collision: assets
                        .blocks
                        .blockset()
                        .offsets()
                        .collisions()
                        .map(|r| r[usize::from(self.selected)].clone()),
                });
            }
            return;
        }
        let Some(blockset) = self.loaded.as_ref().and_then(|l| l.blockset.as_ref()) else {
            return;
        };
        if let Ok(tiles) = blockset.blockset().block(self.selected) {
            self.block_clipboard = Some(BlockClipboard {
                id: self.selected,
                tiles: *tiles,
                attributes: None,
                collision: blockset
                    .blockset()
                    .collisions()
                    .map(|r| r[usize::from(self.selected)].clone()),
            });
        }
    }

    fn paste_block(&mut self, ctx: &egui::Context) {
        let Some(copied) = &self.block_clipboard else {
            return;
        };
        if let Some(attributes) = copied.attributes {
            let mut commands = vec![AttributeCommand::Set {
                block: self.selected,
                offsets: copied.tiles,
                attributes,
            }];
            if let Some(collision) = &copied.collision {
                commands.push(AttributeCommand::SetCollisions {
                    block: self.selected,
                    collision: collision.clone(),
                });
            }
            self.apply_attribute_commands(ctx, &commands);
            return;
        }
        let mut commands = vec![BlockCommand::Set {
            block: self.selected,
            tiles: copied.tiles,
        }];
        if let Some(collision) = &copied.collision {
            commands.push(BlockCommand::SetCollisions {
                block: self.selected,
                collision: collision.clone(),
            });
        }
        self.apply_block_commands(ctx, &commands);
    }

    fn swap_copied_block(&mut self, ctx: &egui::Context) {
        if let Some(copied) = &self.block_clipboard {
            self.apply_block_commands(
                ctx,
                &[BlockCommand::Swap {
                    a: copied.id,
                    b: self.selected,
                }],
            );
        }
    }

    pub(super) fn apply_block_commands(
        &mut self,
        ctx: &egui::Context,
        commands: &[BlockCommand],
    ) -> bool {
        if self
            .loaded
            .as_ref()
            .is_some_and(|l| l.plus_assets.is_some())
        {
            // Only profile-independent operations may use the common command route.
            // A master Set/Paint lacks occurrence attributes and must never flatten them.
            let translated = commands
                .iter()
                .map(|c| match c {
                    BlockCommand::Copy { from, to } => Ok(AttributeCommand::Copy {
                        from: *from,
                        to: *to,
                    }),
                    BlockCommand::Swap { a, b } => Ok(AttributeCommand::Swap { a: *a, b: *b }),
                    BlockCommand::Resize { count } => {
                        Ok(AttributeCommand::Resize { count: *count })
                    }
                    BlockCommand::SetCollisions { block, collision } => {
                        Ok(AttributeCommand::SetCollisions {
                            block: *block,
                            collision: collision.clone(),
                        })
                    }
                    _ => Err("Plusplus block edits require occurrence attributes"),
                })
                .collect::<Result<Vec<_>, _>>();
            return match translated {
                Ok(commands) => self.apply_attribute_commands(ctx, &commands),
                Err(e) => {
                    self.error = Some(e.into());
                    false
                }
            };
        }
        let Some(loaded) = &mut self.loaded else {
            return false;
        };
        let Some(doc) = &mut loaded.blockset else {
            return false;
        };
        // Keep the document and its preview together even if graphics validation fails.
        let mut draft = doc.blockset().clone();
        let MapGraphics::Master(graphics) = &loaded.graphics else {
            self.error = Some("Master block commands require master graphics".into());
            return false;
        };
        let mut graphics = graphics.clone();
        let result = draft
            .apply_batch(commands)
            .map_err(|e| e.to_string())
            .and_then(|()| graphics.replace_blocks(&draft).map_err(|e| e.to_string()))
            .and_then(|()| {
                (0..graphics.block_count())
                    .map(|id| graphics.render_block(id as u8).map_err(|e| e.to_string()))
                    .collect::<Result<Vec<_>, _>>()
            });
        match result {
            Ok(images) => {
                *doc.blockset_mut() = draft;
                loaded.graphics = graphics.into();
                self.textures = images
                    .into_iter()
                    .enumerate()
                    .map(|(id, rgb)| {
                        ctx.load_texture(
                            format!("block-{id}"),
                            egui::ColorImage::from_rgb([32, 32], &rgb.pixels),
                            egui::TextureOptions::NEAREST,
                        )
                    })
                    .collect();
                self.blocks_changed();
                true
            }
            Err(error) => {
                self.error = Some(error);
                false
            }
        }
    }

    pub(super) fn apply_attribute_commands(
        &mut self,
        ctx: &egui::Context,
        commands: &[AttributeCommand],
    ) -> bool {
        let Some(loaded) = &mut self.loaded else {
            return false;
        };
        if loaded.document.profile() != documents::Profile::PlusPlus
            || loaded.graphics.profile() != documents::Profile::PlusPlus
        {
            self.error = Some("Attribute commands require a plusplus map and renderer".into());
            return false;
        }
        let Some(assets) = &mut loaded.plus_assets else {
            self.error = Some("Attribute commands require plusplus assets".into());
            return false;
        };
        let result = (|| {
            let mut draft = assets.blocks.blockset().clone();
            draft.apply_batch(commands).map_err(|e| e.to_string())?;
            let graphics = assets.graphics_for_blocks(&draft, &assets.palettes);
            let textures = documents::block_textures(ctx, &graphics)?;
            Ok::<_, String>((draft, graphics, textures))
        })();
        match result {
            Ok((draft, graphics, textures)) => {
                *assets.blocks.blockset_mut() = draft;
                loaded.graphics = graphics;
                self.textures = textures;
                self.blocks_changed();
                true
            }
            Err(e) => {
                self.error = Some(e);
                false
            }
        }
    }

    fn blocks_changed(&mut self) {
        let count = self.textures.len();
        self.block_hotkeys.prune(count);
        self.sidebar_reveal = self.sidebar_reveal.filter(|&id| usize::from(id) < count);
        if usize::from(self.selected) >= count {
            self.selected = 0;
            self.reset_sidebar_scroll = true;
        }
        if self
            .block_clipboard
            .as_ref()
            .is_some_and(|c| usize::from(c.id) >= count)
        {
            self.block_clipboard = None;
        }
        self.status = format!("Blockset changed ({count} blocks)");
    }

    pub(super) fn prepare_block_count(&mut self) {
        self.block_count_form = self
            .loaded
            .as_ref()
            .filter(|l| l.blockset.is_some() || l.plus_assets.is_some())
            .map(|l| l.graphics.block_count() as u16);
        if let Some(count) = self.block_count_form {
            self.block_count_default = default_value::DefaultValue::new(count);
        }
    }

    pub(super) fn block_shortcuts(&mut self, ctx: &egui::Context) {
        // eframe translates OS clipboard shortcuts to these events; the clipboard
        // contents are intentionally private, as in master, and never parsed as blocks.
        let (paste, cut) = ctx.input(|i| {
            (
                i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::Paste(_) | egui::Event::PasteImage(_))),
                i.events.iter().any(|e| matches!(e, egui::Event::Cut)),
            )
        });
        if ctx.input_mut(shortcuts::consume_copy) {
            self.copy_block();
        }
        if paste || ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::V)) {
            self.paste_block(ctx);
            self.paste_shortcut_handled = true;
        }
        // egui-winit consumes Cmd/Ctrl+V key-down even when the OS clipboard is
        // empty. Its key-up still reaches us. Use that as a fallback for our
        // private clipboard without replacing the user's system clipboard.
        if let Some(command) = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Key {
                    key: egui::Key::V,
                    pressed: false,
                    modifiers,
                    ..
                } => Some(modifiers.command),
                _ => None,
            })
        }) {
            if command && !self.paste_shortcut_handled {
                self.paste_block(ctx);
            }
            self.paste_shortcut_handled = false;
        }
        if cut || ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::X)) {
            self.swap_copied_block(ctx);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::B)) {
            self.prepare_block_count();
        }
    }

    pub(super) fn block_menu(&mut self, ui: &mut egui::Ui) {
        let available = self
            .loaded
            .as_ref()
            .is_some_and(|l| l.blockset.is_some() || l.plus_assets.is_some());
        let copied = available && self.block_clipboard.is_some();
        for (label, enabled, action) in [
            ("Copy Block", available, 0),
            ("Paste Block", copied, 1),
            ("Swap Block", copied, 2),
        ] {
            if menu_popup::action(ui, enabled, label) {
                match action {
                    0 => self.copy_block(),
                    1 => self.paste_block(ui.ctx()),
                    _ => self.swap_copied_block(ui.ctx()),
                }
            }
        }
    }

    pub(super) fn block_count_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut count) = self.block_count_form.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        dialog_host::client(ctx, egui::Id::unique("resize-blockset"), |ui| {
            dialog_host::heading(ui, "Resize Blockset");
            // Original client area is 194 × (10 + 22 + 16 + 22 + 10) = 194 × 80.
            let (area, _) = ui.allocate_exact_size(Vec2::new(194.0, 80.0), Sense::hover());
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            let font = egui::FontId::proportional(if self.block_count_windows_buttons {
                12.0
            } else {
                13.0
            });
            ui.style_mut().override_font_id = Some(font.clone());
            let text = ui.visuals().text_color();
            let width = |s: &str| {
                ui.painter()
                    .layout_no_wrap(s.into(), font.clone(), text)
                    .size()
                    .x
                    .ceil()
            };
            let offset = 10.0 + width("Blocks:") + 4.0;
            let spinner_width = width("999") + 4.0 + 22.0;
            map_options::label(ui, rect(11.0, 10.0, offset - 14.0, 22.0), "Blocks:");
            let response = ui.put(
                rect(offset, 10.0, spinner_width, 22.0),
                numeric_input::IntegerInput::new(&mut count, 1..=256),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::SpinButton, true, "Block count")
            });
            self.block_count_default.restore(ui, &response, &mut count);
            let (ok, no) = if self.block_count_windows_buttons {
                (10.0, 104.0)
            } else {
                (104.0, 10.0)
            };
            accept = ui
                .put(rect(ok, 48.0, 80.0, 22.0), egui::Button::new("OK"))
                .clicked();
            cancel = ui
                .put(rect(no, 48.0, 80.0, 22.0), egui::Button::new("Cancel"))
                .clicked();
            accept |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        });
        if accept && !cancel {
            if self.textures.len() != usize::from(count) {
                self.apply_block_commands(ctx, &[BlockCommand::Resize { count }]);
            }
        } else if !cancel {
            self.block_count_form = Some(count);
        }
    }
}
