//! master event.cpp, main-window.cpp and Event_Options_Dialog adaptations.
//! LGPL-3.0-only. Logical cells are 16/32 points with six event cells of margin.
mod canvas;
mod dialog;
#[cfg(test)]
pub(super) use canvas::drag_coordinate;
#[cfg(all(test, feature = "visual-tests"))]
pub(super) use canvas::draw_warp_label;
pub(super) use canvas::{canvas_events, draw_event_cursor};
#[cfg(test)]
pub(super) use dialog::measured_field_lines;

use super::documents::EventDocument;
use super::*;
use polished_map_core::{EventCommand, EventDefinition, EventKind, Events};
use polished_map_platform::{ScriptOpenError, ScriptOpenRequest};

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
    field_heights: Option<[f32; 2]>,
    title: String,
    pub windows_buttons: bool,
    #[cfg(test)]
    pub bounds: Rect,
}

pub(super) fn auto_load_events(
    map: &documents::MapDocument,
    enabled: bool,
    unified: bool,
) -> (Option<EventDocument>, Option<String>) {
    if !enabled || !map.has_backing_file() {
        return (None, None);
    }
    match if unified {
        map.auto_load_unified_events()
    } else {
        map.auto_load_events()
    } {
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
                        field_heights: None,
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
            match if self.application.is_unified() {
                loaded.document.open_unified_events(&path)
            } else {
                loaded.document.open_events(&path)
            } {
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
            let name = save_feedback::basename(doc.source().path());
            let result = if doc.script().events().is_modified() {
                doc.save()
            } else {
                Ok(())
            };
            match result {
                Ok(()) => {
                    self.status = "Event script saved".into();
                    self.saved_message(name);
                }
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
        let path = match loaded.document.warp_path(&name) {
            Ok(path) => path,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
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
}
