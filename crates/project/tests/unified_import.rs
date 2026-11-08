use polished_map_project::{OpenOptions, plusplus};
use std::fs;

#[test]
fn unified_asm_import_reports_trailing_bytes_and_requires_explicit_binary_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Legacy.MAP");
    let source = b"MapData:\r\n\tdb $12, $34, $56 ; preserve assembly\r\n";
    fs::write(&path, source).unwrap();
    let options = OpenOptions {
        width: Some(2),
        height: Some(1),
    };
    assert!(plusplus::MapDocument::open(&path, &options).is_err());
    let report =
        plusplus::MapDocument::import_asm_with_report(&path, &options, Some(dir.path())).unwrap();
    assert_eq!(report.trailing_blocks, 1);
    let mut doc = report.document;
    assert_eq!(doc.map().blocks(), [0x12, 0x34]);
    assert!(doc.save().unwrap_err().to_string().contains(".ablk"));
    assert_eq!(fs::read(&path).unwrap(), source);
    assert!(plusplus::prepare_save_as(&path).is_err());
    let target = plusplus::prepare_save_as(&dir.path().join("Imported")).unwrap();
    doc.save_as(&target).unwrap();
    assert_eq!(target.path().extension().unwrap(), "ablk");
    assert_eq!(fs::read(target.path()).unwrap(), [0x12, 0x34]);
    assert_eq!(fs::read(path).unwrap(), source);
    assert!(
        plusplus::MapDocument::import_asm_with_report(target.path(), &options, Some(dir.path()))
            .is_err()
    );
    let reopened = plusplus::MapDocument::open(target.path(), &options).unwrap();
    assert_eq!(reopened.map().blocks(), doc.map().blocks());
}

#[test]
fn unified_asm_import_rejects_short_or_malformed_input_without_writes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Bad.map");
    let options = OpenOptions {
        width: Some(2),
        height: Some(1),
    };
    for bytes in [
        b"\tdb $12\n".as_slice(),
        b"\tdb $12, unresolved\n",
        b"\tdb $100, $01\n",
    ] {
        fs::write(&path, bytes).unwrap();
        assert!(
            plusplus::MapDocument::import_asm_with_report(&path, &options, Some(dir.path()))
                .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}
