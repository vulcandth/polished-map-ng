# Dependency review

The isolated Windows platform adapter directly uses already-locked
`raw-window-handle` 0.6.2 and `windows-sys` 0.61.2 (MIT/Apache-2.0), enabling only
the common-dialog, Shell, window-message definitions and COM API features.
No new third-party package versions were added.
The GUI invokes the safe adapter with eframe's borrowed live window handle;
unsafe Win32 calls are confined to the adapter and documented at each call.
The script-opening adapter initializes and balances COM on its worker thread,
uses bounded UTF-16 buffers for association lookup, and checks ShellExecute's
return code. The platform crate is now built on all three OSes, with these Win32
dependencies still target-specific. The workspace-wide unsafe-code prohibition
remains unchanged.

The project adapter directly uses the already-locked `png` 0.18 encoder to retain
upstream's two-bit grayscale tileset output. The `image` convenience writer emits
eight-bit channels; `image` remains responsible for bounded import decoding.
This adds no new package to the dependency graph. The encoder is MIT/Apache-2.0.

The GUI directly uses the already-locked MIT/Apache-2.0 `arboard` 3.7 for an
explicit image-paste button/shortcut. egui exposes image-copy output and incoming
paste events but no request to read an image on button click. Image/Wayland data
control features reuse the locked egui-winit stack, but the direct dependency
makes Windows clipboard packages reachable in the license audit. Clipboard
reads occur only on that user action; native transport still needs platform checks.

Windows clipboard dependencies `clipboard-win` 5.4.3 and `error-code` 3.4.0
declare the Boost Software License 1.0 (BSL-1.0). The
[canonical terms](https://www.boost.org/LICENSE_1_0.txt) permit use, modification,
and redistribution with notice retention; the
[FSF license list](https://www.gnu.org/licenses/license-list.html#boost)
identifies this license as GPL-compatible. `deny.toml` allows it only for these
two crates. `LICENSES/BSL-1.0.txt` retains the complete license from the locked
error-code package; clipboard-win declares the same SPDX license in its manifest.
Both packages identify Douman as author. Preserve their package notices and
sources when preparing release dependency bundles; this review does not replace
the full release notice inventory.

The locked eframe 0.37 stack includes three Linux D-Bus maintenance advisories:
RUSTSEC-2026-0370 (zvariant), 0371 (zbus_names), and 0372 (zvariant_derive).
The upstream crates merged into zbus 6; accesskit/atspi and file-dialog dependencies
still use zbus 5. Forcing a major update would break those APIs. Targeted ignores
record this constraint, preserve all other advisory checks, and require review
on every eframe update and by 2026-12-01. No vulnerability advisory is suppressed.

Default egui fonts require Ubuntu-font-1.0 in addition to MIT/Apache/OFL terms.
Only epaint_default_fonts receives that exception; bundled license texts and
notices are retained. Native/system fonts are not copied into the repository.

Duplicate dependency versions are reported as warnings. Do not use broad ignore
rules or disable accessibility merely to obtain a green dependency check.
