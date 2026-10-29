fn main() -> eframe::Result {
    eframe::run_native(
        "Polished Map NG — development",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_app_id("polished-map-ng")
                .with_inner_size([800.0, 600.0])
                .with_min_inner_size([335.0, 262.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            let mut editor = polished_map_gui::Editor::new(cc);
            if let Some(path) = std::env::args_os().nth(1) {
                editor.prepare_open(path.into());
            }
            Ok(Box::new(editor))
        }),
    )
}
