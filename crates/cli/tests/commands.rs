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
