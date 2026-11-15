//! Native application entry point; session setup lives in the GUI library.
fn main() -> eframe::Result {
    polished_map_gui::NativeStartup::discover_unified().run()
}
