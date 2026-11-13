# Dependency review

Start with the [vendor necessity audit](#vendor-necessity-audit--november-11-2026),
[native patch records](#pinned-native-adapter-patches) and
[continuous license checks](#continuous-license-checks). The full workspace
license report includes test features; [binary packages](releases.md#binary-content-policy)
carry the selected production/build graph's notices. Matching source retains the
entire locked dependency set for offline rebuilds. Asset rights and their known
limits are recorded separately in [asset provenance](assets.md).

## Dependency additions and adapter rationale

The following records explain the additions at their implementation dates.
Cargo.lock and the manifests are authoritative for the current graph.

The November 7 review adds `dark-light` 3.0.0 (MIT OR Apache-2.0) for a
first-launch desktop-portal theme fallback; detection runs on a worker and does
not override saved settings. It adds `futures-channel` 0.3.34 to the resolved
graph. `embed-resource` 3.0.12 (MIT) is a build-only dependency for the original
Windows multi-resolution ICO, using the installed SDK resource compiler.
Its new resolved helpers are `toml_writer` 1.2.0+spec-1.1.0, `vswhom` 0.1.0,
`vswhom-sys` 0.1.3 and `winreg` 0.56.0. No new artwork is introduced. Runtime and
build dependencies are covered by the same license inventory and source archive
policy below; the lockfile pins the complete graph. dark-light omits separate
license files in both its archive and pinned repository; its unchanged published
README/manifest declare the dual license. We select Apache-2.0 and retain those
declarations, authors and full standard terms in the [notice supplements](../LICENSES/dependency-supplements/README.md).

The native opacity adapter adds direct, target-specific uses of already-locked
`x11-dl` 2.21.0 (MIT), `objc2` 0.6.5 (MIT) and `objc2-app-kit` 0.3.2
(Zlib OR Apache-2.0 OR MIT), plus the
already-locked `thiserror` for typed failures. `raw-window-handle` now explicitly
enables its `std` error implementation and is shared across desktop targets.
AppKit enables only std, NSResponder, NSView, NSWindow and the Core Foundation scalar types;
Windows reuses the existing windows-sys UI API features. No package versions
were added or changed. `cargo deny` still passes without new exceptions.
All unsafe opacity code remains isolated in the platform crate, using borrowed
handles on the UI thread. CI adds Xvfb/xauth only to run the Linux property test;
they are not application/runtime dependencies.

Wayland opacity uses the already-locked `wayland-protocols` 0.32.13 alpha-modifier
bindings through the pinned winit/egui-winit adapters. No production package
version was added or changed. The client-frame implementation adapts the
already-locked SCTK 0.19.2 fallback frame under its original MIT license; its
separate source/patch/license records are in [vendor notices](../vendor/NOTICE.md).
The Linux protocol test directly uses already-locked winit and wayland-sys as
dev dependencies only, plus Python's standard library. It binds a private socket,
uses the real adapter and borrowed native handles, and never connects to the
user's desktop. Preserve both vendor manifests and the SCTK license in releases.

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
The GUI invokes the safe adapter with the registered palette window's borrowed live handle (eframe's root handle for the embedded fallback);
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
reads occur only on that user action. The pinned local arboard patch adds Linux
BMP fallback for original FLTK clipboard images, preserving PNG preference and
the existing Windows dependency versions. It adds no registry package; x11rb
is also a GUI dev dependency for an independent selection-owner regression.
See [patch notices](../vendor/NOTICE.md#arboard-370) and
[native acceptance](compatibility.md#plusplus-expanded-native-acceptance).
Wayland and remaining desktop transport acceptance stay open.

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
[RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html) (zvariant),
[0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html) (zbus_names), and
[0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html) (zvariant_derive).
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

The workspace patches winit 0.30.13, egui-winit 0.37.0, eframe 0.37.0,
arboard 3.7.0 and smithay-clipboard 0.7.3 from local, audited
source trees. This supplies complete file transfers without migrating eframe
to an incompatible prerelease windowing API. The GUI's direct egui-winit
dependency disables defaults and shares the features already selected by eframe.
The platform's Windows-only test dependencies reuse winit's locked Windows ABI
and tracing versions; they do not affect the production platform graph.
Linux-only tests enable both winit backends and use the already locked x11rb
0.13.2 for an independent XDND source; this adds no package/version to the lockfile
and no production dependency. The private Xvfb checks require
`libxkbcommon-x11-0` as well as the base xkbcommon library: `libxkbcommon-dev`
alone does not supply `libxkbcommon-x11.so.0`. CI installs the runtime explicitly.

See [vendor maintenance](../vendor/README.md), [source hashes](../vendor/sources.json)
and [license/modification notices](../vendor/NOTICE.md). CI verifies these trees
and runs the shared transfer and modal-lifetime tests. Native file-manager acceptance remains a
separate gate in [architecture](architecture.md#native-external-file-drop-integration-gate).

The modal bridge adds no production package versions. Eframe's audited archive
is now patched for three renderer-owned window registrations; new egui-winit
registry/lifetime logic and winit OS hooks use already-locked Windows, AppKit,
X11 and Wayland dependencies. The hidden native probe adds dev-only uses of
winit and egui-winit on all desktop targets. Wayland xdg-dialog uses the existing
0.32.13 staging protocol bindings. Source identity and full supplemental license
texts are tracked by the vendor manifest, not an unaudited Git checkout.

The macOS legacy-home adapter directly uses already-locked `objc2-foundation`
0.3.2 (Zlib OR Apache-2.0 OR MIT; std/NSString/NSPathUtilities only) and `libc`
(MIT OR Apache-2.0). No package version is added or changed. The bounded passwd
FFI stays in the platform crate; core/GUI retain their unsafe prohibition.

The isolated macOS platform check exposed an implicit NSResponder feature
previously supplied by GUI feature unification. It is now explicit, so the
platform adapter can compile independently as well as inside the application.

Development packages collect licenses for the full resolved workspace graph with
all workspace features enabled. Published crates which omit workspace-level
notices use [pinned supplements](../LICENSES/dependency-supplements/README.md),
with exact sources/hashes in [the manifest](dependency-notices.json). Original
package authors and declared license expressions are retained in the generated
inventory. The egui release-tag fallback and dispatch's published MIT declaration
are distinguished from notices retrieved at crate VCS revisions. Packaging rejects
changed supplement hashes and registry dependencies with no collected notices.

Standard Wayland image transfer extends the already-resolved Smithay adapter;
no production package version was added. Egui-winit explicitly enables the
existing image crate's PNG/BMP features for asynchronous codecs. The standalone
Smithay protocol test uses its own lockfile and upstream test-only xkbcommon
0.8.0 requirement; it is outside the production graph.


## Vendor necessity audit — November 11, 2026

**Retain all five patched crates at the current locked versions.** Each is used
by the production graph and carries behavior required by the editor. None is an
unused or unmodified copy. This is a review of the current patches, not a claim
that every patch must remain forever.

| Local crate | Why the registry version cannot currently replace it | Removal gate |
|---|---|---|
| winit 0.30.13 | Complete positioned file drops; native modal ownership; Wayland clipboard offers, opacity and frame focus. Windows OLE corrections also preserve safe transfers. | A supported replacement must expose equivalent transfer/window APIs and pass the platform/protocol regressions. |
| egui-winit 0.37.0 | Connects those native operations to egui viewports, tracks Copy/Paste results, supplies standard Wayland image transfer, and preserves restore/fullscreen settings. | Equivalent viewport adapters and clipboard/settings behavior must exist upstream or in a smaller maintained adapter. |
| eframe 0.37.0 | Registers renderer-owned windows for modals, tracks normal window geometry, and avoids presenting hidden dialog sizing passes. | Equivalent lifecycle hooks must preserve modal ownership, settings and first-frame dialog sizing. |
| arboard 3.7.0 | Reads original FLTK image/bmp clipboard offers on Linux, preserving PNG preference and bounded decoding. The X11 application path still needs this even with the separate Wayland adapter. | Registry replacement passes the original-image clipboard interoperability tests, including BMP-only offers and malformed data. |
| smithay-clipboard 0.7.3 | Adds bounded asynchronous binary MIME transfers through the existing standard Wayland clipboard owner; the registry API is insufficient for this image path. | Equivalent binary transfers, ownership, cancellation and limits pass the independent protocol probe. |

The [vendor manifest](../vendor/sources.json), [modification notices](../vendor/NOTICE.md)
and [patches](../vendor/patches) identify every source change. The separate SCTK
fallback-frame adaptation is part of winit, not an additional whole vendored
crate; its MIT attribution and source record remain necessary.

The audit verified all 326 crate files and patches against the pinned Cargo
archives with `tools/vendor_sources.py --archives`, plus the SCTK adaptation.
`tools/native_features.py --target` passed for Windows, Linux and both macOS
architectures with dev dependencies excluded. The five source trees total about
3.75 MiB before patch/manifest overhead. Removing source copies alone does not
reduce executable size or runtime memory; those depend on linked code/features.

The much larger registry source tree generated during release packaging is a
separate, ignored offline rebuild bundle. It is not committed in `vendor/`.
Retain it under the matching-source/rebuild delivery policy. Future dependency
updates should revisit each removal gate above, rather than deleting patches
that supply currently tested functionality.

## Continuous license checks

[CI](../.github/workflows/ci.yml) enforces formatting and warning-free Clippy on
Windows, Linux and macOS. Its dependency job pins cargo-deny 0.18.9 and cargo-about
0.9.3, checks all workspace features, and uploads a JSON/HTML license report.
The latter version correctly includes the renamed Linux Fontconfig dependency.
`tools/license_report.py` checks coverage against Cargo's dependency graph for
all four distribution targets and rejects missing packages, empty terms or
missing clarified license texts.

[about.toml](../about.toml) includes checksummed clarifications for workspace
LGPL terms and non-Cargo components: Fugue CC BY 3.0 icons, the OFL fallback font,
FLTK adaptations with their exception, and the MIT SCTK frame adaptation.
These combined collection expressions do not relicense the application code or
establish rights to original UI/game artwork. See [asset provenance](assets.md).
The report also retains all 57 [pinned supplemental notice records](dependency-notices.json)
and project/asset notices that automatic harvesting alone would omit.

Enabling all features exposed `colored` 2.2.0, used only by visual-test image
comparison through dify. A crate-specific MPL-2.0 exception permits that test
dependency; its source and notices remain in the full dependency source bundle.
It is absent from the shipped runtime graph. The existing advisory exceptions
are unchanged; no vulnerability exception was added. The successful
[CI report at 1bf228c](https://github.com/vulcandth/polished-map-ng/actions/runs/43347427575)
was downloaded and independently checked: 388 selected crates, all clarified
texts and 57 hash-verified supplements are covered. The narrower production
inventories in all four binary archives also passed the
[archive audit](releases.md#initial-release-archive-audit).

To reproduce the dependency job after installing its pinned tools:

```sh
cargo deny --all-features check
cargo fetch --locked
mkdir -p .scratch
cargo about generate --workspace --all-features --locked --offline --fail --format json -o .scratch/licenses.json
python3 tools/license_report.py .scratch/licenses.json --output .scratch/licenses.html
python3 -m unittest discover -s tools -p test_license_report.py
```

The [cargo-about configuration reference](https://embarkstudios.github.io/cargo-about/cli/generate/config.html)
describes clarification precedence and checksum validation. Release packaging
continues to retain matching sources and its full notice inventory separately.
