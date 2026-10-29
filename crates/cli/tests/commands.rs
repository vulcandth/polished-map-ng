use serde_json::Value;
use std::{fs, process::Command};

#[test]
fn event_cli_previews_then_hash_checks_and_preserves_script() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("Town.asm");
    let commands = temp.path().join("commands.json");
    let original = "Label:\r\n\twarp_event 1, 2, TOWN, 1\r\n\tdb 42";
    fs::write(&path, original).unwrap();
    fs::write(&commands, r#"[{"op":"move","index":0,"x":9,"y":8}]"#).unwrap();
    let run = |extra: &[&str]| {
        let result = Command::new(env!("CARGO_BIN_EXE_polished-map"))
            .arg("edit-events")
            .arg(&path)
            .arg("--commands")
            .arg(&commands)
            .args(extra)
            .output()
            .unwrap();
        (
            result.status.success(),
            serde_json::from_slice::<Value>(&result.stdout).unwrap(),
        )
    };
    let (ok, preview) = run(&[]);
    assert!(ok, "{preview}");
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    assert_eq!(preview["result"]["events"][0]["definition"]["x"], 9);
    let hash = preview["result"]["previous_hash"].as_str().unwrap();
    assert_eq!(
        run(&["--apply"]).1["error"]["code"],
        "expected_hash_required"
    );
    assert_eq!(
        run(&["--apply", "--expected-hash", "stale"]).1["error"]["code"],
        "conflict"
    );
    assert!(run(&["--apply", "--expected-hash", hash]).0);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "Label:\r\n\twarp_event  9,  8, TOWN, 1\r\n\tdb 42"
    );
    assert_eq!(
        run(&["--apply", "--expected-hash", hash]).1["error"]["code"],
        "conflict"
    );
    fs::write(
        &commands,
        r#"[{"op":"move","index":0,"x":1,"y":1},{"op":"move","index":20,"x":1,"y":1}]"#,
    )
    .unwrap();
    assert!(!run(&[]).0);
    assert!(fs::read_to_string(&path).unwrap().contains(" 9,  8"));
}

#[test]
fn palette_cli_preview_quantization_and_explicit_export_conflict_guards() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.pal");
    let output = dir.path().join("output.pal");
    let commands = dir.path().join("commands.json");
    let original = "\tRGB 00,01,02, 03,04,05, 06,07,08, 09,10,11\n";
    fs::write(&input, original).unwrap();
    fs::write(
        &commands,
        r#"[{"op":"hex","slot":{"set":"custom","palette":8,"hue":2},"value":"037ff9"}]"#,
    )
    .unwrap();
    let run = |extra: &[&str]| {
        let result = Command::new(env!("CARGO_BIN_EXE_polished-map"))
            .arg("edit-palettes")
            .arg(&input)
            .arg("--commands")
            .arg(&commands)
            .arg("--output")
            .arg(&output)
            .arg("--monochrome")
            .args(extra)
            .output()
            .unwrap();
        (
            result.status.success(),
            serde_json::from_slice::<Value>(&result.stdout).unwrap(),
        )
    };
    let (ok, preview) = run(&[]);
    assert!(ok, "{preview}");
    let result = &preview["result"];
    assert_eq!(
        result["sets"][5]["colors"][8][2],
        serde_json::json!([3, 127, 249])
    );
    assert!(result["export_rgb"].as_str().unwrap().contains("00,15,31"));
    assert!(!output.exists());
    let hash = result["previous_hash"].as_str().unwrap();
    assert_eq!(
        run(&["--apply"]).1["error"]["code"],
        "expected_hash_required"
    );
    assert_eq!(
        run(&["--apply", "--expected-hash", "stale"]).1["error"]["code"],
        "conflict"
    );
    fs::write(&output, "existing").unwrap();
    assert_eq!(
        run(&["--apply", "--expected-hash", hash]).1["error"]["code"],
        "overwrite_confirmation_required"
    );
    let (_, overwrite) = run(&[]);
    let out_hash = overwrite["result"]["previous_output_hash"]
        .as_str()
        .unwrap();
    fs::write(&output, "external").unwrap();
    assert_eq!(
        run(&[
            "--apply",
            "--expected-hash",
            hash,
            "--expected-output-hash",
            out_hash
        ])
        .1["error"]["code"],
        "conflict"
    );
    fs::write(&output, "existing").unwrap();
    let (ok, saved) = run(&[
        "--apply",
        "--expected-hash",
        hash,
        "--expected-output-hash",
        out_hash,
    ]);
    assert!(ok, "{saved}");
    assert_eq!(
        fs::read_to_string(&output).unwrap(),
        result["export_rgb"].as_str().unwrap()
    );
    assert_eq!(fs::read_to_string(input).unwrap(), original);
    let inspected = Command::new(env!("CARGO_BIN_EXE_polished-map"))
        .arg("inspect-palettes")
        .arg(output)
        .output()
        .unwrap();
    assert!(inspected.status.success());
    let inspected: Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert_eq!(
        inspected["result"]["sets"][5]["colors"][8][2],
        serde_json::json!([0, 123, 255])
    );
}

#[test]
fn roof_cli_previews_requires_observed_hash_and_saves_only_roof_pixels() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let gfx = root.join("gfx/tilesets/roofs");
    fs::create_dir_all(&gfx).unwrap();
    let original = [255; 9 * 16];
    fs::write(gfx.join("demo.2bpp"), original).unwrap();
    let commands = root.join("commands.json");
    fs::write(
        &commands,
        r#"[{"op":"paint","tile":6,"points":[{"x":0,"y":0}],"hue":2}]"#,
    )
    .unwrap();
    let run = |action: &str, extra: &[&str]| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_polished-map"));
        command.arg(action).arg("demo").arg("--root").arg(root);
        if action == "edit-roof" {
            command.arg("--commands").arg(&commands);
        }
        let output = command.args(extra).output().unwrap();
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        (output.status.success(), value)
    };
    let (ok, preview) = run("edit-roof", &[]);
    assert!(ok, "{preview}");
    let result = &preview["result"];
    assert_eq!(result["applied"], false);
    assert_eq!(result["pixels"][6][0], 2);
    assert_eq!(result["pixels"].as_array().unwrap().len(), 9);
    assert!(result["planned_output"]["bytes"].as_u64().unwrap() > 0);
    assert!(!gfx.join("demo.png").exists());
    let hash = result["previous_hash"].as_str().unwrap();
    let (ok, missing) = run("edit-roof", &["--apply"]);
    assert!(!ok);
    assert_eq!(missing["error"]["code"], "expected_hash_required");
    fs::write(gfx.join("demo.2bpp"), [0; 9 * 16]).unwrap();
    let (ok, conflict) = run("edit-roof", &["--apply", "--expected-hash", hash]);
    assert!(!ok);
    assert_eq!(conflict["error"]["code"], "conflict");
    assert!(!gfx.join("demo.png").exists());
    fs::write(gfx.join("demo.2bpp"), original).unwrap();
    let (ok, saved) = run("edit-roof", &["--apply", "--expected-hash", hash]);
    assert!(ok, "{saved}");
    assert_eq!(saved["result"]["applied"], true);
    assert_eq!(saved["result"]["planned_output"], result["planned_output"]);
    let png = fs::read(gfx.join("demo.png")).unwrap();
    assert_eq!(&png[16..26], &[0, 0, 0, 24, 0, 0, 0, 24, 2, 0]);
    let (ok, reopened) = run("inspect-roof", &[]);
    assert!(ok, "{reopened}");
    assert_eq!(reopened["result"]["pixels"], result["pixels"]);
    assert_eq!(
        reopened["result"]["content_hash"],
        saved["result"]["content_hash"]
    );
    fs::write(
        &commands,
        r#"[{"op":"paint","tile":0,"points":[{"x":0,"y":0}],"hue":1},{"op":"delete","tile":0}]"#,
    )
    .unwrap();
    let (ok, rejected) = run(
        "edit-roof",
        &[
            "--apply",
            "--expected-hash",
            saved["result"]["content_hash"].as_str().unwrap(),
        ],
    );
    assert!(!ok, "{rejected}");
    assert_eq!(fs::read(gfx.join("demo.png")).unwrap(), png);
    assert_eq!(fs::read(gfx.join("demo.2bpp")).unwrap(), original);
}

#[test]
fn tileset_cli_previews_checks_observed_files_and_applies_shared_commands() {
    for palette_id in [3, 0x83] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let gfx = root.join("gfx/tilesets");
        fs::create_dir_all(&gfx).unwrap();
        fs::create_dir_all(root.join("data/tilesets")).unwrap();
        fs::write(root.join("data/tilesets/demo_metatiles.bin"), [1; 16]).unwrap();
        let original = [255; 32];
        fs::write(gfx.join("demo.2bpp"), original).unwrap();
        let palette = "; retained\n\ttilepal 0, GRAY, RED\n";
        fs::write(gfx.join("demo_palette_map.asm"), palette).unwrap();
        let commands = root.join("commands.json");
        fs::write(
            &commands,
            serde_json::json!([
                {"op":"paint","tile":0,"points":[{"x":0,"y":0}],"hue":2},
                {"op":"palette","tile":0,"palette":palette_id}
            ])
            .to_string(),
        )
        .unwrap();
        let run = |action: &str, extra: &[&str]| {
            let mut command = Command::new(env!("CARGO_BIN_EXE_polished-map"));
            command.arg(action).arg("demo").arg("--root").arg(root);
            if action == "edit-tileset" {
                command.arg("--commands").arg(&commands);
            }
            let output = command.args(extra).output().unwrap();
            let value: Value = serde_json::from_slice(&output.stdout).unwrap();
            (output.status.success(), value)
        };
        let (ok, preview) = run("edit-tileset", &[]);
        assert!(ok, "{preview}");
        let result = &preview["result"];
        assert_eq!(result["applied"], false);
        assert_eq!(result["pixels"][0][0], 2);
        assert_eq!(result["attributes"][0], palette_id);
        assert_eq!(result["allow_priority"], palette_id & 0x80 != 0);
        assert_eq!(result["planned_outputs"].as_array().unwrap().len(), 2);
        assert!(!gfx.join("demo.png").exists());
        assert_eq!(
            fs::read_to_string(gfx.join("demo_palette_map.asm")).unwrap(),
            palette
        );
        let hash = result["previous_hash"].as_str().unwrap();
        let (ok, missing) = run("edit-tileset", &["--apply"]);
        assert!(!ok);
        assert_eq!(missing["error"]["code"], "expected_hash_required");
        // A valid external source change invalidates the preview token.
        fs::write(gfx.join("demo.2bpp"), [0; 32]).unwrap();
        let (ok, conflict) = run("edit-tileset", &["--apply", "--expected-hash", hash]);
        assert!(!ok);
        assert_eq!(conflict["error"]["code"], "conflict");
        assert!(!gfx.join("demo.png").exists());
        fs::write(gfx.join("demo.2bpp"), original).unwrap();
        let (ok, saved) = run("edit-tileset", &["--apply", "--expected-hash", hash]);
        assert!(ok, "{saved}");
        assert_eq!(saved["result"]["applied"], true);
        assert_eq!(
            saved["result"]["planned_outputs"],
            result["planned_outputs"]
        );
        assert_eq!(fs::read(gfx.join("demo.2bpp")).unwrap(), original);
        let (ok, reopened) = run("inspect-tileset", &[]);
        assert!(ok, "{reopened}");
        assert_eq!(reopened["result"]["pixels"][0][0], 2);
        assert_eq!(reopened["result"]["attributes"][0], palette_id);
        assert_eq!(reopened["result"]["allow_priority"], palette_id & 0x80 != 0);
        assert_eq!(
            reopened["result"]["content_hash"],
            saved["result"]["content_hash"]
        );
        let new_hash = reopened["result"]["content_hash"].as_str().unwrap();
        assert_ne!(new_hash, hash);
        // Both a new PNG source and block definitions used for trimming are observed.
        fs::write(root.join("data/tilesets/demo_metatiles.bin"), [0; 16]).unwrap();
        let (ok, conflict) = run("edit-tileset", &["--apply", "--expected-hash", new_hash]);
        assert!(!ok);
        assert_eq!(conflict["error"]["code"], "conflict");
    }
}

#[test]
fn collision_cli_previews_paired_copies_and_requires_both_overwrite_hashes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let path = root.join("blocks.bin");
    let collisions = root.join("collision.bin");
    let edits = root.join("edits.json");
    let output = root.join("out.bin");
    let out_collision = root.join("out_collision.bin");
    fs::write(&path, [0; 16]).unwrap();
    fs::write(&collisions, [1, 2, 3, 4]).unwrap();
    fs::write(&output, [8; 16]).unwrap();
    fs::write(&out_collision, [9; 4]).unwrap();
    fs::write(&edits,r#"[{"op":"set_collisions","block":0,"collision":{"format":"binary","values":[255,254,253,252]}}]"#).unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_polished-map"))
            .arg("edit-blockset")
            .arg(&path)
            .arg("--collisions")
            .arg(&collisions)
            .arg("--commands")
            .arg(&edits)
            .arg("--output")
            .arg(&output)
            .arg("--output-collisions")
            .arg(&out_collision)
            .args(extra)
            .output()
            .unwrap()
    };
    let preview = run(&[]);
    assert!(
        preview.status.success(),
        "{}",
        String::from_utf8_lossy(&preview.stdout)
    );
    let value: Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(value["result"]["changed"], true);
    let hash = value["result"]["previous_output_hash"].as_str().unwrap();
    let collision_hash = value["result"]["previous_output_collision_hash"]
        .as_str()
        .unwrap();
    assert!(
        !run(&["--apply", "--expected-output-hash", hash])
            .status
            .success()
    );
    assert_eq!(fs::read(&output).unwrap(), [8; 16]);
    assert_eq!(fs::read(&out_collision).unwrap(), [9; 4]);
    assert!(
        run(&[
            "--apply",
            "--expected-output-hash",
            hash,
            "--expected-output-collision-hash",
            collision_hash
        ])
        .status
        .success()
    );
    assert_eq!(fs::read(output).unwrap(), [0; 16]);
    assert_eq!(fs::read(out_collision).unwrap(), [255, 254, 253, 252]);
    assert_eq!(fs::read(path).unwrap(), [0; 16]);
    assert_eq!(fs::read(collisions).unwrap(), [1, 2, 3, 4]);
}

#[test]
fn blockset_preview_apply_and_invalid_suffix_use_atomic_backend_commands() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("demo.bin");
    let edits = dir.path().join("blocks.json");
    fs::write(&path, [0; 16]).unwrap();
    fs::write(&edits,r#"[{"op":"paint","block":0,"at":{"x":0,"y":0},"tile":95,"brush":{"ctrl":true,"shift":false,"alt":false}}]"#).unwrap();
    let run = |apply: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_polished-map"));
        command
            .arg("edit-blockset")
            .arg(&path)
            .arg("--commands")
            .arg(&edits);
        if apply {
            command.arg("--apply");
        }
        command.output().unwrap()
    };
    let preview = run(false);
    assert!(
        preview.status.success(),
        "{}",
        String::from_utf8_lossy(&preview.stdout)
    );
    let value: Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(value["result"]["blocks"][0][1], 128);
    assert_eq!(fs::read(&path).unwrap(), [0; 16]);
    assert!(run(true).status.success());
    let before = fs::read(&path).unwrap();
    fs::write(
        &edits,
        r#"[{"op":"resize","count":2},{"op":"copy","from":3,"to":0}]"#,
    )
    .unwrap();
    assert!(!run(true).status.success());
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn render_roof_option_reaches_shared_compositor_without_modifying_inputs() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("gfx/tilesets/roofs")).unwrap();
    fs::create_dir_all(root.join("data/tilesets")).unwrap();
    fs::write(root.join("Map.1x1.demo.blk"), [0]).unwrap();
    fs::write(root.join("gfx/tilesets/demo.2bpp"), [0; 16 * 32]).unwrap();
    fs::write(root.join("gfx/tilesets/roofs/roof.2bpp"), [255; 16 * 9]).unwrap();
    fs::write(root.join("data/tilesets/demo_metatiles.bin"), [0x0a; 16]).unwrap();
    let output = root.join("roof.png");
    let result = Command::new(env!("CARGO_BIN_EXE_polished-map"))
        .arg("render")
        .arg(root.join("Map.1x1.demo.blk"))
        .arg("--root")
        .arg(root)
        .args(["--roof", "roof", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let expected = root.join("expected.png");
    polished_map_project::write_png(
        &expected,
        &polished_map_core::RgbImage {
            width: 32,
            height: 32,
            // Upstream DAY monochrome uses DMG colors; darkest is RGB5(1,3,4).
            pixels: [8, 24, 33].repeat(32 * 32),
        },
    )
    .unwrap();
    assert_eq!(fs::read(output).unwrap(), fs::read(expected).unwrap());
    assert_eq!(fs::read(root.join("Map.1x1.demo.blk")).unwrap(), [0]);
    assert_eq!(
        fs::read(root.join("gfx/tilesets/demo.2bpp")).unwrap(),
        [0; 16 * 32]
    );
}

#[test]
fn preview_then_apply_and_failed_batch_never_partially_writes() {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("test.2x1.blk");
    let commands = dir.path().join("commands.json");
    fs::write(&map, [1, 2]).unwrap();
    fs::write(&commands, r#"[{"op":"replace","from":1,"to":7}]"#).unwrap();
    let run = |apply: bool| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_polished-map"));
        cmd.arg("edit").arg(&map).arg("--commands").arg(&commands);
        if apply {
            cmd.arg("--apply");
        }
        cmd.output().unwrap()
    };
    let preview = run(false);
    assert!(preview.status.success());
    let json: Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(json["result"]["blocks"], serde_json::json!([7, 2]));
    assert_eq!(fs::read(&map).unwrap(), [1, 2]);
    assert!(run(true).status.success());
    assert_eq!(fs::read(&map).unwrap(), [7, 2]);
    fs::write(
        &commands,
        r#"[{"op":"replace","from":7,"to":8},{"op":"paint","points":[{"x":8,"y":0}],"block":3}]"#,
    )
    .unwrap();
    let failed = run(true);
    assert!(!failed.status.success());
    let error: Value = serde_json::from_slice(&failed.stdout).unwrap();
    assert_eq!(error["error"]["code"], "invalid_command");
    assert_eq!(fs::read(&map).unwrap(), [7, 2]);
}

#[test]
fn save_as_preview_and_hash_confirmed_overwrite_preserve_asm_source() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("Source.map");
    let output = dir.path().join("Copy.2x1.blk");
    fs::write(&source, "\tdb 1, 2\n").unwrap();
    fs::write(&output, [7, 8]).unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_polished-map"))
            .arg("save-as")
            .arg(&source)
            .args(["--width", "2", "--height", "1", "--output"])
            .arg(&output)
            .args(extra)
            .output()
            .unwrap()
    };
    let preview = run(&[]);
    assert!(preview.status.success());
    let json: Value = serde_json::from_slice(&preview.stdout).unwrap();
    let hash = json["result"]["previous_output_hash"].as_str().unwrap();
    assert!(!run(&["--apply"]).status.success());
    assert_eq!(fs::read(&output).unwrap(), [7, 8]);
    assert!(
        run(&["--apply", "--expected-output-hash", hash])
            .status
            .success()
    );
    assert_eq!(fs::read(&output).unwrap(), [1, 2]);
    assert!(
        !run(&["--apply", "--expected-output-hash", hash])
            .status
            .success()
    );
    assert_eq!(fs::read_to_string(source).unwrap(), "\tdb 1, 2\n");
}

#[test]
fn resize_preview_reports_reshape_and_invalid_suffix_never_writes() {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("Map.2x2.blk");
    let commands = dir.path().join("resize.json");
    fs::write(&map, [1, 2, 3, 4]).unwrap();
    fs::write(
        &commands,
        r#"[{"op":"resize","width":3,"height":3,"anchor":"bottom_right"}]"#,
    )
    .unwrap();
    let run = |apply: bool| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_polished-map"));
        cmd.arg("edit").arg(&map).arg("--commands").arg(&commands);
        if apply {
            cmd.arg("--apply");
        }
        cmd.output().unwrap()
    };
    let preview = run(false);
    assert!(preview.status.success());
    let json: Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(
        json["result"]["blocks"],
        serde_json::json!([0, 0, 0, 0, 1, 2, 0, 3, 4])
    );
    assert_eq!(json["result"]["dimensions_changed"], true);
    assert_eq!(fs::read(&map).unwrap(), [1, 2, 3, 4]);
    fs::write(
        &commands,
        r#"[{"op":"resize","width":3,"height":3},{"op":"resize","width":0,"height":3}]"#,
    )
    .unwrap();
    assert!(!run(true).status.success());
    assert_eq!(fs::read(&map).unwrap(), [1, 2, 3, 4]);
}

#[test]
fn create_defaults_to_preview_and_writes_only_with_apply() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("Created.3x2.blk");
    let run = |apply: bool| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_polished-map"));
        cmd.args(["create", "--root"])
            .arg(dir.path())
            .args(["--width", "3", "--height", "2", "--output"])
            .arg(&output);
        if apply {
            cmd.arg("--apply");
        }
        cmd.output().unwrap()
    };
    let preview = run(false);
    assert!(preview.status.success());
    let json: Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(json["result"]["map"]["path"], Value::Null);
    assert_eq!(json["result"]["map"]["content_hash"], Value::Null);
    assert!(!output.exists());
    assert!(run(true).status.success());
    assert_eq!(fs::read(&output).unwrap(), [0; 6]);
    assert!(!run(true).status.success());
}
