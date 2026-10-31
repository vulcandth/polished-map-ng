# Testing development builds

These packages are for master-profile compatibility testing, not parity or
production releases. Use a copy of your project and keep a backup. Original-font
and remaining aesthetic matching are deferred, including the approved palette
slider appearance. Plusplus support is not ready for acceptance testing.

Download a package and its `.sha256` from the development prerelease:

| Platform | Package suffix | Start the editor |
|---|---|---|
| Windows x86-64 | `x86_64-pc-windows-msvc.zip` | Extract and run `polished-map-gui.exe` |
| Linux x86-64 | `x86_64-unknown-linux-gnu.tar.gz` | Extract and run `./polished-map-gui` |
| macOS Apple Silicon | `aarch64-apple-darwin.zip` | Extract and open `Polished Map NG.app` |
| macOS Intel | `x86_64-apple-darwin.zip` | Extract and open `Polished Map NG.app` |

The CLI is `polished-map` (`polished-map.exe` on Windows); run it with `--help`.
Linux packages are built on Ubuntu 22.04 and need glibc 2.35 or newer, an X11 or
Wayland desktop, working graphics drivers, libxkbcommon, libxkbcommon-x11 and
Fontconfig/fonts. Mac bundles target macOS 11 or newer; actual minimum-version
and clean-machine verification remain open. Packages contain no game assets.

These builds have no publisher certificate or Apple notarization. macOS bundles
use only an ad-hoc signature; Windows/macOS may require an explicit local trust
decision before running them. Do not disable system security globally. If your
policy blocks an unsigned application, build from the accompanying source instead.

Verify with `sha256sum -c FILE.sha256` on Linux, `shasum -a 256 -c FILE.sha256` on
macOS, or compare `(Get-FileHash FILE -Algorithm SHA256).Hash` in PowerShell with
the checksum file. BUILD-INFO.json records the exact source commit and build host.
The matching complete source archive and REBUILD.md support rebuilding/relinking.

Focus testing on:

- Open/discover real master-compatible projects, edit maps/blocks/tiles/palettes/
  roofs/events, undo/redo, save, reopen, and compare intended changes in Git.
- Missing/malformed files, unsaved changes, save conflicts, close and exit guards.
- Clipboard, native file/color dialogs, file-manager drag/drop, and script opening.
- Settings persistence across restart, legacy import on a fresh NG profile,
  window geometry/maximization/fullscreen, multiple monitors and mixed DPI.
- Menu keyboard input: Alt+F then N, Alt+V then L then K, Escape cancellation,
  disabled items, and modal dialogs blocking editor commands.

Report problems at https://github.com/vulcandth/polished-map-ng/issues with the
BUILD-INFO commit, OS/version/architecture, display scale and Linux desktop/backend,
project revision, reproduction steps, expected/actual result, and relevant errors.
Attach a minimal redistributable sample or describe the project paths; do not
upload game graphics or other material you cannot redistribute. Native desktop
acceptance still requires these human checks; passing automated tests is not a
claim that all user workflows have been verified.
