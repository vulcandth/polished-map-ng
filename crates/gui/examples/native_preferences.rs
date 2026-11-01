//! Native settings save/restart probe. Uses disposable paths, never user stores.
//! Opens short-lived native windows; use a private display on Linux and CI elsewhere.
use std::{
    fs,
    process::{Command, ExitStatus},
    time::{Duration, Instant},
};

fn wait(child: &mut std::process::Child) -> ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("native settings probe timed out");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn main() -> eframe::Result {
    assert_eq!(
        std::env::var("POLISHED_MAP_NATIVE_PREFERENCES_PROBE").as_deref(),
        Ok("1")
    );
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !args.is_empty() {
        assert_eq!(args.len(), 3);
        return polished_map_gui::run_native_preferences_probe(
            std::path::Path::new(&args[0]),
            args[1].to_str().unwrap(),
            args[2].to_str().unwrap(),
        );
    }
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("probe-marker"),
        b"temporary native settings fixture",
    )
    .unwrap();
    for (profile, legacy_name, extras) in [
        ("master", "polishedmap.prefs", "all256:0\n"),
        (
            "plusplus",
            "polishedmap-plusplus.prefs",
            "all512:1\nswap01:1\n",
        ),
    ] {
        let legacy = directory.path().join(legacy_name);
        let original = format!(
            "; FLTK preferences file format 1.0\n; vendor: Rangi\n; application: test\n[.]\nw:640\nh:480\ntheme:4\ngrid:0\nrulers:1\n{extras}"
        );
        fs::write(&legacy, original.as_bytes()).unwrap();
        let other = directory
            .path()
            .join(if profile == "master" {
                "polished-map-ng-plusplus"
            } else {
                "polished-map-ng"
            })
            .join("app.ron");
        let other_before = fs::read(&other).ok();
        for phase in ["import", "restore", "future", "preserve", "preserve"] {
            let mut child = Command::new(std::env::current_exe().unwrap())
                .arg(directory.path())
                .args([profile, phase])
                .spawn()
                .unwrap();
            assert!(wait(&mut child).success(), "{profile}/{phase}");
            assert_eq!(fs::read(&legacy).unwrap(), original.as_bytes());
            assert_eq!(
                fs::read(&other).ok(),
                other_before,
                "other profile store changed"
            );
            println!(
                "PASS {profile}/{phase}: native save/restart, source preservation and profile isolation"
            );
        }
    }
    Ok(())
}
