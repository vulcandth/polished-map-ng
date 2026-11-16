//! Map canvas rendering and paint input; upstream main-window.cpp/map-buttons.cpp.
//! LGPL-3.0-only. Source revisions are pinned in upstream.lock.toml.
use super::*;

impl Editor {
    pub(super) fn canvas(&mut self, ui: &mut egui::Ui, layout: &geometry::Layout) {
        if self.reset_canvas_scroll || self.initial_canvas_scroll || self.canvas_zoom != self.zoom {
            self.canvas_pan.reset();
        }
        let content = self.loaded.as_ref().map_or(Vec2::ZERO, |loaded| {
            let map = loaded.document.map();
            Vec2::new(f32::from(map.width()) + 6.0, f32::from(map.height()) + 6.0)
                * layout.block_size
        });
        let pan = self
            .canvas_pan
            .update(ui, self.canvas_scroll, content, false);
        if pan.handled {
            self.stroke.clear();
            self.event_ui.drag = None;
        }
        let priority = self
            .show_priority
            .then(|| priority::Textures::get(ui.ctx()));
        self.ruler_cursor = None;
        self.ruler_scroll = Vec2::ZERO;
        self.event_ui.cursor = None;
        self.event_ui.screen = None;
        self.event_ui.hovered_event = None;
        if self.event_ui.mode {
            self.stroke.clear();
        } else {
            self.event_ui.drag = None;
        }
        let Some(loaded) = &self.loaded else {
            self.canvas_zoom = self.zoom;
            self.canvas_scroll = Vec2::ZERO;
            ui.centered_and_justified(|ui| {
                ui.label(if self.drag_and_drop {
                    "Open a map or drag one here to edit it."
                } else {
                    "Open a map to edit it."
                });
            });
            return;
        };
        let map = loaded.document.map();
        let size = layout.block_size;
        let mut command = None;
        let mut event_action = None;
        let mut scroll = egui::ScrollArea::both()
            .id_salt("map")
            .auto_shrink([false, false])
            // At the source minimum window size, zoom/rulers leave less than
            // egui's default 64-point viewport. Keep the bars inside that area.
            .min_scrolled_width(0.0)
            .min_scrolled_height(0.0)
            .scroll_source(egui::scroll_area::ScrollSource {
                drag: egui::scroll_area::DragScroll::Never,
                ..Default::default()
            });
        let mut retained_scroll = self.canvas_scroll;
        if let Some(offset) = pan.offset {
            retained_scroll = offset;
            scroll = scroll
                .scroll_offset(offset)
                .scroll_source(egui::scroll_area::ScrollSource {
                    mouse_wheel: true,
                    ..egui::scroll_area::ScrollSource::NONE
                });
        }
        if self.reset_canvas_scroll || self.initial_canvas_scroll || self.canvas_zoom != self.zoom {
            // egui starts physics against the previous frame's bounds. A source
            // zoom/reset is immediate, even when it exceeds those old bounds.
            ui.spacing_mut().scroll.kinetic.rubber_band = false;
            ui.spacing_mut().scroll.kinetic.decay_time = 0.0;
        }
        if self.reset_canvas_scroll {
            retained_scroll = Vec2::ZERO;
            scroll = scroll.scroll_offset(Vec2::ZERO);
            self.reset_canvas_scroll = false;
            self.initial_canvas_scroll = false;
        } else if self.initial_canvas_scroll {
            retained_scroll = Vec2::new(
                geometry::open_scroll(
                    content.x,
                    layout.canvas.width,
                    size,
                    ui.spacing().scroll.bar_width,
                ),
                geometry::open_scroll(
                    content.y,
                    layout.canvas.height,
                    size,
                    ui.spacing().scroll.bar_width,
                ),
            );
            scroll = scroll.scroll_offset(retained_scroll);
            self.initial_canvas_scroll = false;
        } else if self.canvas_zoom != self.zoom {
            retained_scroll = Vec2::new(
                geometry::zoom_scroll(self.canvas_scroll.x, self.zoom),
                geometry::zoom_scroll(self.canvas_scroll.y, self.zoom),
            );
            scroll = scroll.scroll_offset(retained_scroll);
        }
        let viewport_origin = ui.max_rect().min;
        let output = scroll.show_viewport(ui, |ui, viewport| {
            let extent = Vec2::new(
                (f32::from(map.width()) + 6.0) * size,
                (f32::from(map.height()) + 6.0) * size,
            )
            // FLTK 1.4.5 Fl_Slider::scrollvalue extends the range to include
            // pos + page size. Zooming out can therefore leave blank space at
            // the far edge. Retain only the last committed offset, not rubber
            // band overscroll, and let the range contract as the user returns.
            .max(retained_scroll + viewport.size());
            let (rect, response) = ui.allocate_exact_size(extent, Sense::click_and_drag());
            self.ruler_scroll = viewport_origin - rect.min;
            let clip = ui.clip_rect();
            // Iterate only rows/columns intersecting the viewport, including the map margin.
            let origin = rect.min + Vec2::splat(geometry::MAP_MARGIN * size);
            let min_x = ((clip.left() - origin.x) / size).floor().max(0.0) as usize;
            let min_y = ((clip.top() - origin.y) / size).floor().max(0.0) as usize;
            let max_x = (((clip.right() - origin.x) / size).ceil().max(0.0) as usize)
                .min(usize::from(map.width()));
            let max_y = (((clip.bottom() - origin.y) / size).ceil().max(0.0) as usize)
                .min(usize::from(map.height()));
            for y in min_y..max_y {
                for x in min_x..max_x {
                    let point = Point {
                        x: x as u16,
                        y: y as u16,
                    };
                    let id = if self.stroke.contains(&point) {
                        self.selected
                    } else {
                        map.blocks()[y * usize::from(map.width()) + x]
                    };
                    let cell = Rect::from_min_size(
                        origin + Vec2::new(x as f32 * size, y as f32 * size),
                        Vec2::splat(size),
                    );
                    if let Some(texture) = self.textures.get(usize::from(id)) {
                        ui.painter().image(
                            texture.id(),
                            cell,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    } else {
                        ui.painter()
                            .rect_filled(cell, 0.0, Color32::from_rgb(0xab, 0xcd, 0xef));
                    }
                    if let Some(priority) = &priority {
                        priority.block(
                            ui.painter(),
                            cell,
                            loaded.graphics.block_priorities(id),
                            self.zoom,
                        );
                    }
                    let hovered = !self.event_ui.mode
                        && !pan.handled
                        && ui.is_enabled()
                        && ui.rect_contains_pointer(cell.intersect(clip));
                    map_buttons::decorate(
                        ui.painter(),
                        cell,
                        self.zoom,
                        self.grid,
                        hovered,
                        ui.visuals().dark_mode,
                    );
                    if self.ids {
                        map_labels::draw_id(ui.painter(), cell, id, self.hex, self.zoom, hovered);
                    }
                }
            }
            if let Some(events) = &loaded.events {
                event_action = event_workflows::canvas_events(
                    ui,
                    events,
                    origin,
                    size / 2.0,
                    (map.width(), map.height()),
                    &mut self.event_ui,
                    !pan.handled,
                );
            }
            if ui.is_enabled()
                && !pan.handled
                && ui.rect_contains_pointer(clip)
                && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
            {
                let event = self.event_ui.hovered_event.and_then(|index| {
                    loaded
                        .events
                        .as_ref()?
                        .script()
                        .events()
                        .entries()
                        .get(index)
                        .map(|event| (event.definition().x, event.definition().y))
                });
                self.ruler_cursor = geometry::EventCursor::at(
                    pos.x - origin.x,
                    pos.y - origin.y,
                    size,
                    map.width(),
                    map.height(),
                    event,
                );
                if self.event_ui.mode {
                    self.event_ui.cursor = self.ruler_cursor;
                    event_workflows::draw_event_cursor(ui, origin, size / 2.0, &mut self.event_ui);
                }
            }
            if !self.event_ui.mode
                && !pan.handled
                && let Some(pos) = response
                    .interact_pointer_pos()
                    .or_else(|| response.hover_pos())
            {
                let relative = pos - rect.min;
                if let Some((x, y)) = layout.hit(relative.x, relative.y, map.width(), map.height())
                {
                    let point = Point { x, y };
                    let old =
                        map.blocks()[usize::from(y) * usize::from(map.width()) + usize::from(x)];
                    // Block::handle calls change_block_cb on FL_PUSH, not release.
                    if ui.is_enabled()
                        && response.contains_pointer()
                        && clip.contains(pos)
                        && ui.input(|i| i.pointer.secondary_pressed())
                        && usize::from(old) < self.textures.len()
                    {
                        self.selected = old;
                        self.sidebar_reveal = Some(old);
                        ui.ctx().request_repaint();
                    }
                    let (pressed, down, modifiers) = ui.input(|i| {
                        (
                            i.pointer.primary_pressed(),
                            i.pointer.primary_down(),
                            i.modifiers,
                        )
                    });
                    if pressed && response.hovered() {
                        if modifiers.shift {
                            command = Some(Command::Fill {
                                at: point,
                                block: self.selected,
                            });
                        } else if modifiers.ctrl {
                            command = Some(Command::Replace {
                                from: old,
                                to: self.selected,
                            });
                        } else if modifiers.alt {
                            command = Some(Command::Swap {
                                a: old,
                                b: self.selected,
                            });
                        }
                    }
                    if down
                        && (response.hovered() || response.dragged())
                        && !modifiers.ctrl
                        && !modifiers.shift
                        && !modifiers.alt
                    {
                        if let Some(&last) = self.stroke.last() {
                            append_line(&mut self.stroke, last, point);
                        } else {
                            self.stroke.push(point);
                        }
                    }
                    if response.clicked()
                        && !down
                        && self.stroke.is_empty()
                        && !modifiers.ctrl
                        && !modifiers.shift
                        && !modifiers.alt
                    {
                        command = Some(Command::Paint {
                            points: vec![point],
                            block: self.selected,
                        });
                    }
                }
            }
        });
        self.canvas_zoom = self.zoom;
        self.canvas_scroll = output.state.clamped_offset();
        if let Some(command) = command {
            self.apply(command);
        }
        if let Some(action) = event_action {
            self.event_action(action, ui.ctx());
        }
    }
}

pub(super) fn append_line(points: &mut Vec<Point>, from: Point, to: Point) {
    let dx = i32::from(to.x) - i32::from(from.x);
    let dy = i32::from(to.y) - i32::from(from.y);
    let steps = dx.abs().max(dy.abs());
    for step in 1..=steps {
        let p = Point {
            x: (i32::from(from.x) + dx * step / steps) as u16,
            y: (i32::from(from.y) + dy * step / steps) as u16,
        };
        if !points.contains(&p) {
            points.push(p);
        }
    }
}
