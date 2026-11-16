//! Event markers, labels and pointer gestures. LGPL-3.0-only.
use super::*;

pub(in crate::main_window) fn drag_coordinate(position: f32, cell: f32, dimension: u8) -> i16 {
    let coordinate = (position / cell).trunc() as i32 - i32::from(position < 0.0);
    coordinate.clamp(-6, i32::from(Events::drag_max(dimension))) as i16
}

fn marker_colors(kind: EventKind) -> (Color32, Color32) {
    // Original embedded PNGs contain a one-pixel border and flat interior,
    // both at alpha 128. Reconstruct geometry; do not ship bitmap assets.
    let (a, b) = kind.marker_rgb();
    (
        Color32::from_rgba_unmultiplied(a[0], a[1], a[2], 128),
        Color32::from_rgba_unmultiplied(b[0], b[1], b[2], 128),
    )
}

/// Event selection and guide dispatch; painting lives in `crate::gameboy_screen`.
pub(in crate::main_window) fn draw_event_cursor(
    ui: &egui::Ui,
    origin: Pos2,
    cell: f32,
    state: &mut EventUi,
) {
    let Some(cursor) = state.cursor else {
        return;
    };
    if state.hovered_event.is_none() {
        let area = Rect::from_min_size(
            origin + Vec2::new(f32::from(cursor.event_x), f32::from(cursor.event_y)) * cell,
            Vec2::splat(cell),
        );
        map_buttons::selection(ui.painter(), area, cell == 32.0);
    }
    if state.game_boy {
        state.screen = Some(crate::gameboy_screen::draw(
            ui.painter(),
            origin,
            cursor,
            cell,
        ));
    }
}

pub(in crate::main_window) fn draw_warp_label(
    painter: &egui::Painter,
    area: Rect,
    label: polished_map_core::WarpIdLabel,
    zoom: bool,
) {
    if !zoom {
        // Individual 1×1 filled RectShapes are omitted by egui's feathered
        // tessellator. Explicit quads retain exact opaque bitmap pixels.
        let mut mesh = egui::Mesh::default();
        for digit in label.digits() {
            let origin = area.min + Vec2::new(f32::from(digit.x()), f32::from(digit.y()));
            mesh.add_colored_rect(
                Rect::from_min_size(origin, Vec2::new(5.0, 7.0)),
                Color32::WHITE,
            );
            for (y, row) in digit.rows().iter().enumerate() {
                for x in 0..5 {
                    if row & (1 << (4 - x)) != 0 {
                        mesh.add_colored_rect(
                            Rect::from_min_size(
                                origin + Vec2::new(x as f32, y as f32),
                                Vec2::splat(1.0),
                            ),
                            Color32::BLACK,
                        );
                    }
                }
            }
        }
        painter.add(egui::Shape::mesh(mesh));
    } else {
        let text = label.zoomed_text();
        // Upstream uses 11 unless the selected native face is Consolas (12).
        let font = egui::FontId::monospace(fonts::mono_size(painter.ctx(), 11.0));
        let pos = area.right_bottom() - Vec2::new(4.0, 2.0);
        utils::draw_outlined_text(
            painter,
            pos,
            egui::Align2::RIGHT_BOTTOM,
            &text,
            font,
            Color32::BLACK,
            Color32::WHITE,
        );
    }
}

pub(in crate::main_window) fn canvas_events(
    ui: &mut egui::Ui,
    doc: &EventDocument,
    origin: Pos2,
    cell: f32,
    dimensions: (u8, u8),
    state: &mut EventUi,
    interact: bool,
) -> Option<EventAction> {
    if !state.mode && !state.show {
        return None;
    }
    let model = doc.script().events();
    let rect = |d: &EventDefinition| {
        Rect::from_min_size(
            origin + Vec2::new(f32::from(d.x) * cell, f32::from(d.y) * cell),
            Vec2::splat(cell),
        )
    };
    let pointer = ui
        .input(|i| i.pointer.hover_pos())
        .filter(|p| ui.clip_rect().contains(*p));
    let hovered =
        if interact && state.mode && ui.is_enabled() && ui.rect_contains_pointer(ui.clip_rect()) {
            pointer.and_then(|p| {
                model
                    .entries()
                    .iter()
                    .rposition(|e| rect(e.definition()).contains(p))
            })
        } else {
            None
        };
    let map_name = doc
        .source()
        .path()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let destination = hovered.and_then(|i| model.local_warp_target(i, map_name));
    state.hovered_event = hovered;
    let mut action = None;
    // FL_PUSH captures the marker before FL_DRAG moves it (upstream event.cpp).
    // Native input can batch a press and movement beyond the marker into one
    // frame, so egui's final-position hit test cannot identify that press.
    if state.mode && interact && ui.is_enabled() {
        let pressed = ui.input(|i| {
            i.events.iter().find_map(|event| match event {
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers,
                } if !modifiers.shift => Some(*pos),
                _ => None,
            })
        });
        if let Some(pos) = pressed
            && ui.clip_rect().contains(pos)
            && ui.ctx().layer_id_at(pos) == Some(ui.layer_id())
        {
            state.drag = model
                .entries()
                .iter()
                .rposition(|event| rect(event.definition()).contains(pos));
        }
    }
    for (index, event) in model.entries().iter().enumerate() {
        let d = event.definition();
        let area = rect(d);
        if !ui.is_rect_visible(area) {
            continue;
        }
        let (inside, border) = marker_colors(d.kind);
        let painter = ui.painter();
        painter.rect_filled(area.shrink(1.0), 0.0, inside);
        for edge in [
            Rect::from_min_size(area.min, Vec2::new(cell, 1.0)),
            Rect::from_min_size(
                Pos2::new(area.left(), area.bottom() - 1.0),
                Vec2::new(cell, 1.0),
            ),
            Rect::from_min_size(area.min + Vec2::new(0.0, 1.0), Vec2::new(1.0, cell - 2.0)),
            Rect::from_min_size(
                area.min + Vec2::new(cell - 1.0, 1.0),
                Vec2::new(1.0, cell - 2.0),
            ),
        ] {
            painter.rect_filled(edge, 0.0, border);
        }
        painter.text(
            area.center(),
            egui::Align2::CENTER_CENTER,
            d.kind.symbol(),
            fonts::sans_bold(painter.ctx(), if cell == 32.0 { 24.0 } else { 14.0 }),
            Color32::WHITE,
        );
        if state.warp_ids
            && let Some(id) = event.warp_id()
            && let Some(label) = polished_map_core::WarpIdLabel::new(id, d.hex_coordinates)
        {
            draw_warp_label(painter, area, label, cell == 32.0);
        }
        if hovered == Some(index) || destination == Some(index) {
            for inset in 0..if cell == 32.0 { 4 } else { 3 } {
                let color = if inset == 0 || inset == if cell == 32.0 { 3 } else { 2 } {
                    Color32::BLACK
                } else if destination == Some(index) {
                    Color32::YELLOW
                } else {
                    Color32::WHITE
                };
                painter.rect_stroke(
                    area.shrink(inset as f32),
                    0.0,
                    (1.0, color),
                    egui::StrokeKind::Inside,
                );
            }
        }
        if state.mode && interact {
            let response = ui.interact(
                area,
                ui.scope_id().with(("map-event", index)),
                Sense::click_and_drag(),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::Role::Button,
                    ui.is_enabled(),
                    format!("Event {index}: {}", d.kind.macro_name()),
                )
            });
            if response.secondary_clicked() {
                action = Some(EventAction::Options(index));
            }
            // Use modifiers attached to the release: Shift may already have
            // been released by the time this frame is processed.
            let shift_click = ui.input(|i| i.events.iter().any(|event| matches!(event,
                egui::Event::PointerButton {button:egui::PointerButton::Primary,pressed:false,modifiers,..} if modifiers.shift)));
            if response.clicked() && shift_click {
                action = Some(EventAction::Warp(index));
            } else if response.double_clicked() {
                action = Some(EventAction::ViewScript(index));
            }
            response.on_hover_text(event.text());
        }
    }
    if state.mode
        && interact
        && ui.is_enabled()
        && let Some(index) = state.drag
    {
        if action.is_none()
            && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
            && ui.input(|i| !i.modifiers.shift || !i.pointer.primary_down())
        {
            action = Some(EventAction::Move(
                index,
                drag_coordinate(pos.x - origin.x, cell, dimensions.0),
                drag_coordinate(pos.y - origin.y, cell, dimensions.1),
            ));
        }
        if !ui.input(|i| i.pointer.primary_down()) {
            state.drag = None;
        }
    }
    action
}
