"""Regressions for archives built from reproducibly timestamped Cargo sources."""
import os
from pathlib import Path
import tempfile
import unittest
import zipfile

from development_package import archive_tree


class DevelopmentArchiveTests(unittest.TestCase):
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
