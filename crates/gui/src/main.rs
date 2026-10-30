fn main() -> eframe::Result {
    eframe::run_native(
        polished_map_gui::WINDOW_TITLE,
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_app_id("polished-map-ng")
                .with_icon(polished_map_gui::application_icon())
                .with_inner_size([800.0, 600.0])
                .with_min_inner_size([335.0, 262.0]),
            // Retain normal geometry, maximized state and fullscreen together.
            persist_window: true,
            ..Default::default()
        },
        Box::new(|cc| {
            let mut editor = polished_map_gui::Editor::new_native(cc);
            if let Some(path) = std::env::args_os().nth(1) {
                editor.prepare_open(path.into());
            }
            Ok(Box::new(editor))
        }),
    )
}
