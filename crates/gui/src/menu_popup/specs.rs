//! Visible subset of Main_Window::initialize. Missing commands are not rendered.
use super::{Mark::*, Spec};
use crate::egui;

pub(super) fn group(name: &str) -> &'static [Spec<'static>] {
    match name {
        "File" => {
            const {
                &[
                    Spec::new("New…", Blank, "C+N", false).mnemonic(egui::Key::N),
                    Spec::new("Open…", Blank, "C+O", false).mnemonic(egui::Key::O),
                    Spec::new("Open Recent", Submenu, "", true),
                    Spec::new("Close", Blank, "C+W", true).mnemonic(egui::Key::C),
                    Spec::new("Save", Blank, "C+S", false).mnemonic(egui::Key::S),
                    Spec::new("Save As…", Blank, "CS+S", false).mnemonic(egui::Key::A),
                    Spec::new("Save Map", Blank, "", false).mnemonic(egui::Key::M),
                    Spec::new("Save Blockset", Blank, "", false).mnemonic(egui::Key::B),
                    Spec::new("Save Tileset", Blank, "", false).mnemonic(egui::Key::T),
                    Spec::new("Save Roof", Blank, "", true).mnemonic(egui::Key::R),
                    Spec::new("Print…", Blank, "C+P", true).mnemonic(egui::Key::P),
                    Spec::new("Exit", Blank, "A+F4", false).mnemonic(egui::Key::X),
                ]
            }
        }
        "Data" => {
            const {
                &[
                    Spec::new("Load Event Script…", Blank, "C+A", false).mnemonic(egui::Key::E),
                    Spec::new("Save Event Script", Blank, "CS+A", false).mnemonic(egui::Key::V),
                    Spec::new("View Event Script", Blank, "C+U", false).mnemonic(egui::Key::I),
                    Spec::new("Reload Event Script", Blank, "C+R", false).mnemonic(egui::Key::D),
                    Spec::new("Unload Event Script", Blank, "CS+W", true).mnemonic(egui::Key::U),
                    Spec::new("Load Palettes…", Blank, "C+L", false).mnemonic(egui::Key::P),
                    Spec::new("Export Palettes…", Blank, "", true).mnemonic(egui::Key::L),
                    Spec::new("Load Roof Colors", Blank, "", false).mnemonic(egui::Key::F),
                ]
            }
        }
        "Edit" => {
            const {
                &[
                    Spec::new("Undo", Blank, "C+Z", false).mnemonic(egui::Key::U),
                    Spec::new("Redo", Blank, "C+Y", true).mnemonic(egui::Key::R),
                    Spec::new("Copy Block", Blank, "C+C", false).mnemonic(egui::Key::C),
                    Spec::new("Paste Block", Blank, "C+V", false).mnemonic(egui::Key::P),
                    Spec::new("Swap Block", Blank, "C+X", false).mnemonic(egui::Key::W),
                ]
            }
        }
        "View" => {
            const {
                &[
                    Spec::new("Theme", Submenu, "", true).mnemonic(egui::Key::T),
                    Spec::new("Grid", Check, "C+G", false).mnemonic(egui::Key::G),
                    Spec::new("Rulers", Check, "CS+R", false).mnemonic(egui::Key::R),
                    Spec::new("Zoom", Check, "C+=", false).mnemonic(egui::Key::Z),
                    Spec::new("Block IDs", Check, "C+I", false).mnemonic(egui::Key::I),
                    Spec::new("Hexadecimal", Check, "CS+4", false).mnemonic(egui::Key::H),
                    Spec::new("Show Priority", Check, "CS+P", false).mnemonic(egui::Key::P),
                    Spec::new("Game Boy Screen", Check, "CS+M", false).mnemonic(egui::Key::B),
                    Spec::new("Show Events", Check, "CS+V", false).mnemonic(egui::Key::E),
                    Spec::new("Show Warp IDs", Check, "CS+3", true).mnemonic(egui::Key::W),
                    Spec::new("Palettes", Submenu, "", true).mnemonic(egui::Key::L),
                    Spec::new("Transparent", Check, "F10", false).mnemonic(egui::Key::A),
                    Spec::new("Full Screen", Check, "F11", false).mnemonic(egui::Key::S),
                ]
            }
        }
        "Theme" => {
            // Accepted initial theme scope: preserve source order among the
            // supported choices; other upstream themes remain deferred.
            const {
                &[
                    Spec::new("Greybird", Radio, "", false).mnemonic(egui::Key::G),
                    Spec::new("Dark", Radio, "", false).mnemonic(egui::Key::D),
                ]
            }
        }
        "Palettes" => {
            const {
                &[
                    Spec::new("Morn", Radio, "", false).mnemonic(egui::Key::M),
                    Spec::new("Day", Radio, "", false).mnemonic(egui::Key::D),
                    Spec::new("Night", Radio, "", false).mnemonic(egui::Key::N),
                    Spec::new("Dark", Radio, "", false).mnemonic(egui::Key::K),
                    Spec::new("Indoor", Radio, "", false).mnemonic(egui::Key::I),
                    Spec::new("Custom", Radio, "", false).mnemonic(egui::Key::C),
                ]
            }
        }
        "Mode" => {
            const {
                &[
                    Spec::new("Blocks", Radio, "CS+B", false).mnemonic(egui::Key::B),
                    Spec::new("Events", Radio, "CS+E", true).mnemonic(egui::Key::E),
                    Spec::new("Switch Mode", Blank, "Tab", false).mnemonic(egui::Key::S),
                ]
            }
        }
        "Tools" => {
            const {
                &[
                    Spec::new("Resize Blockset…", Blank, "C+B", false).mnemonic(egui::Key::B),
                    Spec::new("Resize Map…", Blank, "C+E", true).mnemonic(egui::Key::M),
                    Spec::new("Change Tileset…", Blank, "C+H", false).mnemonic(egui::Key::G),
                    Spec::new("Edit Tileset…", Blank, "C+T", true).mnemonic(egui::Key::T),
                    Spec::new("Change Roof…", Blank, "CS+H", false).mnemonic(egui::Key::H),
                    Spec::new("Edit Roof…", Blank, "C+F", true).mnemonic(egui::Key::R),
                    Spec::new("Edit Current Palettes…", Blank, "CS+L", false)
                        .mnemonic(egui::Key::P),
                ]
            }
        }
        "Options" => {
            const {
                &[
                    Spec::new("Monochrome", Check, "", false).mnemonic(egui::Key::M),
                    Spec::new("Tile Priority", Check, "", false).mnemonic(egui::Key::P),
                    Spec::new("256 Tiles", Check, "", false).mnemonic(egui::Key::T),
                    Spec::new("Roof Palettes", Submenu, "", true).mnemonic(egui::Key::F),
                    Spec::new("Auto-Load Events", Check, "", false).mnemonic(egui::Key::E),
                    Spec::new("Auto-Load Special Palettes", Check, "", false)
                        .mnemonic(egui::Key::S),
                    Spec::new("Auto-Load Roof Colors", Check, "", true).mnemonic(egui::Key::R),
                    Spec::new("Drag and Drop", Check, "", true).mnemonic(egui::Key::D),
                    Spec::new("Overworld Map Size…", Blank, "", false).mnemonic(egui::Key::O),
                ]
            }
        }
        "Roof Palettes" => {
            const {
                &[
                    Spec::new("Custom", Radio, "", false).mnemonic(egui::Key::C),
                    Spec::new("Morn + Day, Night", Radio, "", false).mnemonic(egui::Key::N),
                    Spec::new("Morn, Day, Night", Radio, "", false).mnemonic(egui::Key::M),
                    Spec::new("Morn + Day, Night, Custom", Radio, "", false).mnemonic(egui::Key::U),
                    Spec::new("Morn, Day, Night, Custom", Radio, "", false).mnemonic(egui::Key::O),
                ]
            }
        }
        "Help" => {
            const {
                &[
                    Spec::new("Help", Blank, "F1", true).mnemonic(egui::Key::H),
                    Spec::new("About", Blank, "C+/", false).mnemonic(egui::Key::A),
                ]
            }
        }
        _ => panic!("unknown menu group: {name}"),
    }
}
