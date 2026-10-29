use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn fixture(root: &Path) {
    fs::create_dir_all(root.join("gfx/tilesets")).unwrap();
    fs::create_dir_all(root.join("data/tilesets")).unwrap();
    fs::write(root.join("Map.1x1.demo.blk"), [0]).unwrap();
    fs::write(root.join("gfx/tilesets/demo.2bpp"), [0; 16]).unwrap();
    fs::write(root.join("data/tilesets/demo_metatiles.bin"), [0; 16]).unwrap();
}

fn run(root: &Path, output: &str, args: &[&str]) -> (bool, Value) {
    let result = Command::new(env!("CARGO_BIN_EXE_polished-map"))
        .current_dir(root)
        .args([
            "render",
            "Map.1x1.demo.blk",
            "--root",
            ".",
            "--output",
            output,
        ])
        .args(args)
        .output()
        .unwrap();
    let json = serde_json::from_slice(&result.stdout)
        .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&result.stderr)));
    (result.status.success(), json)
}

#[test]
fn print_flags_compose_grid_and_text_and_report_font_sources() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    fs::write(root.join("mono.ttf"), epaint_default_fonts::HACK_REGULAR).unwrap();
    fs::write(root.join("sans.ttf"), epaint_default_fonts::UBUNTU_LIGHT).unwrap();
    fs::write(
        root.join("events.asm"),
        "\twarp_event $1, $1, DESTINATION, 1\n",
    )
    .unwrap();
    let (ok, plain) = run(root, "plain.png", &[]);
    assert!(ok, "{plain}");
    assert!(plain["result"].get("print").is_none());
    let (ok, grid) = run(
        root,
        "grid.png",
        &[
            "--print-grid",
            "--print-dark",
            "--print-warp-ids",
            "--print-mono-font",
            "missing.ttf",
            "--print-sans-font",
            "missing.ttf",
        ],
    );
    assert!(ok, "{grid}");
    assert!(grid["result"]["print"]["fonts"].is_null());
    let plain = image::open(root.join("plain.png")).unwrap().to_rgb8();
    let grid = image::open(root.join("grid.png")).unwrap().to_rgb8();
    for (x, y, pixel) in grid.enumerate_pixels() {
        assert_eq!(
            *pixel,
            if x == 31 || y == 31 {
                image::Rgb([38; 3])
            } else {
                *plain.get_pixel(x, y)
            }
        );
    }
    let (ok, printed) = run(
        root,
        "print.png",
        &[
            "--print-grid",
            "--print-ids",
            "--print-events",
            "--print-warp-ids",
            "--print-hex",
            "--events-file",
            "events.asm",
            "--print-mono-font",
            "mono.ttf",
            "--print-sans-font",
            "sans.ttf",
        ],
    );
    assert!(ok, "{printed}");
    let details = &printed["result"]["print"];
    assert_eq!(details["options"]["warp_ids"], true);
    assert_eq!(details["style"]["hexadecimal_ids"], true);
    assert_eq!(details["fonts"]["courier"], "Hack Regular");
    assert_eq!(details["fonts"]["helvetica"], "Ubuntu Light");
    let print = image::open(root.join("print.png")).unwrap().to_rgb8();
    assert_eq!(print.dimensions(), (32, 32));
    assert_ne!(print, plain);
    // The white label and its black outline are present in the first block.
    assert!((0..16).any(|y| (0..16).any(|x| print.get_pixel(x, y).0 == [255; 3])));
    assert!((0..16).any(|y| (0..16).any(|x| print.get_pixel(x, y).0 == [0; 3])));
    assert_eq!(fs::read(root.join("Map.1x1.demo.blk")).unwrap(), [0]);
    assert_eq!(
        fs::read_to_string(root.join("events.asm")).unwrap(),
        "\twarp_event $1, $1, DESTINATION, 1\n"
    );
    let before = fs::read(root.join("print.png")).unwrap();
    assert!(!run(root, "print.png", &[]).0);
    assert_eq!(fs::read(root.join("print.png")).unwrap(), before);
}

#[test]
fn bad_font_and_explicit_event_input_do_not_create_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    fs::write(root.join("broken.ttf"), b"not a font").unwrap();
    let (ok, error) = run(
        root,
        "font.png",
        &[
            "--print-ids",
            "--print-mono-font",
            "broken.ttf",
            "--print-sans-font",
            "broken.ttf",
        ],
    );
    assert!(!ok, "{error}");
    assert!(!root.join("font.png").exists());
    let (ok, error) = run(
        root,
        "event.png",
        &["--print-events", "--events-file", "missing.asm"],
    );
    assert!(!ok, "{error}");
    assert!(!root.join("event.png").exists());
    // No discovered event file is the same as printing with none loaded.
    assert!(run(root, "no-events.png", &["--print-events"]).0);
}
