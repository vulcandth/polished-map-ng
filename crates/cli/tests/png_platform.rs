use serde_json::Value;
use std::{fs, process::Command};

#[test]
fn explicit_png_platform_controls_inspection_render_and_edit_preflight() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("gfx/tilesets/roofs")).unwrap();
    fs::create_dir_all(root.join("data/tilesets")).unwrap();
    fs::write(root.join("data/tilesets/demo_metatiles.bin"), [0; 16]).unwrap();
    fs::write(root.join("Map.1x1.demo.blk"), [0]).unwrap();
    let image_path = root.join("gfx/tilesets/demo.png");
    image::RgbaImage::from_pixel(8, 8, image::Rgba([255, 255, 255, 0]))
        .save(&image_path)
        .unwrap();
    image::RgbaImage::from_pixel(24, 24, image::Rgba([255, 255, 255, 0]))
        .save(root.join("gfx/tilesets/roofs/roof.png"))
        .unwrap();
    let original = fs::read(&image_path).unwrap();
    let run = |action: &str, name: &str, platform: &str, extra: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_polished-map"))
            .current_dir(root)
            .args([action, name, "--root"])
            .arg(root)
            .args(["--png-platform", platform])
            .args(extra)
            .output()
            .unwrap();
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        (output.status.success(), value)
    };
    let mut hashes = Vec::new();
    for (platform, hue) in [("unix", 0), ("windows", 3)] {
        for (action, name) in [("inspect-tileset", "demo"), ("inspect-roof", "roof")] {
            let (ok, result) = run(action, name, platform, &[]);
            assert!(ok, "{result}");
            assert_eq!(result["result"]["png_platform"], platform);
            assert_eq!(
                result["result"]["pixels"][0],
                serde_json::json!([hue; 64].as_slice())
            );
            if action == "inspect-tileset" {
                hashes.push(
                    result["result"]["content_hash"]
                        .as_str()
                        .unwrap()
                        .to_owned(),
                );
            }
        }
        let file = format!("{platform}.png");
        let (ok, result) = run("render", "Map.1x1.demo.blk", platform, &["--output", &file]);
        assert!(ok, "{result}");
        assert_eq!(result["result"]["png_platform"], platform);
    }
    let unix = image::open(root.join("unix.png")).unwrap().to_rgb8();
    let windows = image::open(root.join("windows.png")).unwrap().to_rgb8();
    assert_ne!(unix, windows);
    fs::write(root.join("commands.json"), "[]").unwrap();
    let (ok, conflict) = run(
        "edit-tileset",
        "demo",
        "windows",
        &[
            "--commands",
            "commands.json",
            "--apply",
            "--expected-hash",
            &hashes[0],
        ],
    );
    assert!(!ok);
    assert_eq!(conflict["error"]["code"], "conflict");
    assert_ne!(hashes[0], hashes[1]);
    assert_eq!(fs::read(image_path).unwrap(), original);
}
