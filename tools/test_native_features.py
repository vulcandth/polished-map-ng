"""Regression tests for the consolidated headless dependency boundary."""
import unittest
from native_features import backend_errors


class HeadlessBackendTests(unittest.TestCase):
    def test_cpu_fonts_and_explicit_file_io_are_allowed(self):
        self.assertEqual(backend_errors({"polished-map-core", "fontique", "skrifa",
                                        "vello_cpu", "png", "image", "tempfile"}), [])

    def test_platform_gui_and_retired_crate_are_rejected(self):
        for forbidden in ["polished-map-platform", "polished-map-project", "polished-map-gui", "winit",
                          "egui", "eframe", "wgpu", "rfd", "arboard"]:
            with self.subTest(package=forbidden):
                self.assertTrue(backend_errors({"polished-map-core", forbidden}))

    def test_missing_shared_core_is_rejected(self):
        self.assertTrue(backend_errors({"polished-map-cli"}))


if __name__ == "__main__":
    unittest.main()
