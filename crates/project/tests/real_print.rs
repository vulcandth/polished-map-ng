//! Print integration against the pinned project without distributing its assets.
use polished_map_core::{PrintOptions, PrintStyle, PrintTextMask};
use polished_map_project::{
    EventScript, MapDocument, MasterGraphicsOptions, OpenOptions, load_master_map_graphics,
};
use std::{fs, path::PathBuf};

#[test]
#[ignore = "requires POLISHED_MAP_POKECRYSTAL; run in the real-project CI job"]
fn pinned_map_print_composes_real_roofs_palettes_and_events() {
    let root = PathBuf::from(
        std::env::var_os("POLISHED_MAP_POKECRYSTAL").expect("set POLISHED_MAP_POKECRYSTAL"),
    );
    let path = root.join("maps/NewBarkTown.blk");
    let map_bytes = fs::read(&path).unwrap();
    let script_path = root.join("maps/NewBarkTown.asm");
    let script_bytes = fs::read_to_string(&script_path).unwrap();
    let script = EventScript::parse(&script_bytes).unwrap();
    assert!(!script.events().entries().is_empty());
    let doc = MapDocument::open_in_project(&path, &OpenOptions::default(), Some(&root)).unwrap();
    for roof in [None, Some("new_bark"), Some("goldenrod")] {
        let graphics = load_master_map_graphics(
            &doc,
            "johto",
            &MasterGraphicsOptions {
                roof: roof.map(str::to_owned),
                ..Default::default()
            },
        )
        .unwrap();
        let plain = graphics.render_map(doc.map()).unwrap();
        let print = graphics
            .render_print(
                doc.map(),
                Some(script.events()),
                PrintOptions::default(),
                PrintStyle::default(),
                |_, _| panic!("fonts off"),
            )
            .unwrap();
        assert_eq!(plain, print);
        for flags in 0..32 {
            let print = graphics
                .render_print(
                    doc.map(),
                    Some(script.events()),
                    PrintOptions {
                        grid: flags & 1 != 0,
                        ids: flags & 2 != 0,
                        priority: flags & 4 != 0,
                        events: flags & 8 != 0,
                        warp_ids: flags & 16 != 0,
                    },
                    PrintStyle::default(),
                    |_, _| PrintTextMask::new(1, 1, vec![255]),
                )
                .unwrap();
            assert_eq!((print.width, print.height), (plain.width, plain.height));
            assert_eq!(print.pixels.len(), plain.pixels.len());
            if flags & 11 != 0 {
                assert_ne!(plain, print);
            }
        }
        assert_eq!(plain, graphics.render_map(doc.map()).unwrap());
    }
    assert_eq!(doc.map().blocks(), map_bytes);
    assert_eq!(script.render(), script_bytes);
    assert_eq!(fs::read(path).unwrap(), map_bytes);
    assert_eq!(fs::read_to_string(script_path).unwrap(), script_bytes);
}
