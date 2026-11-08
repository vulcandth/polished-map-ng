use super::*;
use eframe::Storage;
use egui_kittest::{Harness, kittest::Queryable};
use polished_map_core::plusplus::BankOrder;

fn restore(storage: &dyn Storage) -> Harness<'_, Editor> {
    Harness::builder()
        .with_size(Vec2::new(800.0, 600.0))
        .build_eframe(|cc| {
            cc.storage = Some(storage);
            Editor::new_plusplus(cc)
        })
}

#[test]
fn plusplus_restart_restores_own_options_and_recent_files_without_master_state() {
    let mut storage = tests::MemoryStorage::default();
    let master = r#"{"version":1,"dark":true,"monochrome":true,"allow_256":true,"recent_maps":[{"utf8":"master.blk"}]}"#;
    storage.set_string(KEY, master.into());
    let mut h = restore(&storage);
    assert!(h.state().profile() == documents::Profile::PlusPlus);
    assert!(!h.state().dark && !h.state().monochrome && !h.state().allow_256);
    assert!(!h.state().allow_512 && h.state().bank_order == BankOrder::Bank1First);
    assert_eq!(
        h.state().palette_document().colors(PaletteSet::Day).len(),
        8
    );
    assert!(
        h.state()
            .last_window_title
            .as_ref()
            .is_some_and(|t| t.contains("NG++"))
    );
    assert_eq!(h.state().recent_maps, recent_maps::RecentMaps::default());
    // Use actual option inputs, then save via eframe's shutdown hook.
    for label in ["512 Tiles", "$0:80-FF Before $1:80-FF"] {
        h.get_by_label("Options").click();
        h.run();
        h.get_by_label(label).click();
        h.run();
    }
    h.state_mut().recent_maps.remember("plus.ablk".into());
    h.state_mut().dark = true;
    h.state_mut().resize_anchor = ResizeAnchor::BottomRight;
    let expected = Preferences::capture(h.state());
    let mut editor = std::mem::take(h.state_mut());
    drop(h);
    eframe::App::save(&mut editor, &mut storage);
    assert_eq!(storage.get_string(KEY).unwrap(), master);
    let serialized = storage.get_string(PLUSPLUS_KEY).unwrap();
    let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["allow_512"], true);
    assert_eq!(value["bank0_first"], true);
    let h = restore(&storage);
    assert_eq!(Preferences::capture(h.state()), expected);
    assert!(h.state().loaded.is_none() && h.state().open_form.is_none());
    drop(h);
    let h = Harness::builder().build_eframe(|cc| {
        cc.storage = Some(&storage);
        Editor::new(cc)
    });
    assert!(h.state().profile() == documents::Profile::Master);
    assert!(h.state().monochrome && h.state().allow_256 && !h.state().allow_512);
    assert_eq!(storage.get_string(PLUSPLUS_KEY).unwrap(), serialized);
}

#[test]
fn plusplus_invalid_records_are_preserved_and_hidden_master_modes_are_not_applied() {
    for raw in [
        r#"{"version":2}"#,
        r#"{"allow_512":"yes"}"#,
        r#"{"bank0_first":7}"#,
        r#"{"overworld_map_size":0}"#,
        r#"{"unknown":true}"#,
    ] {
        let mut storage = tests::MemoryStorage::default();
        storage.set_string(PLUSPLUS_KEY, raw.into());
        storage.set_string(KEY, "unrelated master record".into());
        let mut h = restore(&storage);
        assert!(!h.state().preferences_writable);
        assert!(h.state().status.contains("preserving"));
        let mut editor = std::mem::take(h.state_mut());
        drop(h);
        eframe::App::save(&mut editor, &mut storage);
        assert_eq!(storage.get_string(PLUSPLUS_KEY).unwrap(), raw);
        assert_eq!(storage.get_string(KEY).unwrap(), "unrelated master record");
    }
    let mut storage = tests::MemoryStorage::default();
    storage.set_string(PLUSPLUS_KEY,r#"{"monochrome":true,"allow_256":true,"allow_priority":true,"meta128":true,"allow_512":true}"#.into());
    let h = restore(&storage);
    assert!(h.state().preferences_writable && h.state().allow_512);
    assert!(
        !h.state().monochrome
            && !h.state().allow_256
            && !h.state().allow_priority
            && !h.state().meta128
    );
}
