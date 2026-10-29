//! Visible subset of Main_Window::initialize. Missing commands are not rendered.
use super::{Mark::*, Spec};

pub(super) fn group(name: &str) -> &'static [Spec<'static>] {
    match name {
        "File" => {
            const {
                &[
                    Spec::new("New…", Blank, "C+N", false),
                    Spec::new("Open…", Blank, "C+O", false),
                    Spec::new("Open Recent", Submenu, "", true),
                    Spec::new("Close", Blank, "C+W", true),
                    Spec::new("Save", Blank, "C+S", false),
                    Spec::new("Save As…", Blank, "CS+S", false),
                    Spec::new("Save Map", Blank, "", false),
                    Spec::new("Save Blockset", Blank, "", false),
                    Spec::new("Save Tileset", Blank, "", false),
                    Spec::new("Save Roof", Blank, "", true),
                    Spec::new("Print…", Blank, "C+P", true),
                    Spec::new("Exit", Blank, "A+F4", false),
                ]
            }
        }
        "Data" => {
            const {
                &[
                    Spec::new("Load Event Script…", Blank, "C+A", false),
                    Spec::new("Save Event Script", Blank, "CS+A", false),
                    Spec::new("View Event Script", Blank, "C+U", false),
                    Spec::new("Reload Event Script", Blank, "C+R", false),
                    Spec::new("Unload Event Script", Blank, "CS+W", true),
                    Spec::new("Load Palettes…", Blank, "C+L", false),
                    Spec::new("Export Palettes…", Blank, "", true),
                    Spec::new("Load Roof Colors", Blank, "", false),
                ]
            }
        }
        "Edit" => {
            const {
                &[
                    Spec::new("Undo", Blank, "C+Z", false),
                    Spec::new("Redo", Blank, "C+Y", true),
                    Spec::new("Copy Block", Blank, "C+C", false),
                    Spec::new("Paste Block", Blank, "C+V", false),
                    Spec::new("Swap Block", Blank, "C+X", false),
                ]
            }
        }
        "View" => {
            const {
                &[
                    Spec::new("Dark appearance", Check, "", true),
                    Spec::new("Grid", Check, "C+G", false),
                    Spec::new("Rulers", Check, "CS+R", false),
                    Spec::new("Zoom", Check, "C+=", false),
                    Spec::new("Block IDs", Check, "C+I", false),
                    Spec::new("Hexadecimal", Check, "CS+4", false),
                    Spec::new("Show Priority", Check, "CS+P", false),
                    Spec::new("Game Boy Screen", Check, "CS+M", false),
                    Spec::new("Show Events", Check, "CS+V", false),
                    Spec::new("Show Warp IDs", Check, "CS+3", true),
                    Spec::new("Palettes", Submenu, "", true),
                    Spec::new("Transparent", Check, "F10", false),
                    Spec::new("Full Screen", Check, "F11", false),
                ]
            }
        }
        "Palettes" => {
            const {
                &[
                    Spec::new("Morn", Radio, "", false),
                    Spec::new("Day", Radio, "", false),
                    Spec::new("Night", Radio, "", false),
                    Spec::new("Dark", Radio, "", false),
                    Spec::new("Indoor", Radio, "", false),
                    Spec::new("Custom", Radio, "", false),
                ]
            }
        }
        "Mode" => {
            const {
                &[
                    Spec::new("Blocks", Radio, "CS+B", false),
                    Spec::new("Events", Radio, "CS+E", true),
                    Spec::new("Switch Mode", Blank, "Tab", false),
                ]
            }
        }
        "Tools" => {
            const {
                &[
                    Spec::new("Resize Blockset…", Blank, "C+B", false),
                    Spec::new("Resize Map…", Blank, "C+E", true),
                    Spec::new("Change Tileset…", Blank, "C+H", false),
                    Spec::new("Edit Tileset…", Blank, "C+T", true),
                    Spec::new("Change Roof…", Blank, "CS+H", false),
                    Spec::new("Edit Roof…", Blank, "C+F", true),
                    Spec::new("Edit Current Palettes…", Blank, "CS+L", false),
                ]
            }
        }
        "Options" => {
            const {
                &[
                    Spec::new("Monochrome", Check, "", false),
                    Spec::new("Tile Priority", Check, "", false),
                    Spec::new("256 Tiles", Check, "", false),
                    Spec::new("Roof Palettes", Submenu, "", true),
                    Spec::new("Auto-Load Events", Check, "", false),
                    Spec::new("Auto-Load Special Palettes", Check, "", false),
                    Spec::new("Auto-Load Roof Colors", Check, "", true),
                    Spec::new("Drag and Drop", Check, "", true),
                    Spec::new("Overworld Map Size…", Blank, "", false),
                ]
            }
        }
        "Roof Palettes" => {
            const {
                &[
                    Spec::new("Custom", Radio, "", false),
                    Spec::new("Morn + Day, Night", Radio, "", false),
                    Spec::new("Morn, Day, Night", Radio, "", false),
                    Spec::new("Morn + Day, Night, Custom", Radio, "", false),
                    Spec::new("Morn, Day, Night, Custom", Radio, "", false),
                ]
            }
        }
        "Help" => {
            const {
                &[
                    Spec::new("Help", Blank, "F1", true),
                    Spec::new("About", Blank, "C+/", false),
                ]
            }
        }
        _ => panic!("unknown menu group: {name}"),
    }
}
