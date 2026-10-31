"""Regressions for archives built from reproducibly timestamped Cargo sources."""
import os
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

from development_package import archive_tree, supplemental_notices


class DevelopmentArchiveTests(unittest.TestCase):
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
