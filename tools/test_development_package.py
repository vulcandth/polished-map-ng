"""Regressions for archives built from reproducibly timestamped Cargo sources."""
import os
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

from development_package import archive_tree, common, supplemental_notices


class DevelopmentArchiveTests(unittest.TestCase):
    def test_binary_assets_keep_notices_without_loose_runtime_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary)
            common(destination, {"development": True})
            source = Path("crates/gui/assets")
            expected = sorted(source.rglob("NOTICE.md"))
            self.assertTrue(expected)
            copied = sorted(p for p in (destination / source).rglob("*") if p.is_file())
            self.assertEqual([p.relative_to(destination) for p in copied], expected)
            for notice in expected:
                self.assertEqual((destination / notice).read_bytes(), notice.read_bytes())
            self.assertTrue((destination / "LICENSES/egui-fonts/NotoSansSymbols2-OFL.txt").is_file())

    def test_pinned_upstream_notices_match_the_provenance_manifest(self):
        supplemental_notices(Path(__file__).resolve().parents[1])

    def test_changed_upstream_license_text_blocks_packaging(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "docs").mkdir()
            notice = root / "LICENSE.txt"
            notice.write_bytes(b"Original upstream license\n")
            record = {"name": "example", "version": "1.0.0", "files": [
                {"path": "LICENSE.txt", "sha256": hashlib.sha256(notice.read_bytes()).hexdigest()}]}
            (root / "docs/dependency-notices.json").write_text(json.dumps([record]), encoding="utf-8")
            self.assertIn(("example", "1.0.0"), supplemental_notices(root))
            notice.write_bytes(b"Accidentally replaced text\n")
            with self.assertRaisesRegex(ValueError, "Changed dependency notice"):
                supplemental_notices(root)

    def test_old_dependency_notices_and_executable_modes_survive_zip(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "candidate/Contents/MacOS").mkdir(parents=True)
            notice = root / "candidate/LICENSE.txt"
            notice.write_bytes(b"Copyright example\r\nLicense bytes\x00\xff")
            os.utime(notice, (1, 1))
            executable = root / "candidate/Contents/MacOS/editor"
            executable.write_bytes(b"test executable")
            executable.chmod(0o755)
            archive = archive_tree(root, root, "candidate", "zip")
            with zipfile.ZipFile(archive) as zipped:
                self.assertEqual(zipped.read("candidate/LICENSE.txt"), notice.read_bytes())
                self.assertEqual(zipped.getinfo("candidate/LICENSE.txt").date_time[0], 1980)
                stored = zipped.getinfo("candidate/Contents/MacOS/editor")
                self.assertEqual(stored.external_attr >> 16, executable.stat().st_mode)
                self.assertEqual(zipped.testzip(), None)


if __name__ == "__main__":
    unittest.main()
