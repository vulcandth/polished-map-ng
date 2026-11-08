//! Read-only graphics previews for the reviewed Change Roof/Tileset dialogs.
//! Uses the same profile discovery/codecs as acceptance; no document is replaced
//! until OK. A single worker and latest-selection check bound work and prevent
//! a slow earlier selection from painting over the current choice.
use super::*;
use polished_map_core::Tile;

pub(super) const HEIGHT: f32 = 300.0;
pub(super) const BUTTON_Y: f32 = 268.0;
pub(super) const MENU_HEIGHT: f32 = 220.0;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Request {
    root: PathBuf,
    profile: documents::Profile,
    name: Option<String>,
    roof: bool,
}

impl Request {
    pub(super) fn new(editor: &Editor, name: Option<&str>, roof: bool) -> Option<Self> {
        Some(Self {
            root: editor.loaded.as_ref()?.tileset_location()?.0.to_owned(),
            profile: editor.profile(),
            name: name.map(str::to_owned),
            roof,
        })
    }

    fn load(&self) -> Result<egui::ColorImage, String> {
        self.read().map_err(|error| error.to_string())
    }

    fn read(&self) -> Result<egui::ColorImage, polished_map_project::ProjectError> {
        let name = self.name.as_deref().expect("named preview request");
        let tiles = match (self.profile, self.roof) {
            (documents::Profile::Master, true) => {
                polished_map_project::RoofDocument::open_in_project(&self.root, name, [0; 9])?
                    .roof()
                    .tileset()
                    .tiles()
                    .to_vec()
            }
            (documents::Profile::PlusPlus, true) => {
                polished_map_project::plusplus::RoofDocument::open_in_project(
                    &self.root,
                    name,
                    Default::default(),
                )?
                .tiles()
                .to_vec()
            }
            (documents::Profile::Master, false) => MasterTilesetDocument::open_in_project(
                &self.root,
                name,
                &MasterGraphicsOptions {
                    monochrome: true,
                    allow_256: true,
                    ..Default::default()
                },
            )?
            .tileset()
            .tiles()
            .to_vec(),
            (documents::Profile::PlusPlus, false) => {
                polished_map_project::plusplus::TilesetDocument::open_in_project(
                    &self.root,
                    name,
                    Default::default(),
                )?
                .model()
                .tiles()
                .to_vec()
            }
        };
        Ok(sheet(&tiles, self.roof))
    }
}

fn sheet(tiles: &[Tile], roof: bool) -> egui::ColorImage {
    let columns = if roof { 3 } else { 16 };
    let rows = tiles.len().div_ceil(columns);
    let width = columns * 8;
    let height = rows * 8;
    let mut pixels = vec![Color32::WHITE; width * height];
    for (index, tile) in tiles.iter().enumerate() {
        let row = if roof {
            rows - 1 - index / columns
        } else {
            index / columns
        };
        for (p, hue) in tile.hues().iter().enumerate() {
            pixels[(row * 8 + p / 8) * width + (index % columns) * 8 + p % 8] =
                Color32::from_gray([255, 85, 170, 0][usize::from(*hue)]);
        }
    }
    egui::ColorImage::new([width, height], pixels)
}

#[derive(Default)]
pub(super) struct Preview {
    wanted: Option<Request>,
    worker: Option<(Request, mpsc::Receiver<Result<egui::ColorImage, String>>)>,
    result: Option<Result<egui::TextureHandle, String>>,
}

impl Preview {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, request: Option<Request>) {
        if self.wanted != request {
            self.wanted = request;
            self.result = None;
        }
        if let Some((request, rx)) = &self.worker {
            let result = match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => Some(Err("Preview worker stopped".into())),
                Err(mpsc::TryRecvError::Empty) => None,
            };
            if let Some(result) = result {
                if self.wanted.as_ref() == Some(request) {
                    self.result = Some(result.map(|image| {
                        ui.ctx().load_texture(
                            "selected-graphics-preview",
                            image,
                            egui::TextureOptions::NEAREST,
                        )
                    }));
                }
                self.worker = None;
            }
        }
        let Some(request) = &self.wanted else { return };
        if request.name.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("No roof");
            });
            return;
        }
        if self.result.is_none() && self.worker.is_none() {
            let request = request.clone();
            let work = request.clone();
            let ctx = ui.ctx().clone();
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(work.load());
                ctx.request_repaint();
            });
            self.worker = Some((request, rx));
        }
        match &self.result {
            Some(Ok(texture)) => {
                let size = texture.size_vec2() * if request.roof { 6.0 } else { 2.0 };
                egui::ScrollArea::both()
                    .id_salt("graphics-preview-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add(
                            egui::Image::new((texture.id(), size))
                                .alt_text("Selected graphics preview"),
                        );
                    });
            }
            Some(Err(error)) => {
                ui.label(format!("Preview unavailable: {error}"));
            }
            None => {
                ui.label("Loading preview…");
            }
        }
    }
}

#[cfg(test)]
pub(super) fn wait_for_preview(h: &mut egui_kittest::Harness<'_, Editor>) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        h.run();
        let preview = h
            .state()
            .roof_choice_form
            .as_ref()
            .map(|f| &f.preview)
            .or_else(|| h.state().tileset_choice_form.as_ref().map(|f| &f.preview))
            .unwrap();
        if preview.result.is_some() || preview.wanted.as_ref().is_some_and(|r| r.name.is_none()) {
            return;
        }
        assert!(std::time::Instant::now() < deadline, "preview timed out");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(test)]
pub(super) fn scroll_to_last_choice(
    h: &mut egui_kittest::Harness<'_, Editor>,
    current: &str,
    last: &str,
    bounds: Rect,
) {
    use egui_kittest::kittest::Queryable;
    let combo = h.get_by_label(current).rect();
    h.get_by_label(current).click();
    h.run();
    h.hover_at(combo.center() + Vec2::new(0.0, 60.0));
    h.run();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: Vec2::new(0.0, -10000.0),
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    let last_rect = h.get_by_label(last).rect();
    assert!(
        bounds.contains_rect(last_rect),
        "last item must be reachable inside the bounded client: {last_rect:?} vs {bounds:?}"
    );
    h.get_by_label(last).click();
    h.run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn sheet_uses_grayscale_hues_and_editor_roof_row_order() {
        let tiles: Vec<_> = (0..9).map(|i| Tile::new([i % 4; 64]).unwrap()).collect();
        let roof = sheet(&tiles, true);
        assert_eq!(roof.size, [24, 24]);
        assert_eq!(roof.pixels[0], Color32::from_gray(170)); // tile 6
        assert_eq!(roof.pixels[16 * 24], Color32::WHITE); // tile 0
        let sheet = sheet(&tiles, false);
        assert_eq!(sheet.size, [128, 8]);
        assert_eq!(sheet.pixels[8], Color32::from_gray(85));
        assert_eq!(sheet.pixels[24], Color32::BLACK);
    }

    #[test]
    fn late_preview_cannot_replace_the_current_selection() {
        let old = Request {
            root: PathBuf::new(),
            profile: documents::Profile::Master,
            name: Some("old".into()),
            roof: true,
        };
        let none = Request {
            name: None,
            ..old.clone()
        };
        let (tx, rx) = mpsc::channel();
        let preview = Preview {
            wanted: Some(old.clone()),
            worker: Some((old, rx)),
            result: None,
        };
        let mut h = Harness::builder().build_ui_state(
            move |ui, state: &mut Preview| {
                state.show(ui, Some(none.clone()));
            },
            preview,
        );
        tx.send(Ok(egui::ColorImage::filled([24, 24], Color32::RED)))
            .unwrap();
        h.run();
        assert!(h.state().result.is_none());
        assert!(h.state().worker.is_none());
        h.get_by_label("No roof");
    }

    #[test]
    fn preview_loads_both_profiles_without_changing_documents_or_files() {
        for plus in [false, true] {
            let (_dir, mut h) = if plus {
                crate::plusplus_session::tests::fixture()
            } else {
                crate::roof_editor_tests::fixture()
            };
            let before = h
                .state()
                .loaded
                .as_ref()
                .unwrap()
                .graphics
                .render_block(0)
                .unwrap();
            for roof in [true, false] {
                let name = if roof {
                    h.state().loaded.as_ref().unwrap().roof_name().unwrap()
                } else {
                    h.state()
                        .loaded
                        .as_ref()
                        .unwrap()
                        .tileset_location()
                        .unwrap()
                        .1
                };
                let image = Request::new(h.state(), Some(name), roof)
                    .unwrap()
                    .load()
                    .unwrap();
                assert_eq!(
                    image.size,
                    if roof {
                        [24, 24]
                    } else if plus {
                        [128, 256]
                    } else {
                        [128, 128]
                    }
                );
                if roof {
                    h.state_mut().request_roof_change();
                } else {
                    h.state_mut().request_tileset_change();
                }
                h.run();
                wait_for_preview(&mut h);
                h.get_by_label("Cancel").click();
                h.run();
                assert_eq!(
                    h.state()
                        .loaded
                        .as_ref()
                        .unwrap()
                        .graphics
                        .render_block(0)
                        .unwrap(),
                    before
                );
                assert!(!h.state().is_modified());
            }
        }
    }
}
