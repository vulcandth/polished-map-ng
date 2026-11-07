# Testing plusplus development builds

These packages exercise the separate plusplus editor before parity acceptance.
Use a disposable copy of a plusplus-compatible project. Aesthetics and original
fonts remain deferred. The master release has its own [tester guide](testing.md).
Availability and exact build evidence belong in [releases](releases.md).

| Platform | Package suffix | Start the editor |
|---|---|---|
| Windows x86-64 | `x86_64-pc-windows-msvc.zip` | `polished-map-plusplus.exe` |
| Linux x86-64 | `x86_64-unknown-linux-gnu.tar.gz` | `./polished-map-plusplus` |
| macOS Apple Silicon | `aarch64-apple-darwin.zip` | `Polished Map NG++.app` |
| macOS Intel | `x86_64-apple-darwin.zip` | `Polished Map NG++.app` |

Platform dependencies, unsigned/ad-hoc signing limitations and checksum commands
are the same as the [master packages](testing.md). Check that BUILD-INFO.json says
`"profile": "plusplus"` and that its commit matches the corresponding source
archive. The included CLI is `polished-map`; its plusplus commands are documented
in [CLI usage](cli.md). [REBUILD.md](rebuild.md) explains rebuilding and relinking.
Packages contain no game project fixtures.

Use the pinned Polished Crystal revision in [upstream.lock.toml](../upstream.lock.toml)
for reproducible acceptance; record the revision if testing another project.
Do not open a master project in this launcher or rely on automatic conversion.

The remaining native checks are:

- On macOS, exercise Open → map/block/tile/roof/palette/event edits → save →
  close → reopen. Compare intended changes in Git and reopen supported output
  in the original plusplus application. Exercise compound graphics, tile banks,
  occurrence attributes and collision data. Check Cancel and save conflicts.
- On Linux X11 and Wayland, use the desktop's actual file manager to drop a map
  and supported assets onto their intended editor targets. Test multiple files,
  canceled drops and dirty-document confirmation; record the desktop/backend.
- Check pixel/slider/swatches against the visible mouse target on a directly
  hosted desktop at normal and high DPI. Drag outside child windows and release;
  check middle-button panning, focus loss, cancellation and returning focus.
- Move the main window and dialogs between monitors with different scales.
  Check usable placement, hit targets, normal/maximized/fullscreen restoration,
  restart after removing a monitor, and cancellation without losing drafts.
- Exchange Print images with another application using Copy/Paste, and verify
  exported images with each Print option. Test file/color dialogs, external script
  launching, settings restart and supported window transparency.

Record pass/fail separately for each platform/backend and include the BUILD-INFO
commit, OS/version, display scales, project revision and exact steps in
[a bug report](https://github.com/vulcandth/polished-map-ng/issues). Attach only
redistributable samples. A successful build or startup alone does not close the
[PP3/PP4 acceptance gates](roadmap.md#2-plusplus-parity--in-progress).
