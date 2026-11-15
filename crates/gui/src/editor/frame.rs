//! Main-frame scheduling, keyboard routing and view composition. LGPL-3.0-only.
use super::*;

impl Editor {
    /// Advance pending editor work and draw one embedded frame.
    /// Native launchers also use the `eframe::App` adapter for OS dispatch; direct
    /// callers can use this surface for deterministic egui interaction tests.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.poll_workers(&ctx);
        self.poll_event_scripts();
        self.advance_save();
        if ctx.input(|i| i.viewport().close_requested()) && self.interaction_blocked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        } else if ctx.input(|i| i.viewport().close_requested())
            && self.is_modified()
            && !self.exit_approved
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.request_exit(&ctx);
        }
        self.external_file_drops(&ctx, ui.max_rect());
        let blocked = self.interaction_blocked();
        if !blocked {
            menu_popup::begin(&ctx);
        }
        if !blocked && !ctx.egui_wants_keyboard_input() {
            block_hotkeys::normalize_shifted_digits(&ctx);
            self.block_hotkey_input(&ctx);
            if ctx.input_mut(|i| {
                shortcuts::consume_exact(i, egui::Modifiers::COMMAND, egui::Key::Slash)
            }) {
                self.about = Some(Default::default());
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F1)) {
                self.help_window.open();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
                fullscreen::toggle(&ctx);
            }
            if ctx.input_mut(|i| shortcuts::consume_exact(i, egui::Modifiers::NONE, egui::Key::F9))
            {
                self.dark = !self.dark;
                self.apply_theme_choice(&ctx);
            }
            if ctx.input_mut(|i| shortcuts::consume_exact(i, egui::Modifiers::NONE, egui::Key::F10))
            {
                self.transparency.toggle();
            }
            if ctx.input_mut(|i| {
                // egui's logical match ignores extra Shift/Alt modifiers.
                // Command+Shift+P belongs to Show Priority, even when disabled.
                shortcuts::consume_exact(i, egui::Modifiers::COMMAND, egui::Key::P)
            }) {
                self.prepare_print();
            }
            self.recent_shortcuts(&ctx);
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Plus,
                ) || i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Equals,
                ) || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Equals)
                    || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Plus)
            }) {
                self.zoom = !self.zoom;
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::G)) {
                self.grid = !self.grid;
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::I)) {
                self.ids = !self.ids;
            }
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Num4,
                )
            }) {
                self.hex = !self.hex;
            }
            if (self.allow_priority || self.storage() == documents::Storage::Attributed)
                && ctx.input_mut(|i| {
                    i.consume_key(
                        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                        egui::Key::P,
                    )
                })
            {
                self.show_priority = !self.show_priority;
            }
            self.event_shortcuts(&ctx);
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::W)) {
                self.request_close_map();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::F4)) {
                self.request_exit(&ctx);
            }
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::L,
                )
            }) {
                self.prepare_palettes();
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::L)) {
                self.choose_palettes(&ctx);
            }
            self.block_shortcuts(&ctx);
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::T)) {
                self.prepare_tileset();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F)) {
                self.prepare_roof();
            }
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::H,
                )
            }) {
                self.request_roof_change();
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::H)) {
                self.request_tileset_change();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::N)) {
                self.choose_new();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O)) {
                self.choose_open();
            }
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::S,
                )
            }) {
                self.save_as();
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
                self.save();
            }
            if let Some(l) = &mut self.loaded {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)) {
                    l.document.map_mut().undo();
                }
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y)) {
                    l.document.map_mut().redo();
                }
            }
        }
        let bounds = ui.max_rect();
        ui.painter()
            .rect_filled(bounds, 0.0, ui.visuals().panel_fill);
        ui.add_enabled_ui(!blocked, |ui| {
            let menu =
                Rect::from_min_size(bounds.min, Vec2::new(bounds.width(), geometry::MENU_HEIGHT));
            ui.scope_builder(egui::UiBuilder::new().max_rect(menu), |ui| {
                self.menu_bar(ui);
            });
            let toolbar = Rect::from_min_size(
                bounds.min + Vec2::new(0.0, geometry::MENU_HEIGHT),
                Vec2::new(bounds.width(), geometry::TOOLBAR_HEIGHT),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(toolbar), |ui| {
                ui.set_clip_rect(toolbar.intersect(ui.clip_rect()));
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(toolbar.shrink2(Vec2::new(0.0, 1.0))),
                    |ui| {
                        ui.horizontal(|ui| self.toolbar(ui));
                    },
                );
            });
            // Controls can change zoom/rulers during this frame. Size content
            // before applying the scaled offset, so the old extent cannot clamp it.
            let layout = geometry::Layout::new(
                bounds.width(),
                bounds.height(),
                self.zoom,
                self.rulers,
                16.0,
            );
            if self.canvas_zoom != self.zoom {
                // update_layout repositions sidebar children at its origin;
                // FLTK derives a zero scroll position from those child bounds.
                self.reset_sidebar_scroll = true;
            }
            let rect = |r: geometry::Rect| {
                Rect::from_min_size(
                    bounds.min + Vec2::new(r.x, r.y),
                    Vec2::new(r.width, r.height),
                )
            };
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(rect(layout.sidebar)),
                |ui| {
                    self.sidebar(ui, layout.block_size);
                },
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect(layout.canvas)), |ui| {
                self.canvas(ui, &layout);
            });
            self.draw_rulers(ui, &layout, bounds.min);
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect(layout.status)), |ui| {
                self.draw_status_bar(ui);
            });
        });
        if ctx.input(|i| i.pointer.any_released()) && !self.stroke.is_empty() {
            let points = std::mem::take(&mut self.stroke);
            self.apply(Command::Paint {
                points,
                block: self.selected,
            });
        }
        self.help_window.show(&ctx, !blocked);
        self.dialogs(&ctx);
        self.sync_window_title(&ctx);
    }
}

impl eframe::App for Editor {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.store_preferences(storage);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        // Main_Window::exit_cb persists preferences only on exit.
        std::time::Duration::MAX
    }

    fn persist_egui_memory(&self) -> bool {
        // Document scroll, widget focus and transient modal state are not settings.
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_startup_theme(ui.ctx());
        artwork::install_window_icons(ui.ctx());
        self.apply_startup_window_state(ui.ctx());
        self.show(ui);
        #[cfg(not(test))]
        {
            self.dispatch_event_script(ui.ctx(), polished_map_platform::ScriptOpenRequest::open);
            self.dispatch_print(ui.ctx(), printing::choose_export, printing::run);
            self.dispatch_native_transparency(ui.ctx(), |opacity| {
                polished_map_platform::set_window_opacity(_frame, opacity)
            });
        }
        #[cfg(windows)]
        self.choose_palette_color(ui.ctx(), _frame);
    }
}
