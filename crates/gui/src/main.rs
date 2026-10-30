fn main() -> eframe::Result {
    let startup = polished_map_gui::NativeStartup::discover();
    eframe::run_native(
        polished_map_gui::WINDOW_TITLE,
        eframe::NativeOptions {
            viewport: startup.viewport(),
            // Retain normal geometry, maximized state and fullscreen together.
            persist_window: true,
            ..Default::default()
        },
        Box::new(move |cc| {
            let mut editor = startup.create_editor(cc);
            if let Some(path) = std::env::args_os().nth(1) {
                editor.prepare_open(path.into());
            }
            Ok(Box::new(editor))
        }),
    )
}
