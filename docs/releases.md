# Release and compliance checklist

Development artifacts are not parity releases. The user authorized tester
packages before native/master acceptance on 2026-10-31. Those packages follow
the development policy below; the full checklist remains the final release gate.

## Development packages

[The manual workflow](../.github/workflows/development.yml) creates a complete
source archive, then builds Windows x86-64, Linux x86-64, Apple Silicon and Intel
Mac packages **offline from that archive**. Registry source is vendored separately
from our patched native dependencies. All four jobs run the packaged CLI and
the platform home-discovery tests. BUILD-INFO identifies the immutable commit,
toolchain, build host, target and signing status. Each archive has a SHA-256 file.
Mac bundles have local ad-hoc signatures, not Developer ID signing/notarization.

Fonts, icons and Help are embedded in the executable; no neighboring asset files
are required to run it. Binary packages retain asset notices, license texts and a
dependency license inventory for the resolved workspace with all features enabled.
Published-crate omissions are covered by [pinned license supplements](../LICENSES/dependency-supplements/README.md),
including original authorship metadata and independently verified hashes.
The matching source archive includes
the full tracked repository (including original artwork and converted pixels),
all registry dependencies, lockfile, toolchain,
patches and [rebuild/relink instructions](rebuild.md). No external project/game
fixtures or ignored working files are included. Source and binaries must be
published together as a **prerelease**, with successful CI for the packaged commit.
Do not mark a development release as the latest stable release.

Windows uses an MSVC ZIP; Linux uses an Ubuntu 22.04-built tarball; macOS uses
separate app-bundle ZIPs. The Mac runner labels follow
[GitHub's runner documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job).
ZIP packaging clamps reproducible dependency timestamps to ZIP's supported
range while preserving license bytes and executable modes. The regression in
[the archive test](../tools/test_development_package.py) covers the pre-1980
notice timestamps found during the first Mac package run.

The [2026-11-06 master development candidate](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-2026-11-06-73c0e36)
is published from `73c0e362ead36be8b8a44c0e7b999ed187b8ac17`.
[All eight CI jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42297779471)
and [all five packaging jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42297818324)
passed. The source archive contains all 455 locked registry packages, patched
native dependencies, original artwork and rebuilding instructions. Each binary
archive retains all 463 dependency license/attribution records and asset notices.
All five archives passed checksum, exact-commit, source-content, executable
architecture/mode, Mac bundle/signature-metadata and license checks. All ten
published asset digests match the verified local files. The shipped Windows CLI
also ran locally; focused native Windows GUI evidence is recorded in the
[review disposition](compatibility.md#november-1-upstream-review).

Historical package evidence follows; it does not supersede this candidate.

The [2026-10-31 development candidate](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-2026-10-31-9a6f607)
is published from `9a6f6074fc939276fa747c287280a5d61898c5b8`.
[All eight CI jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/41236338190)
and [all five package jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/41236340646)
passed. Source contains all 455 locked registry packages; each binary archive
contains all 463 resolved package license/attribution records, including pinned
supplements. Downloaded archives passed checksum, commit identity, architecture,
executable permission, Mac bundle/signature metadata, license and source-content
checks. All ten published asset digests match the verified local files.

Native package build/CLI hosts were Windows Server 2025 (10.0.26100), Ubuntu
22.04/glibc 2.35, macOS 15.8.2 arm64 and macOS 15.7.9 Intel. Both Macs ran the
Foundation/passwd home-discovery tests and verified their ad-hoc bundle signatures.
The downloaded Windows CLI also ran locally. The downloaded Linux CLI and GUI
ran on WSL Ubuntu 22.04.5; the GUI stayed running through a 15-second isolated
Xvfb/Vulkan startup check, with no desktop automation or project edits.
Windows executables import VCRUNTIME140.dll; the current [tester guide](testing.md)
and release notes specify Microsoft's x64 redistributable prerequisite.

Native clean-machine GUI launch, original executable/bundle icon resources,
minimum-OS verification and full M5–M7/P4 acceptance remain open. This packaging
work does not resume deferred M4 aesthetics.

[Tester instructions](testing.md) explain platform requirements, unsigned-build
handling, test scope and useful bug reports. [INSTALL](../INSTALL.md) retains
local build instructions. Before publishing a candidate, verify archive contents,
matching commits/checksums, executable modes, licensing/source files and the
successful native CLI smoke checks; record the workflow and release links here.

## Plusplus development packages

The [2026-11-06 plusplus development candidate](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-plusplus-2026-11-06-10019ad)
is published from `10019ad8e781ce8dc787f28cfbaa702b91785c08` for PP3/PP4 native
acceptance. [All eight CI jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42342468720)
and [all five packaging jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42342490709)
passed. The manual packaging workflow accepts `profile: plusplus`; the
default remains `master`. Source and binaries record the selected profile in
BUILD-INFO.json. Binary packaging inherits it from the matching source and rejects
an inconsistent override. Plusplus archives use `polished-map-ng-plusplus-dev-`
names and contain `polished-map-plusplus` (with `.exe` on Windows), or a separate
`Polished Map NG++.app` with its own bundle identifier on macOS. The common CLI,
complete source, notices, checksums and offline rebuild policy remain the same.
The package's TESTING.md links to its included [plusplus tester guide](testing-plusplus.md),
preserving the guide's offline links in `docs/`.
All five archives passed checksum, exact-commit/profile, executable
architecture/mode, Mac bundle/signature-metadata and license checks. Tracked
source matches the packaged commit, with offline Cargo source configuration
appended for the archive's 453 locked registry sources. Each binary retains all 463 dependency
license/attribution records. All ten published asset sizes and digests match the
verified local files. The packaged CLI ran on each native build runner, locally
on Windows, and under WSL Ubuntu with glibc 2.35. Both Mac jobs verified their
ad-hoc bundle signatures. The packaged tester entry point and its guide links
are verified for offline use.

Direct-desktop input/drop/DPI, macOS GUI workflows and the other PP3/PP4 native
acceptance gates remain open. No plusplus parity release is claimed; aesthetics
and original-font matching remain deferred. The master candidate above remains
available separately.

This candidate supersedes the [earlier November 6 plusplus build](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-plusplus-2026-11-06-3646262),
which could crash when restoring fullscreen on Wayland. The
[startup ordering fix](compatibility.md#native-wayland-window-restart)
passes native fullscreen/maximized restart and normal-size restoration checks,
with an independent protocol regression. The downloaded Linux release binary
also opened successfully from the previously failing fullscreen settings,
canceled Map Options and exited cleanly on isolated Weston. These checks do not
close directly hosted desktop, mixed-DPI or macOS GUI acceptance.

## Final release gate

Before publishing a parity/production release:

- Complete the applicable compatibility matrix and native smoke tests.
- Include upstream and port notices, LGPL v3 and GPL v3 texts, and all required
  dependency/asset notices; verify individual asset provenance.
- Include original application/message XPMs, decoded pixels, `app.ico`,
  `crates/gui/assets/artwork/NOTICE.md` and `docs/artwork-inventory.json`. Run
  `python3 tools/artwork_assets.py` on the source archive. Package the original
  multi-resolution ICO as Windows executable/application resources; derive macOS
  bundle icon metadata from the attributed application picture. These packaging
  steps and clean-machine OS display checks remain P2/P4.
- For the imported Fugue-derived icons, include `LICENSES/CC-BY-3.0.txt`,
  `crates/gui/assets/icons/NOTICE.md`, source XPMs and the icon inventory. Preserve
  the in-app attribution and run `python3 tools/icon_assets.py` on the source
  archive. Rendered screenshots containing icons carry the same asset notices.
- Bundle the complete `crates/gui/assets/help` directory, including its original
  source, provenance, generated platform variants and LGPL notice. Run
  `python3 tools/help_assets.py` on the source archive; the converter uses only
  Python's standard library and requires no network for regeneration checks.
- Publish matching complete source, Cargo.lock, toolchain configuration, patches,
  packaging inputs and instructions to rebuild/relink with a modified core.
- Include the complete `vendor/` trees, source/adaptation manifests, readable
  patches, notices and all retained licenses, including the SCTK frame's MIT
  license. Run `python3 tools/vendor_sources.py` on the source archive; preserve
  the root Cargo patch table so rebuilding uses the same native adapters.
- Include `LICENSES/FLTK.txt` and the FLTK adaptation credit in `NOTICE.md`.
  Bundle `crates/gui/assets/fonts` and
  `LICENSES/egui-fonts/NotoSansSymbols2-OFL.txt`; verify the original font/license
  hashes with `python3 tools/font_assets.py`. The font's OFL terms remain separate
  from application code licensing.
- Verify rebuilding from the source archive; include required dependency source
  or another license-compliant means of supplying corresponding source.
- Build Windows MSVC ZIP, Linux tarball and both macOS application bundles.
- Include CLI binaries, SHA-256 checksums and actual tested OS/architecture details.
- Signing/notarization is not yet configured; never claim signed delivery.

No binaries should be presented as production-ready before these gates pass.

## Development binary size

Release builds use thin LTO, one codegen unit and explicit symbol stripping.
This preserves the existing optimization level and panic behavior. Further size
tradeoffs need measurement rather than assuming size-oriented optimization is
always smaller or appropriate for CPU rendering. See the official
[Cargo profile reference](https://doc.rust-lang.org/cargo/reference/profiles.html#strip).
Measured GUI executable sizes (bytes, uncompressed):

| Target | October 31 | November 6 |
|---|---:|---:|
| Windows x86-64 | 18,750,464 | 18,740,224 |
| Linux x86-64 | 26,835,744 | 22,501,552 |
| macOS Apple Silicon | 16,233,296 | 13,975,392 |
| macOS Intel | 17,574,448 | 15,233,216 |

These are complete-build comparisons, not an isolated optimization benchmark.
Linux is about 16% smaller; both Macs about 14%; Windows is essentially unchanged.
Further compression/size tradeoffs are deferred to measured improvements.
