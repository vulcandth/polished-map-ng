use super::*;
use eframe::Storage;
use egui_kittest::Harness;

fn restore(storage: &dyn Storage) -> Harness<'_, Editor> {
    Harness::builder().build_eframe(|cc| {
        cc.storage = Some(storage);
        Editor::new_unified(cc)
    })
}

#[test]
fn unified_preferences_survive_restart_with_both_capability_sets_and_separate_identity() {
    let mut storage = tests::MemoryStorage::default();
    storage.set_string(KEY, r#"{"dark":true,"monochrome":true}"#.into());
    storage.set_string(PLUSPLUS_KEY, r#"{"allow_512":true}"#.into());
    let original_master = storage.get_string(KEY);
    let original_plus = storage.get_string(PLUSPLUS_KEY);
    let mut h = restore(&storage);
    assert!(!h.state().dark && !h.state().monochrome && !h.state().allow_512);
    let editor = h.state_mut();
    editor.monochrome = true; // Effective fallback must not become a preference.
    assert!(!Preferences::capture(editor).monochrome);
    editor.set_monochrome_preference(true);
    editor.allow_256 = true;
    editor.allow_512 = true;
    editor.allow_priority = true;
    editor.meta128 = true;
    editor.bank_order = polished_map_core::plusplus::BankOrder::Bank0First;
    editor.recent_maps.remember("master.blk".into());
    editor.recent_maps.remember("plus.ablk".into());
    // Saving while plusplus is active must still use the unified app's key.
    editor.unloaded_palettes = polished_map_project::plusplus::PaletteDocument::default().into();
    assert!(editor.profile() == documents::Profile::PlusPlus);
    let expected = Preferences::capture(editor);
    let mut editor = std::mem::take(h.state_mut());
    drop(h);
    eframe::App::save(&mut editor, &mut storage);
    assert_eq!(storage.get_string(KEY), original_master);
    assert_eq!(storage.get_string(PLUSPLUS_KEY), original_plus);
    assert!(
        storage
            .get_string(application_key(Application::Unified))
            .is_some()
    );
    let h = restore(&storage);
    assert_eq!(Preferences::capture(h.state()), expected);
    assert!(h.state().application.is_unified());
    assert!(h.state().loaded.is_none());
}
