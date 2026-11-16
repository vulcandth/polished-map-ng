use super::*;
use std::fs;

fn file(body: &str) -> Vec<u8> {
    format!("; FLTK preferences file format 1.0\n; vendor: Rangi\n; application: Polished Map\n\n[.]\n\n{body}").into_bytes()
}

#[test]
fn root_groups_continuations_duplicates_and_numeric_prefixes() {
    let p = LegacyPreferences::parse(&file("grid:1\ngrid:-2trailing\nrecent0:C:\\\\project\\\\a\n+bc.blk\n[./child]\ngrid:0\nrecent0:wrong\n[.]\nzoom:+1\nempty:\n# annotation\nunknown:future\n")).unwrap();
    assert_eq!(p.integer("grid", 1).unwrap(), -2);
    assert_eq!(p.integer("zoom", 0).unwrap(), 1);
    assert_eq!(p.integer("empty", 8).unwrap(), 0);
    assert_eq!(p.integer("missing", 8).unwrap(), 8);
    assert_eq!(
        p.path("recent0").unwrap(),
        Some(PathBuf::from("C:\\project\\abc.blk"))
    );
    let crlf = file("grid:0\n")
        .into_iter()
        .flat_map(|b| {
            if b == b'\n' {
                vec![b'\r', b'\n']
            } else {
                vec![b]
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        LegacyPreferences::parse(&crlf)
            .unwrap()
            .integer("grid", 1)
            .unwrap(),
        0
    );
}

#[test]
fn string_escapes_roundtrip_and_malformed_files_return_errors() {
    for byte in 0..=255u8 {
        assert_eq!(
            decode_text(format!("\\{byte:03o}").as_bytes()).unwrap(),
            [byte]
        );
    }
    let p = LegacyPreferences::parse(&file(
        "recent0:folder\\\\file\\nline\\rreturn\\011tab-é.blk\n",
    ))
    .unwrap();
    assert_eq!(
        p.path("recent0").unwrap().unwrap(),
        PathBuf::from("folder\\file\nline\rreturn\ttab-é.blk")
    );
    for body in [
        "+orphan\n",
        "[broken\n",
        "key:bad\rnewline\n",
        ":empty-key\n",
    ] {
        assert!(LegacyPreferences::parse(&file(body)).is_err(), "{body:?}");
    }
    for value in ["bad\\", "bad\\4", "bad\\12", "bad\\x", "bad\\000"] {
        let p = LegacyPreferences::parse(&file(&format!("recent0:{value}\n"))).unwrap();
        assert!(p.path("recent0").is_err(), "{value}");
    }
    for value in ["2147483648", "-2147483649", "999999999999999999999"] {
        let p = LegacyPreferences::parse(&file(&format!("grid:{value}\n"))).unwrap();
        assert!(p.integer("grid", 1).is_err());
    }
    assert!(LegacyPreferences::parse(b"not FLTK").is_err());
    assert!(LegacyPreferences::parse(&file(&"a".repeat(1023))).is_err());
    assert!(LegacyPreferences::parse(&vec![b'a'; MAX_BYTES + 1]).is_err());
    let mut nul = file("grid:1");
    nul.push(0);
    assert!(LegacyPreferences::parse(&nul).is_err());
}

#[cfg(unix)]
#[test]
fn recent_paths_and_xdg_expansion_preserve_non_utf8_bytes() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let p = LegacyPreferences::parse(&file("recent0:maps/\\377.blk\n")).unwrap();
    assert_eq!(
        p.path("recent0").unwrap().unwrap().as_os_str().as_bytes(),
        b"maps/\xff.blk"
    );
    let path = PathBuf::from(std::ffi::OsString::from_vec(b"~/\xff".to_vec()));
    assert_eq!(
        expand_home(&path, Some(Path::new("/home/user")))
            .as_os_str()
            .as_bytes(),
        b"/home/user/\xff"
    );
}

#[test]
fn discovery_uses_executable_order_and_never_merges_or_changes_sources() {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir.path().join("bin");
    fs::create_dir(&exe).unwrap();
    let user = dir.path().join("profile");
    fs::create_dir_all(user.join("Rangi")).unwrap();
    let user_file = user.join("Rangi").join(FILE_NAME);
    fs::write(&user_file, file("grid:0\n")).unwrap();
    let location = LegacyUserLocation::Windows {
        roaming_appdata: Some(user),
    };
    assert_eq!(
        find_legacy_preferences(&exe, &location).unwrap(),
        Some(user_file.clone())
    );
    let display = exe.join(FILE_NAME);
    fs::write(&display, file("grid:1\n")).unwrap();
    assert_eq!(
        find_legacy_preferences(&exe, &location).unwrap(),
        Some(display.clone())
    );
    let short = exe.join("polishedmap.prefs");
    fs::write(&short, b"malformed selected file").unwrap();
    assert_eq!(
        find_legacy_preferences(&exe, &location).unwrap(),
        Some(short.clone())
    );
    assert!(LegacyPreferences::read(&short).is_err());
    assert_eq!(fs::read(&short).unwrap(), b"malformed selected file");
    assert_eq!(fs::read(&display).unwrap(), file("grid:1\n"));
    assert_eq!(fs::read(&user_file).unwrap(), file("grid:0\n"));
}

#[test]
fn unix_vendor_directory_precedence_and_mac_location_match_source() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let exe = dir.path().join("bin");
    fs::create_dir_all(&exe).unwrap();
    fs::create_dir_all(home.join(".fltk/Rangi")).unwrap();
    let old = home.join(".fltk/Rangi").join(FILE_NAME);
    fs::write(&old, file("grid:0\n")).unwrap();
    let location = LegacyUserLocation::Unix {
        home: Some(home.clone()),
        xdg_config_home: None,
    };
    assert_eq!(find_legacy_preferences(&exe, &location).unwrap(), Some(old));
    fs::create_dir_all(home.join(".config/Rangi")).unwrap();
    assert_eq!(
        find_legacy_preferences(&exe, &location).unwrap(),
        None,
        "directory masks old prefs even without a new file"
    );
    let modern = home.join(".config/Rangi").join(FILE_NAME);
    fs::write(&modern, file("grid:1\n")).unwrap();
    assert_eq!(
        find_legacy_preferences(&exe, &location).unwrap(),
        Some(modern)
    );
    for token in ["~/settings", "${HOME}/settings", "$HOME/settings"] {
        let location = LegacyUserLocation::Unix {
            home: Some(home.clone()),
            xdg_config_home: Some(token.into()),
        };
        fs::create_dir_all(home.join("settings/Rangi")).unwrap();
        let path = home.join("settings/Rangi").join(FILE_NAME);
        fs::write(&path, file("zoom:1\n")).unwrap();
        assert_eq!(
            find_legacy_preferences(&exe, &location).unwrap(),
            Some(path)
        );
    }
    let mac = home.join("Library/Preferences/Rangi").join(FILE_NAME);
    fs::create_dir_all(mac.parent().unwrap()).unwrap();
    fs::write(&mac, file("ids:1\n")).unwrap();
    assert_eq!(
        find_legacy_preferences(&exe, &LegacyUserLocation::Mac { home: Some(home) }).unwrap(),
        Some(mac)
    );
}

#[test]
fn plusplus_uses_its_own_names_with_shared_platform_precedence() {
    for platform in 0..3 {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("bin");
        let home = dir.path().join("home");
        fs::create_dir_all(&exe).unwrap();
        let (user, root) = match platform {
            0 => (
                LegacyUserLocation::Windows {
                    roaming_appdata: Some(home.clone()),
                },
                home.clone(),
            ),
            1 => (
                LegacyUserLocation::Mac {
                    home: Some(home.clone()),
                },
                home.join("Library/Preferences"),
            ),
            _ => (
                LegacyUserLocation::Unix {
                    home: Some(home.clone()),
                    xdg_config_home: None,
                },
                home.join(".fltk"),
            ),
        };
        let vendor = root.join("Rangi");
        fs::create_dir_all(&vendor).unwrap();
        let master = exe.join("polishedmap.prefs");
        fs::write(&master, file("grid:1\n")).unwrap();
        fs::write(vendor.join(FILE_NAME), file("grid:1\n")).unwrap();
        assert_eq!(find_legacy_plusplus_preferences(&exe, &user).unwrap(), None);
        let user_file = vendor.join("Polished Map++.prefs");
        fs::write(&user_file, file("all512:1\n")).unwrap();
        assert_eq!(
            find_legacy_plusplus_preferences(&exe, &user).unwrap(),
            Some(user_file)
        );
        if platform == 2 {
            fs::create_dir_all(home.join(".config/Rangi")).unwrap();
            assert_eq!(
                find_legacy_plusplus_preferences(&exe, &user).unwrap(),
                None,
                "vendor directory masks old store"
            );
        }
        let display = exe.join("Polished Map++.prefs");
        fs::write(&display, file("swap01:1\n")).unwrap();
        assert_eq!(
            find_legacy_plusplus_preferences(&exe, &user).unwrap(),
            Some(display.clone())
        );
        let short = exe.join("polishedmap-plusplus.prefs");
        fs::write(&short, b"malformed first store").unwrap();
        assert_eq!(
            find_legacy_plusplus_preferences(&exe, &user).unwrap(),
            Some(short.clone())
        );
        assert!(LegacyPreferences::read(&short).is_err());
        assert_eq!(
            find_legacy_preferences(&exe, &user).unwrap(),
            Some(master.clone())
        );
        assert_eq!(fs::read(master).unwrap(), file("grid:1\n"));
        assert_eq!(fs::read(display).unwrap(), file("swap01:1\n"));
        assert_eq!(fs::read(short).unwrap(), b"malformed first store");
    }
}
