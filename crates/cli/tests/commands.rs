use serde_json::Value;
use std::{fs, process::Command};

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
