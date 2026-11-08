"""Regressions for archives built from reproducibly timestamped Cargo sources."""
import os
import hashlib
import json
import plistlib
from pathlib import Path
import tempfile
import unittest
import zipfile

from development_package import PROFILES, archive_tree, common, copy_gui, package_profile, package_stem, supplemental_notices


class DevelopmentArchiveTests(unittest.TestCase):
    def test_profile_inheritance_and_source_binary_mismatch(self):
        self.assertEqual(package_profile(None, {}), "master")
        self.assertEqual(package_profile(None, {"commit": "old"}), "master")
        for profile in PROFILES:
            self.assertEqual(package_profile(profile, {}), profile)
            self.assertEqual(package_profile(None, {"profile": profile}), profile)
            for other in PROFILES:
                if other != profile:
                    with self.assertRaisesRegex(ValueError, "must match"):
                        package_profile(other, {"profile": profile})
        with self.assertRaisesRegex(ValueError, "Unknown source profile"):
            package_profile(None, {"profile": "unknown"})
        with self.assertRaisesRegex(ValueError, "Unknown requested profile"):
            package_profile("unknown", {})

    def test_archive_names_distinguish_all_launchers_at_the_same_commit(self):
        commit = "1234567890abcdef"
        names = {package_stem(commit, profile) for profile in PROFILES}
        self.assertEqual(names, {
            "polished-map-ng-dev-1234567890ab",
            "polished-map-ng-plusplus-dev-1234567890ab",
            "polished-map-ng-unified-dev-1234567890ab",
        })

    def test_launchers_and_mac_bundle_identities_are_separate(self):
        for target in ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu",
                       "aarch64-apple-darwin", "x86_64-apple-darwin"]:
            with self.subTest(target=target), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                binaries, destination = root / "bin", root / "package"
                binaries.mkdir()
                destination.mkdir()
                suffix = ".exe" if "windows" in target else ""
                for executable, _, _ in PROFILES.values():
                    source = binaries / (executable + suffix)
                    source.write_bytes(executable.encode("utf-8"))
                    source.chmod(0o755)
                identities = set()
                for profile, executable, display in [
                    ("master", "polished-map-gui", "Polished Map NG"),
                    ("plusplus", "polished-map-plusplus", "Polished Map NG++"),
                    ("unified", "polished-map-unified", "Polished Map NG Unified"),
                ]:
                    bundle = copy_gui(destination, binaries, target, profile)
                    output = destination / (executable + suffix)
                    if "apple" in target:
                        self.assertEqual(bundle.name, display + ".app")
                        with (bundle / "Contents/Info.plist").open("rb") as file:
                            plist = plistlib.load(file)
                        self.assertEqual(plist["CFBundleExecutable"], executable)
                        self.assertEqual(plist["CFBundleDisplayName"], display)
                        self.assertNotIn(plist["CFBundleIdentifier"], identities)
                        identities.add(plist["CFBundleIdentifier"])
                        output = bundle / "Contents/MacOS" / executable
                    else:
                        self.assertIsNone(bundle)
                    source = binaries / (executable + suffix)
                    self.assertEqual(output.read_bytes(), source.read_bytes())
                    self.assertEqual(output.stat().st_mode, source.stat().st_mode)

    def test_missing_launcher_never_substitutes_master(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "polished-map-gui").write_bytes(b"master")
            for profile in ["plusplus", "unified"]:
                with self.assertRaises(FileNotFoundError):
                    copy_gui(root, root, "x86_64-unknown-linux-gnu", profile)

    def test_binary_tester_entry_points_preserve_offline_guide_links(self):
        for profile, guide in [("master", "testing.md"), ("plusplus", "testing-plusplus.md"),
                               ("unified", "testing-unified.md")]:
            with self.subTest(profile=profile), tempfile.TemporaryDirectory() as temporary:
                destination = Path(temporary)
                common(destination, {"development": True, "profile": profile})
                entry = (destination / "TESTING.md").read_text(encoding="utf-8")
                self.assertIn(f"](docs/{guide})", entry)
                self.assertEqual((destination / "docs" / guide).read_bytes(),
                                 (Path("docs") / guide).read_bytes())
                for relative in ["testing.md", "testing-plusplus.md", "releases.md", "cli.md",
                                 "rebuild.md", "roadmap.md", "../upstream.lock.toml"]:
                    self.assertTrue((destination / "docs" / relative).is_file(), relative)

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
