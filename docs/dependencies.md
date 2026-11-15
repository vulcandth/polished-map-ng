# Dependencies and native patches

[Cargo.toml](../Cargo.toml), the crate manifests and [Cargo.lock](../Cargo.lock)
are authoritative for versions and features. The license report covers all
workspace features, including tests. Binary packages collect the selected target's
production and build dependencies; matching source retains the entire locked set.
See [package policy](releases.md#binary-content-policy) and [asset rights](assets.md).

## Dependency additions and adapter rationale

| Dependency group | Purpose and boundary |
|---|---|
| egui/eframe/wgpu | Native GUI and textures; defaults are disabled and required features selected explicitly. Clipboard and links must be enabled through eframe, not just through direct adapter dependencies. |
| rfd, arboard, smithay-clipboard, webbrowser | Native file dialogs, explicit image paste/copy and activated links. Wayland images use the patched standard data-device path; X11 retains arboard's FLTK BMP fallback. |
| Fontique, Skrifa, Vello CPU | Installed-font enumeration, outlines and CPU Print text masks. No display/GPU is needed for headless rendering. Fontconfig is loaded dynamically; explicit font files bypass discovery. System fonts are not redistributed. |
| dark-light | Background first-launch theme detection; never overrides saved settings or a subsequent manual choice. |
| embed-resource | Build-only Windows SDK resource compiler integration for the original multi-resolution icon. |
| png, image | Two-bit grayscale export and bounded PNG/BMP decoding; feature selection avoids unrelated image formats. |
| serde/serde_json, eframe persistence | Validated application settings; native window records remain separate. See [preferences](preferences.md). |
| OS-specific interop crates | Narrow Windows, AppKit, X11 and Wayland operations behind the platform/native boundaries. Target features are explicit so isolated crate builds do not rely on GUI feature unification. |

The font adapter uses the existing modern font stack; the rejected fontdue/fontdb
candidate is not in the lockfile. Xvfb/xauth and protocol peers are test tools,
not application runtime dependencies. Accessibility remains enabled.

## Pinned native adapter patches

The five local Cargo patches below retain original source, licenses, archive
hashes, per-file hashes and readable modifications in [vendor](../vendor/README.md).
The separate MIT SCTK fallback-frame adaptation is tracked in
[adaptations.json](../vendor/adaptations.json). Do not silently replace patched
crates with registry versions or regenerate their manifests to hide differences.

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

Verify the patch manifests with `python tools/vendor_sources.py`; use
`--archives DIR` to compare pristine Cargo inputs too. The production feature
graph is checked without dev dependencies by `tools/native_features.py` on each
CI platform. Neither check substitutes for desktop acceptance.

The larger `third-party/` tree generated for release source is a separate offline
rebuild bundle, not committed vendor code. Removing source copies alone does not
reduce executable size or memory. Revisit the removal gates on dependency updates.

## License policy and exceptions

[deny.toml](../deny.toml) and [about.toml](../about.toml) scope exceptions to the
packages that need them:

- `epaint_default_fonts`: Ubuntu-font-1.0 in addition to its other font terms.
- `clipboard-win` and `error-code`: BSL-1.0, with the complete retained text in
  [LICENSES](../LICENSES/BSL-1.0.txt) and original authorship notices.
- `colored`: MPL-2.0 for optional visual-test comparison through dify; absent
  from the production binary graph, retained in full source.
- Three D-Bus maintenance advisories (RUSTSEC-2026-0370/0371/0372) are narrowly
  ignored while transitive accesskit/rfd users remain on zbus 5. The configured
  review deadline is **2026-12-01**, also revisited on each eframe update. These
  are maintenance notices, not vulnerability exceptions.

Missing registry notices use [checksummed supplements](../LICENSES/dependency-supplements/README.md).
That document distinguishes exact VCS sources, egui's immutable release-tag
fallback, published declarations and selected standard terms. Preserve those
origins rather than inventing copyright grants. Duplicate versions warn; unknown
registries/Git sources are denied. Non-Cargo asset terms remain separate.

## Continuous license checks

[CI](../.github/workflows/ci.yml) enforces formatting and warning-free Clippy on
Windows, Linux and macOS for workspace crates. The maintained arboard adapter is
also checked explicitly for deprecated bindings and unnecessary unsafe blocks on
both macOS architectures; vendored dependencies are outside the workspace lint
selection. Its dependency job pins cargo-deny 0.18.9 and cargo-about
0.9.3, checks all workspace features, and uploads a JSON/HTML license report.
`tools/license_report.py` checks coverage against Cargo's dependency graph for
all four distribution targets and rejects missing packages, empty terms or
missing clarified license texts.

Workflow actions are pinned to reviewed Node.js 24 releases by full commit hash.
CI downloads its uploaded license report and compares both files byte-for-byte;
artifact digest mismatches fail both CI and development source downloads.
All eight jobs in the [maintenance verification](https://github.com/vulcandth/polished-map-ng/actions/runs/43672932248)
passed, including the two-architecture binding checks and artifact comparison. This updates
future workflows; the November 14 tester archives retain their exact source revision.

[about.toml](../about.toml) includes checksummed clarifications for workspace
LGPL terms and non-Cargo components: Fugue CC BY 3.0 icons, the OFL fallback font,
FLTK adaptations with their exception, and the MIT SCTK frame adaptation.
These combined collection expressions do not relicense the application code or
establish rights to original UI/game artwork. See [asset provenance](assets.md).
The report also retains [pinned supplemental notice records](dependency-notices.json)
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
