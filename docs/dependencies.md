# Dependency review

The native opacity adapter adds direct, target-specific uses of already-locked
`x11-dl` 2.21.0 (MIT), `objc2` 0.6.5 (MIT) and `objc2-app-kit` 0.3.2
(Zlib OR Apache-2.0 OR MIT), plus the
already-locked `thiserror` for typed failures. `raw-window-handle` now explicitly
enables its `std` error implementation and is shared across desktop targets.
AppKit enables only std, NSView, NSWindow and the Core Foundation scalar types;
Windows reuses the existing windows-sys UI API features. No package versions
were added or changed. `cargo deny` still passes without new exceptions.
All unsafe opacity code remains isolated in the platform crate, using borrowed
handles on the UI thread. CI adds Xvfb/xauth only to run the Linux property test;
they are not application/runtime dependencies.

eframe 0.37 now explicitly enables `clipboard` and `links` alongside the existing
selected features. Disabling defaults without these features left egui's image
copy and URL output commands without native adapters; a direct GUI dependency on
arboard did not enable egui-winit's optional arboard integration. The production
graph now connects eframe -> egui-winit -> arboard (image copy/paste),
smithay-clipboard (Linux/Wayland text clipboard), and webbrowser (URL dispatch).
`tools/native_features.py` checks this graph without dev dependencies on each CI
host; it can also inspect a supplied target without cross-compiling.

The lockfile adds 33 packages and removes/upgrades none of the existing package
versions. The new roots are webbrowser 1.2.4 (MIT OR Apache-2.0) and
smithay-clipboard 0.7.3 (MIT). Their transitive additions include URL/IDNA/ICU,
Smithay client toolkit 0.20, calloop 0.14 and Wayland protocol crates. The new
packages declare MIT, Apache-2.0 or Unicode-3.0 terms already allowed by our
policy; declared Rust minima are at most 1.88, below our 1.95 MSRV. `cargo deny`
passes with no new license or advisory exception. Preserve their notices/source
in release dependency bundles. This graph check is not native clipboard/browser
runtime verification; clipboard ownership and compositor support remain native
acceptance gates.

Print now directly uses the already-locked Skrifa 0.44 and Vello CPU 0.2
(MIT OR Apache-2.0), plus Fontique 0.12 (MIT OR Apache-2.0, MSRV 1.88) for
native font enumeration. Vello enables only std and its u8 raster pipeline;
there is no GPU/display dependency in this adapter. Fontique uses DirectWrite
on Windows, Core Text on macOS and Fontconfig on Linux. Its `fontconfig-dlopen`
feature allows non-font operations and explicitly supplied fonts to run even
without the Fontconfig library; missing system fonts return an error.
New locked packages are Fontique, parlance 0.1.1, objc2-core-text 0.3.2,
yeslogic-fontconfig-sys 6.0.1 and roxmltree 0.21.2. Existing transitive font/render
versions stay unchanged. Retain all dependency notices in release bundles.

The initial fontdue/fontdb candidate was rejected by `cargo deny` because it
introduced the unmaintained `ttf-parser` dependency
([RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192)). None of
those three packages remains in Cargo.lock; no new advisory exception was added.
The replacement passes advisory/license/source checks. See the primary
[Fontique documentation](https://docs.rs/fontique/0.12.0/fontique/) and
[Skrifa outline API](https://docs.rs/skrifa/0.44.0/skrifa/outline/index.html).
Tests use the already-locked epaint_default_fonts data; these are test faces,
not additional bundled product fonts or purported bold substitutions.

GUI startup also uses this existing Fontique/Skrifa graph for the reference's
installed font preferences, without adding packages or redistributing host font
files. Missing GUI roles explicitly retain bundled fallbacks and log the gap.
The platform crate enables `windows-sys`'s `Win32_Graphics_Gdi` declarations for
`NONCLIENTMETRICSW`/`LOGFONTW`; the small read-only settings query stays inside
the audited unsafe boundary. Collection indices and variable coordinates are
preserved when passing a selected face to egui.

eframe 0.37's `persistence` feature now supplies platform preference storage and
native window placement. The GUI directly uses the already-locked serde and
serde_json for its versioned settings record. The feature adds ron 0.12.3,
enumn 0.1.14 and typeid 1.0.3, each MIT OR Apache-2.0, with declared Rust minima
below this workspace's MSRV. Existing package versions are unchanged; resolution
also selects already-locked windows-sys 0.59 for clipboard-win's allowed range.
Retain these crates' licenses/source notices in release dependency bundles.
Widget-memory persistence and periodic settings autosave are disabled; storage
locations and the limited saved state are documented in `preferences.md`.

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

## Pinned native adapter patches

The workspace patches winit 0.30.13 and egui-winit 0.37.0 from local, audited
source trees. This supplies complete file transfers without migrating eframe
to an incompatible prerelease windowing API. The GUI's direct egui-winit
dependency disables defaults and shares the features already selected by eframe.
The platform's Windows-only test dependencies reuse winit's locked Windows ABI
and tracing versions; they do not affect the production platform graph.

See [vendor maintenance](../vendor/README.md), [source hashes](../vendor/sources.json)
and [license/modification notices](../vendor/NOTICE.md). CI verifies these trees
and runs the shared transfer tests. Native file-manager acceptance remains a
separate gate in [architecture](architecture.md#native-external-file-drop-integration-gate).
