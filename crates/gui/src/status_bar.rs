//! master Main_Window::{initialize,update_status,update_event_cursor},
//! widgets.cpp::{Label,Toolbar::draw} and Greybird/Dark frames in themes.cpp.
//! Label's three-point inset follows FLTK 1.4.5 fl_labeltype.cxx; based in part
//! on the work of the FLTK project (see NOTICE.md and LICENSES/FLTK.txt).
//! Rust adaptation: LGPL-3.0-only. Bundled font metrics remain an approximation.
use super::*;

const SAMPLES: [&str; 5] = [
    "Blocks: 999",
    "Map: 999 x 999",
    "ID: $99",
    "X/Y ($99, $99)",
    "Event: X/Y ($999, $999)",
];

fn field_rects(bounds: Rect, text_widths: [f32; 5]) -> [Rect; 5] {
    // Toolbar frame has a one-point inset. Each label is 21 high and keeps
    // text_width(sample, 8); only the final empty spacer absorbs window resize.
    let mut x = bounds.left() + 1.0;
    text_widths.map(|width| {
        let rect = Rect::from_min_size(
            Pos2::new(x, bounds.top() + 1.0),
            Vec2::new(width.ceil() + 16.0, 21.0),
        );
        x = rect.right() + 2.0;
        rect
    })
}

fn hover_fields(id: u8, cursor: geometry::EventCursor, hex: bool, events: bool) -> [String; 3] {
    let (id, xy, event) = if hex {
        (
            format!("ID: ${id:02X}"),
            format!("X/Y (${:X}, ${:X})", cursor.block_x, cursor.block_y),
            format!("Event: X/Y (${:X}, ${:X})", cursor.event_x, cursor.event_y),
        )
    } else {
        (
            format!("ID: {id}"),
            format!("X/Y ({}, {})", cursor.block_x, cursor.block_y),
            format!("Event: X/Y ({}, {})", cursor.event_x, cursor.event_y),
        )
    };
    [id, xy, if events { event } else { String::new() }]
}

impl Editor {
    fn status_fields(&self) -> [String; 5] {
        let mut fields = std::array::from_fn(|_| String::new());
        let Some(loaded) = &self.loaded else {
            return fields;
        };
        let map = loaded.document.map();
        fields[0] = format!("Blocks: {}", loaded.graphics.block_count());
        fields[1] = format!("Map: {} x {}", map.width(), map.height());
        if let Some(cursor) = self.ruler_cursor
            && let Some(&id) = map.blocks().get(
                usize::from(cursor.block_y) * usize::from(map.width())
                    + usize::from(cursor.block_x),
            )
        {
            // Source Block IDs change during painting. Our history command is
            // committed on release, so report the same live stroke preview.
            let point = Point {
                x: u16::from(cursor.block_x),
                y: u16::from(cursor.block_y),
            };
            let id = if self.stroke.contains(&point) {
                self.selected
            } else {
                id
            };
            let [id, xy, event] = hover_fields(id, cursor, self.hex, self.event_ui.mode);
            fields[2] = id;
            fields[3] = xy;
            fields[4] = event;
        }
        fields
    }

    pub(super) fn draw_status_bar(&self, ui: &mut egui::Ui) {
        // Keep NG's operation feedback discoverable without replacing the
        // original fixed fields. Errors continue to use their existing dialogs.
        let diagnostic = format!(
            "{}\n{}",
            if self.is_modified() {
                "Modified"
            } else {
                "Saved"
            },
            self.status
        );
        draw(ui, ui.max_rect(), &self.status_fields(), &diagnostic);
    }
}

fn frame(painter: &egui::Painter, rect: Rect, upper: u8, lower: u8) {
    // Exact fl_yxline/fl_xyline corner ownership, drawn as pixel-aligned quads.
    for (min, size, color) in [
        (rect.min, Vec2::new(rect.width() - 1.0, 1.0), upper),
        (rect.min, Vec2::new(1.0, rect.height() - 1.0), upper),
        (
            Pos2::new(rect.left(), rect.bottom() - 1.0),
            Vec2::new(rect.width(), 1.0),
            lower,
        ),
        (
            Pos2::new(rect.right() - 1.0, rect.top()),
            Vec2::new(1.0, rect.height()),
            lower,
        ),
    ] {
        painter.rect_filled(
            Rect::from_min_size(min, size),
            0.0,
            Color32::from_gray(color),
        );
    }
}

fn draw(ui: &mut egui::Ui, bounds: Rect, fields: &[String; 5], diagnostic: &str) {
    let clip = bounds.intersect(ui.clip_rect());
    let painter = ui.painter().with_clip_rect(clip);
    let dark = ui.visuals().dark_mode;
    let (background, foreground, upper, lower, separator_upper, separator_lower) = if dark {
        (83, 255, 106, 40, 56, 116)
    } else {
        (206, 60, 218, 193, 186, 218)
    };
    painter.rect_filled(bounds, 0.0, Color32::from_gray(background));
    let font = egui::FontId::proportional(if ui.ctx().os() == egui::os::OperatingSystem::Windows {
        12.0
    } else {
        13.0
    });
    let color = Color32::from_gray(foreground);
    let widths = SAMPLES.map(|sample| {
        painter
            .layout_no_wrap(sample.into(), font.clone(), color)
            .size()
            .x
    });
    let rects = field_rects(bounds, widths);
    ui.interact(bounds, ui.scope_id().with("map-status"), Sense::hover())
        .on_hover_text(diagnostic);
    for (index, (rect, text)) in rects.into_iter().zip(fields).enumerate() {
        let response = ui.interact(
            rect,
            ui.scope_id().with(("status-field", index)),
            Sense::hover(),
        );
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, ui.is_enabled(), text));
        let text_rect = rect.shrink2(Vec2::new(3.0, 0.0));
        let galley = painter.layout_no_wrap(text.clone(), font.clone(), color);
        painter.with_clip_rect(text_rect.intersect(clip)).galley(
            Pos2::new(
                text_rect.left(),
                (text_rect.center().y - galley.size().y / 2.0).floor(),
            ),
            galley,
            color,
        );
        if index < 4 {
            frame(
                &painter,
                Rect::from_min_size(Pos2::new(rect.right(), rect.top()), Vec2::new(2.0, 21.0)),
                separator_upper,
                separator_lower,
            );
        }
    }
    // Toolbar::draw draws its frame after its children, including when the
    // fixed fields extend beyond a narrow window's right edge.
    frame(&painter, bounds, upper, lower);
}

#[cfg(test)]
mod tests;
