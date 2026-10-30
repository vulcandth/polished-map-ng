//! master event.cpp, main-window.cpp and Event_Options_Dialog adaptations.
//! LGPL-3.0-only. Logical cells are 16/32 points with six event cells of margin.
use super::*;
use polished_map_core::{EventCommand, EventDefinition, EventKind, Events};
use polished_map_platform::{ScriptOpenError, ScriptOpenRequest};
use polished_map_project::{EventDocument, event_script_path};

pub(super) struct EventUi {
    pub mode: bool,
    pub show: bool,
    pub warp_ids: bool,
    pub auto_load: bool,
    pub drag: Option<usize>,
    pub form: Option<EventForm>,
    pub change: Option<EventChange>,
    pub warp_path: Option<PathBuf>,
    pub script_request: Option<ScriptOpenRequest>,
    pub game_boy: bool,
    pub hovered_event: Option<usize>,
    pub cursor: Option<geometry::EventCursor>,
    pub screen: Option<Rect>,
    script_jobs: Vec<mpsc::Receiver<Result<(), ScriptOpenError>>>,
}
impl Default for EventUi {
    fn default() -> Self {
        Self {
            mode: false,
            show: true,
            warp_ids: true,
            auto_load: true,
            drag: None,
            form: None,
            change: None,
            warp_path: None,
            script_request: None,
            game_boy: false,
            hovered_event: None,
            cursor: None,
            screen: None,
            script_jobs: Vec::new(),
        }
    }
}
impl EventUi {
    pub(super) fn clear_document_state(&mut self) {
        self.drag = None;
        self.form = None;
        self.change = None;
        self.warp_path = None;
        self.script_request = None;
        self.hovered_event = None;
        self.cursor = None;
        self.screen = None;
    }
}
pub(super) enum EventChange {
    Load(PathBuf),
    Reload,
    Unload,
}
pub(super) enum EventAction {
    Move(usize, i16, i16),
    Options(usize),
    Warp(usize),
    ViewScript(usize),
}
pub(super) struct EventForm {
    index: usize,
    definition: EventDefinition,
    defaults: [default_value::DefaultValue<i16>; 2],
    title: String,
    pub windows_buttons: bool,
    #[cfg(test)]
    pub bounds: Rect,
}

pub(super) fn auto_load_events(
    map: &MapDocument,
    enabled: bool,
) -> (Option<EventDocument>, Option<String>) {
    if !enabled || !map.has_backing_file() {
        return (None, None);
    }
    let open = || -> Result<Option<EventDocument>, polished_map_project::ProjectError> {
        let discovered = map
            .metadata()
            .root
            .as_deref()
            .map(|root| event_script_path(root, &map.metadata().name))
            .transpose()?;
        let path = discovered
            .filter(|p| p.is_file())
            .unwrap_or_else(|| map.path().with_extension("asm"));
        match EventDocument::open(&path) {
            Ok(doc) => Ok(Some(doc)),
            Err(polished_map_project::ProjectError::Io { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    };
    match open() {
        Ok(doc) => (doc, None),
        Err(error) => (
            None,
            Some(format!(
                "The map opened, but its event script could not be loaded:\n{error}"
            )),
        ),
    }
}

pub(super) fn resize_with_events(loaded: &mut Loaded, command: &Command) -> Result<bool, String> {
    let mut map = loaded.document.map().clone();
    let old = (map.width(), map.height());
    let changed = map.apply(command).map_err(|e| e.to_string())?;
    if !changed {
        return Ok(false);
    }
    let mut events = loaded.events.clone();
    if let Command::Resize { anchor, .. } = command
        && let Some(events) = &mut events
    {
        let (block_dx, block_dy) = anchor.offset(old, (map.width(), map.height()));
        events
            .apply(&[EventCommand::Translate { block_dx, block_dy }])
            .map_err(|e| e.to_string())?;
    }
    *loaded.document.map_mut() = map;
    loaded.events = events;
    Ok(true)
}

/// Deliberately preserves upstream's extra cell at exact negative boundaries.
/// Inputs are logical points relative to the map origin, after scrolling/DPI.
pub(super) fn drag_coordinate(position: f32, cell: f32, dimension: u8) -> i16 {
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

/// master map-buttons.cpp::draw_selection_border and gameboy-screen.cpp::draw.
pub(super) fn draw_event_cursor(ui: &egui::Ui, origin: Pos2, cell: f32, state: &mut EventUi) {
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
        let r = cursor.game_boy_screen(cell);
        let area = Rect::from_min_size(origin + Vec2::new(r.x, r.y), Vec2::new(r.width, r.height));
        state.screen = Some(area);
        // The original 320x288 embedded PNG is uniformly RGBA(224,255,0,64).
        // Recreate its flat fill; no source bitmap is included.
        ui.painter()
            .rect_filled(area, 0.0, Color32::from_rgba_unmultiplied(224, 255, 0, 64));
        ui.painter()
            .rect_stroke(area, 0.0, (1.0, Color32::BLACK), egui::StrokeKind::Inside);
        ui.painter().rect_stroke(
            area.shrink(1.0),
            0.0,
            (1.0, Color32::WHITE),
            egui::StrokeKind::Inside,
        );
    }
}

pub(super) fn draw_warp_label(
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
        for delta in [
            Vec2::new(-1.0, -1.0),
            Vec2::new(-1.0, 1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
        ] {
            painter.text(
                pos + delta,
                egui::Align2::RIGHT_BOTTOM,
                &text,
                font.clone(),
                Color32::WHITE,
            );
        }
        painter.text(pos, egui::Align2::RIGHT_BOTTOM, text, font, Color32::BLACK);
    }
}

pub(super) fn canvas_events(
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
            if response.drag_started_by(egui::PointerButton::Primary) {
                state.drag = Some(index);
            }
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
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos())
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

impl Editor {
    fn request_event_script(&mut self, index: Option<usize>) {
        let Some(doc) = self.loaded.as_ref().and_then(|l| l.events.as_ref()) else {
            return;
        };
        let line = match index {
            Some(index) => {
                let Some(event) = doc.script().events().entries().get(index) else {
                    return;
                };
                Some(event.definition().line)
            }
            None => None,
        };
        match ScriptOpenRequest::new(doc.source().path().to_owned(), line) {
            Ok(request) => self.event_ui.script_request = Some(request),
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    /// The native App adapter supplies the launcher. Headless interaction tests
    /// retain the request unless explicitly dispatching to a test launcher.
    pub(super) fn dispatch_event_script(
        &mut self,
        ctx: &egui::Context,
        launch: fn(&ScriptOpenRequest) -> Result<(), ScriptOpenError>,
    ) {
        let Some(request) = self.event_ui.script_request.take() else {
            return;
        };
        let (send, receive) = mpsc::channel();
        let ctx = ctx.clone();
        match std::thread::Builder::new()
            .name("event-script-opener".into())
            .spawn(move || {
                let result = launch(&request);
                let _ = send.send(result);
                ctx.request_repaint();
            }) {
            Ok(_) => {
                self.event_ui.script_jobs.push(receive);
                self.status = "Opening saved event script in external editor…".into();
            }
            Err(e) => self.error = Some(format!("Cannot start the event script opener: {e}")),
        }
    }

    pub(super) fn poll_event_scripts(&mut self) {
        self.event_ui
            .script_jobs
            .retain(|receive| match receive.try_recv() {
                Ok(Ok(())) => {
                    self.status = "Event script opener completed".into();
                    false
                }
                Ok(Err(e)) => {
                    self.error = Some(e.to_string());
                    false
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.error = Some("The event script opener stopped unexpectedly".into());
                    false
                }
                Err(mpsc::TryRecvError::Empty) => true,
            });
    }

    pub(super) fn event_action(&mut self, action: EventAction, ctx: &egui::Context) {
        match action {
            EventAction::Move(index, x, y) => {
                if let Some(doc) = self.loaded.as_mut().and_then(|l| l.events.as_mut()) {
                    match doc.apply(&[EventCommand::Move { index, x, y }]) {
                        Ok(()) => self.status = format!("Event {index}    X: {x}    Y: {y}"),
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
            }
            EventAction::Options(index) => {
                if let Some(event) = self
                    .loaded
                    .as_ref()
                    .and_then(|l| l.events.as_ref())
                    .and_then(|d| d.script().events().entries().get(index))
                {
                    let d = event.definition().clone();
                    let suffix = event
                        .warp_id()
                        .map(|id| {
                            if d.hex_coordinates {
                                format!(" ${id:X}")
                            } else {
                                format!(" #{id}")
                            }
                        })
                        .unwrap_or_default();
                    self.event_ui.form = Some(EventForm {
                        index,
                        title: format!("{}{suffix}:", d.kind.macro_name()),
                        defaults: [d.x, d.y].map(default_value::DefaultValue::new),
                        definition: d,
                        windows_buttons: cfg!(windows),
                        #[cfg(test)]
                        bounds: Rect::NOTHING,
                    });
                }
            }
            EventAction::Warp(index) => self.request_warp(index, ctx),
            EventAction::ViewScript(index) => self.request_event_script(Some(index)),
        }
    }
    pub(super) fn request_event_change(&mut self, change: EventChange) {
        if self.loaded.is_none() {
            return;
        }
        if self
            .loaded
            .as_ref()
            .and_then(|l| l.events.as_ref())
            .is_some_and(|d| d.script().events().is_modified())
        {
            self.event_ui.change = Some(change);
        } else {
            self.perform_event_change(change);
        }
    }
    fn perform_event_change(&mut self, change: EventChange) {
        let Some(loaded) = &mut self.loaded else {
            return;
        };
        let path = match change {
            EventChange::Load(path) => Some(path),
            EventChange::Reload => loaded.events.as_ref().map(|d| d.source().path().to_owned()),
            EventChange::Unload => {
                loaded.events = None;
                self.event_ui.drag = None;
                self.event_ui.form = None;
                self.status = "Event script unloaded".into();
                return;
            }
        };
        if let Some(path) = path {
            match EventDocument::open(&path) {
                Ok(doc) => {
                    loaded.events = Some(doc);
                    self.event_ui.drag = None;
                    self.event_ui.form = None;
                    self.status = format!("Events: {}", path.display());
                }
                Err(error) => self.error = Some(error.to_string()),
            }
        }
    }
    pub(super) fn choose_events(&mut self) {
        if self.loaded.is_none() {
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Event script", &["asm"])
            .pick_file()
        {
            self.request_event_change(EventChange::Load(path));
        }
    }
    pub(super) fn save_events(&mut self) {
        if let Some(doc) = self.loaded.as_mut().and_then(|l| l.events.as_mut()) {
            match doc.save() {
                Ok(()) => self.status = "Event script saved".into(),
                Err(e) => self.error = Some(e.to_string()),
            }
        }
    }
    pub(super) fn event_data_menu(&mut self, ui: &mut egui::Ui) {
        if menu_popup::action(ui, self.loaded.is_some(), "Load Event Script…") {
            self.choose_events();
        }
        let enabled = self.loaded.as_ref().is_some_and(|l| l.events.is_some());
        if menu_popup::action(ui, enabled, "Save Event Script") {
            self.save_events();
        }
        if menu_popup::action(ui, enabled, "View Event Script") {
            self.request_event_script(None);
        }
        if menu_popup::action(ui, enabled, "Reload Event Script") {
            self.request_event_change(EventChange::Reload);
        }
        if menu_popup::action(ui, enabled, "Unload Event Script") {
            self.request_event_change(EventChange::Unload);
        }
    }
    pub(super) fn set_event_mode(&mut self, mode: bool) {
        self.event_ui.mode = mode;
        self.event_ui.drag = None;
        self.stroke.clear();
    }
    pub(super) fn event_shortcuts(&mut self, ctx: &egui::Context) {
        let shift = egui::Modifiers::COMMAND | egui::Modifiers::SHIFT;
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)) {
            // egui prepares focus traversal before widgets consume input.
            // Cancel it when Tab is used as the original mode shortcut.
            ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
            self.set_event_mode(!self.event_ui.mode);
        }
        if ctx.input_mut(|i| i.consume_key(shift, egui::Key::E)) {
            self.set_event_mode(true);
        }
        if ctx.input_mut(|i| i.consume_key(shift, egui::Key::B)) {
            self.set_event_mode(false);
        }
        if ctx.input_mut(|i| i.consume_key(shift, egui::Key::V)) {
            self.event_ui.show = !self.event_ui.show;
        }
        if ctx.input_mut(|i| i.consume_key(shift, egui::Key::Num3)) {
            self.event_ui.warp_ids = !self.event_ui.warp_ids;
        }
        if ctx.input_mut(|i| i.consume_key(shift, egui::Key::M)) {
            self.event_ui.game_boy = !self.event_ui.game_boy;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::U)) {
            self.request_event_script(None);
        }
        if ctx.input_mut(|i| i.consume_key(shift, egui::Key::A)) {
            self.save_events();
        } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::A)) {
            self.choose_events();
        }
        if ctx.input_mut(|i| i.consume_key(shift, egui::Key::R)) {
            self.rulers = !self.rulers;
        } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::R)) {
            self.request_event_change(EventChange::Reload);
        }
        if ctx.input_mut(|i| i.consume_key(shift, egui::Key::W)) {
            self.request_event_change(EventChange::Unload);
        }
    }
    fn request_warp(&mut self, index: usize, ctx: &egui::Context) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let Some((name, _)) = loaded
            .events
            .as_ref()
            .and_then(|d| d.script().events().entries().get(index))
            .and_then(|e| e.warp_destination())
        else {
            return;
        };
        if name.is_empty() {
            return;
        }
        // Macro operands are untrusted text. Navigation stays in the map directory.
        if !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            self.error = Some("Invalid warp map name".into());
            return;
        }
        let extension = loaded
            .document
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("blk");
        let path = loaded
            .document
            .path()
            .with_file_name(format!("{name}.{extension}"));
        if path == loaded.document.path() {
            self.error = Some(format!("{name} is already open"));
            return;
        }
        if !path.is_file() {
            self.error = Some(format!("Could not warp to {}", path.display()));
            return;
        }
        if self.is_modified() {
            self.event_ui.warp_path = Some(path);
            self.confirm_discard = true;
        } else {
            self.open_warp(path, ctx);
        }
    }
    pub(super) fn open_warp(&mut self, path: PathBuf, ctx: &egui::Context) {
        self.prepare_open(path);
        if self
            .open_form
            .as_ref()
            .is_some_and(|f| f.width > 0 && f.height > 0 && !f.tileset.is_empty())
        {
            let form = self.open_form.take().expect("checked open form");
            self.start_load(form, ctx);
        }
    }

    pub(super) fn event_dialog(&mut self, ctx: &egui::Context) {
        if self.event_ui.change.is_some() {
            let mut accept = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::unique("replace-events")).show(ctx, |ui| {
                ui.heading("Unsaved event changes");
                ui.label("The events have been edited. Discard those edits and continue?");
                ui.horizontal(|ui| {
                    accept = ui.button("Discard Event Changes").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
                cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            });
            if cancel {
                self.event_ui.change = None;
            } else if accept && let Some(change) = self.event_ui.change.take() {
                self.perform_event_change(change);
            }
            return;
        }
        let Some(mut form) = self.event_ui.form.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::unique("edit-event")).show(ctx, |ui| {
            ui.heading("Event Options");
            let font = egui::FontId::proportional(13.0);
            let measure = |s: &str| {
                ui.painter()
                    .layout_no_wrap(s.into(), font.clone(), ui.visuals().text_color())
                    .size()
                    .x
            };
            let mut offset = measure("X:") + 4.0;
            if form.definition.prefix.is_some() {
                offset = offset.max(measure("Prefix:") + 4.0);
            }
            if form.definition.suffix.is_some() {
                offset = offset.max(measure("Suffix:") + 4.0);
            }
            offset += 10.0;
            let field_width = 290.0 - offset;
            let height = |s: &Option<String>| {
                s.as_ref().map_or(0.0, |s| {
                    ui.painter()
                        .layout(
                            s.clone(),
                            font.clone(),
                            ui.visuals().text_color(),
                            field_width,
                        )
                        .size()
                        .y
                        .ceil()
                        .max(18.0)
                        + 4.0
                })
            };
            let ph = height(&form.definition.prefix);
            let sh = height(&form.definition.suffix);
            let ch = 48.0
                + if ph > 0.0 { ph + 4.0 } else { 0.0 }
                + if sh > 0.0 { sh + 4.0 } else { 0.0 };
            let number_width = if form.definition.hex_coordinates {
                measure("AA").max(measure("FF")) + 26.0
            } else {
                measure("999") + 26.0
            };
            let y_offset = measure("Y:") + 14.0;
            let line = format!("Line: {}", form.definition.line);
            let line_width = measure(&line) + 12.0;
            let (area, _) = ui.allocate_exact_size(Vec2::new(300.0, ch + 58.0), Sense::hover());
            #[cfg(test)]
            {
                form.bounds = area;
            }
            let rect =
                |x, y, w, h| Rect::from_min_size(area.min + Vec2::new(x, y), Vec2::new(w, h));
            ui.put(
                rect(10.0, 10.0, 280.0 - line_width, 22.0),
                egui::Label::new(&form.title),
            );
            ui.put(
                rect(290.0 - line_width, 10.0, line_width, 22.0),
                egui::Label::new(line),
            );
            let mut y = 36.0;
            if let Some(field) = &mut form.definition.prefix {
                ui.put(
                    rect(10.0, y, offset - 14.0, 22.0),
                    egui::Label::new("Prefix:"),
                );
                ui.put(
                    rect(offset, y, field_width, ph),
                    egui::TextEdit::multiline(field).desired_rows(1),
                )
                .widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Event prefix")
                });
                y += ph + 4.0;
            }
            for ((label, value, x), default) in [
                ("X:", &mut form.definition.x, offset),
                (
                    "Y:",
                    &mut form.definition.y,
                    offset + number_width + y_offset,
                ),
            ]
            .into_iter()
            .zip(&mut form.defaults)
            {
                // FLTK places the coordinate label outside the control. Do not
                // wrap X into the slightly narrower measured Y-label width.
                ui.painter().text(
                    area.min + Vec2::new(x - 4.0, y + 11.0),
                    egui::Align2::RIGHT_CENTER,
                    label,
                    font.clone(),
                    ui.visuals().text_color(),
                );
                let mut spinner = egui::DragValue::new(&mut *value).range(-6..=249).speed(1.0);
                if form.definition.hex_coordinates {
                    spinner = spinner
                        .custom_formatter(|n, _| format!("{:02X}", n as i32 as u32))
                        .custom_parser(|s| {
                            i64::from_str_radix(s.trim().trim_start_matches('$'), 16)
                                .ok()
                                .filter(|n| (i64::from(i32::MIN)..=i64::from(u32::MAX)).contains(n))
                                .map(|n| f64::from(n as i32))
                        });
                }
                let response = ui.put(rect(x, y, number_width, 22.0), spinner);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::SpinButton,
                        true,
                        if label == "X:" { "Event X" } else { "Event Y" },
                    )
                });
                default.restore(ui, &response, value);
            }
            y += 26.0;
            if let Some(field) = &mut form.definition.suffix {
                ui.put(
                    rect(10.0, y, offset - 14.0, 22.0),
                    egui::Label::new("Suffix:"),
                );
                ui.put(
                    rect(offset, y, field_width, sh),
                    egui::TextEdit::multiline(field).desired_rows(1),
                )
                .widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Event suffix")
                });
            }
            let (ok, no) = if form.windows_buttons {
                (116.0, 210.0)
            } else {
                (210.0, 116.0)
            };
            accept = ui
                .put(rect(ok, ch + 26.0, 80.0, 22.0), egui::Button::new("OK"))
                .clicked();
            cancel = ui
                .put(rect(no, ch + 26.0, 80.0, 22.0), egui::Button::new("Cancel"))
                .clicked();
            cancel |= ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        });
        if cancel {
            return;
        }
        if accept {
            let d = &form.definition;
            if let Some(doc) = self.loaded.as_mut().and_then(|l| l.events.as_mut()) {
                match doc.apply(&[EventCommand::Edit {
                    index: form.index,
                    x: d.x,
                    y: d.y,
                    prefix: d.prefix.clone(),
                    suffix: d.suffix.clone(),
                }]) {
                    Ok(()) => self.status = "Event updated".into(),
                    Err(e) => {
                        self.error = Some(e.to_string());
                        self.event_ui.form = Some(form);
                    }
                }
            }
        } else {
            self.event_ui.form = Some(form);
        }
    }
}
