# Release and compliance checklist

Development artifacts are not parity releases. The user authorized tester
packages before native/master acceptance on 2026-10-31. Those packages follow
the development policy below; the full checklist remains the final release gate.

## Development packages

| Application | Current development download | Testing guide |
|---|---|---|
| Unified editor | [November 11 unified build](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-unified-2026-11-11-6d8a2ea) | [Unified testing](testing-unified.md) |
| Fixed master parity | [November 7 master build](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-2026-11-07-618aefd) | [Master testing](testing.md) |
| Fixed plusplus parity | [November 7 plusplus build](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-plusplus-2026-11-07-618aefd) | [Plusplus testing](testing-plusplus.md) |

[The manual workflow](../.github/workflows/development.yml) creates a complete
source archive, then builds Windows x86-64, Linux x86-64, Apple Silicon and Intel
Mac packages **offline from that archive**. Registry source is vendored separately
from our patched native dependencies. All four jobs run the packaged CLI and
the platform home-discovery tests. BUILD-INFO identifies the immutable commit,
toolchain, build host, target and signing status. Each archive has a SHA-256 file.
Mac bundles have local ad-hoc signatures, not Developer ID signing/notarization.

Fonts, icons and Help are embedded in the executable; no neighboring asset files
are required to run it. Binary packages retain asset notices, license texts and a
dependency license inventory for the selected GUI/CLI production graph and target.
Build generators and procedural macros are retained conservatively; test-only
dependencies, test features and unrelated platform dependencies are excluded.
Published-crate omissions are covered by [pinned license supplements](../LICENSES/dependency-supplements/README.md),
including original authorship metadata and independently verified hashes.
The matching source archive includes
the full tracked repository (including original artwork and converted pixels),
all registry dependencies, lockfile, toolchain,
patches and [rebuild/relink instructions](rebuild.md). No external project/game
fixtures or ignored working files are included. Source and binaries must be
published together as a **prerelease**, with successful CI for the packaged commit.
Do not mark a development release as the latest stable release.

### Binary content policy

Binaries ship one selected GUI launcher (a bundle on macOS), the headless CLI,
BUILD-INFO, TESTING, INSTALL and REBUILD, the editing/menu/preferences/CLI and
tester guides, project/asset/vendor attribution, complete applicable license
texts and a dependency notice inventory. Fonts, Help and UI images are embedded;
no loose runtime asset tree is needed. Shared supplemental license texts are
stored once under `LICENSES/dependency-supplements`; each inventory record points
to those paths relative to `LICENSES`. The record's `notices` paths are relative
to `LICENSES/dependencies/NAME-VERSION`.

Internal roadmap, design/compatibility evidence, test fixtures, screenshots,
source code, tools and build caches do not belong in binary archives. User help
links to shipped documents work offline; links to source-only documents are
rebased to the exact commit on GitHub. Project notice text is preserved with
only those link destinations adjusted; dependency and license text bytes remain
unchanged. The complete source archive retains the originals, the full locked
dependency sources, patch records, development tools and rebuild instructions.
It also retains the specifically authorized README screenshot with attribution.
That is not authorization to bundle game-project fixtures.

Packaging from Git rejects a dirty working tree; fresh packages default to the
unified profile. Binary packages built from the source archive inherit its
commit/profile. Source repackaging from an extracted archive is rejected so a
containing Git checkout cannot silently supply unrelated source. Mac bundle
versions come from the workspace manifest. The
[package regressions](../tools/test_development_package.py) check the explicit
content policy, offline links, immutable source links, notice hashes, dependency
selection, profile identity and executable modes. The production notice graph
uses Cargo's [normal and build edges](https://doc.rust-lang.org/cargo/commands/cargo-tree.html#feature-unification),
without all-feature/test expansion. This is a conservative attribution inventory,
not a claim that every selected crate contributes machine code to the executable.

### Initial-release archive audit

The published November 11 unified archives at `6d8a2ea173db` were downloaded and
their SHA-256 files and BUILD-INFO revision checked. The four binary archives
contained 942–944 files, all 470 lockfile-wide package notice records and the
entire internal docs tree. Their Markdown had 214 unresolved local links per
archive. The source archive contained 23,209 files (about 498 MB unpacked), of
which about 483 MB was the complete offline registry source. That source is
retained for rebuild/relink delivery, not copied into binaries.

The audit also found that binaries lacked `vendor/NOTICE.md` and its explicit
`vendor/licenses/SCTK-MIT.txt` location. The same complete Victor Berger MIT text
was already present in dependency notices; the defect was the missing adaptation
notice/link, not an absent copy of the MIT terms. The binary content policy now
includes both vendor records explicitly.

The completed [R3 review](roadmap.md#initial-release-review--complete) verified
`1bf228cbb222337a42e347816b4be82944d47447`. All eight
[CI jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/43347427575)
and all five [source/platform packaging jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/43347427189)
passed. The four binaries were built offline from the same source archive.
These are verification artifacts; this review has not replaced the published
November 11 downloads or declared an initial release.

| Verified target | Files | Dependency notice records | Compressed bytes | Support bytes excluding GUI/CLI executables |
|---|---:|---:|---:|---:|
| Windows x86-64 | 452 | 232 | 10,222,292 | 2,653,949 |
| Linux x86-64 | 581 | 308 | 11,470,440 | 3,225,325 |
| macOS Apple Silicon | 441 | 229 | 8,668,289 | 2,515,640 |
| macOS Intel | 441 | 229 | 9,531,514 | 2,515,640 |

Independent inspection checked each archive's SHA-256, BUILD-INFO, selected
launcher, native executable architecture/modes, explicit content allowlist,
offline help links, exact curated/dependency license bytes and supplemental
notice hashes. Test-only notice entries and internal documentation are absent.
Windows support contents fell from 5,638,950 to 2,653,949 bytes. This reduces
distribution overhead; it is not a claim of reduced application memory use.
The packaged Windows CLI ran against a synthetic map. Both macOS packaging
jobs verified their ad-hoc signatures; Developer ID signing and native user
acceptance remain separate gates.

The corresponding source contains 23,223 files (498,563,139 bytes unpacked).
Its 936 tracked files match the commit exactly; all 460 registry source packages
were checked against their lockfile archive checksums and individual file hashes.
Generated offline Cargo configuration, BUILD-INFO and the exact copied REBUILD
instructions account for the additional non-registry files. Complete source,
lockfile, licenses and relink instructions remain available together; registry
sources are deliberately retained for offline rebuilding.

The packaging workflow can build all three launchers from the same shared-source
revision. Select `master`, `plusplus` or `unified` explicitly; the profile in BUILD-INFO.json,
launcher, preferences, Mac bundle identity and tester guide remain separate.
Using a shared revision carries native adapter fixes into every tester build.
The unified launcher adapts to storage context without silently converting files.
The Git branch name
alone does not determine which editor is packaged.

Windows uses an MSVC ZIP; Linux uses an Ubuntu 22.04-built tarball; macOS uses
separate app-bundle ZIPs. The Mac runner labels follow
[GitHub's runner documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job).
ZIP packaging clamps reproducible dependency timestamps to ZIP's supported
range while preserving license bytes and executable modes. The regression in
[the archive test](../tools/test_development_package.py) covers the pre-1980
notice timestamps found during the first Mac package run.

The [November 7 review-fix master candidate](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-2026-11-07-618aefd)
is published from shared-source revision `618aefd47f29bd6b0025ff3db6d0faf5a1e5c454`.
It includes all [M9 review fixes](compatibility.md#november-7-upstream-review),
including bounded roof/tileset previews and the original Windows multi-size icons.
[All eight CI jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42426696741)
and [all five packaging jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42426711300)
passed for this exact revision. All five downloaded archives passed SHA-256,
commit/profile, source-content, architecture/mode, Mac bundle/signature-metadata
and license checks. All 460 vendored registry sources match Cargo.lock and their
file checksums; each binary retains 470 dependency notice/attribution records.
Both Windows launchers' five embedded ICO frames match the original artwork.
All ten published asset sizes/digests match the audited local files. The shipped
CLI ran on all four build runners and locally on Windows and WSL Ubuntu; both
Mac jobs verified their ad-hoc signatures. Native GUI acceptance remains open.
General aesthetics, original fonts and future feature suggestions remain deferred.

### Earlier master candidates (historical)

The [2026-11-07 master development candidate](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-2026-11-07-7e5fc4d)
is published from shared-source revision `7e5fc4dccf01e5897a3cf120e7ba3e53472c937c`,
with the master launcher, settings and bundle identity. It includes the shared
native input, dialog, clipboard, fullscreen-startup and keyboard-focus fixes.
[All eight CI jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42385507344)
and [all five packaging jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42385522286)
passed. All five archives passed checksum, exact-commit/profile, executable
architecture/mode, Mac bundle/signature-metadata and license checks. The tracked
source matches the packaged revision, apart from the appended offline Cargo
configuration. All 453 registry sources match Cargo.lock and their vendor file
checksums; each binary retains 463 dependency license/attribution records.
All ten published asset sizes and digests match the verified local files.
The packaged CLI ran on all four native build runners, locally on Windows and
under WSL Ubuntu. Both Mac jobs verified their ad-hoc bundle signatures.
These package checks do not close native GUI acceptance; no new manual GUI
verification is claimed for this delivery. Aesthetics remain deferred.

Earlier historical package evidence follows.

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

Native clean-machine GUI launch, macOS bundle icon metadata,
minimum-OS verification and full M5–M7/P4 acceptance remain open. This packaging
work does not resume deferred M4 aesthetics.

[Tester instructions](testing.md) explain platform requirements, unsigned-build
handling, test scope and useful bug reports. [INSTALL](../INSTALL.md) retains
local build instructions. Before publishing a candidate, verify archive contents,
matching commits/checksums, executable modes, licensing/source files and the
successful native CLI smoke checks; record the workflow and release links here.

## Plusplus development packages

The [November 7 review-fix plusplus candidate](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-plusplus-2026-11-07-618aefd)
is published from the same `618aefd47f29bd6b0025ff3db6d0faf5a1e5c454` revision as
master, with a separate launcher, preferences and Mac bundle identity. It includes
all M9 review fixes and preserves the earlier plusplus functionality.
[All eight CI jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42426696741)
and [all five packaging jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42426713272)
passed. Its five archives and ten published asset digests passed the same full
source, checksum, profile, architecture, executable-mode, signature-metadata,
Windows ICO and license audit as master: 460 vendored registry sources and 470
dependency notice records. The shipped CLI ran on all native build runners,
locally on Windows and in WSL Ubuntu. Both Mac signatures were verified by the
native packaging jobs. PP3/PP4 native GUI gates remain open; these are development
packages, not parity releases. The [plusplus tester guide](testing-plusplus.md)
and matching source/rebuild instructions are included.

### Earlier plusplus candidates (historical)

The [2026-11-07 plusplus development candidate](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-plusplus-2026-11-07-7e5fc4d)
is published from `7e5fc4dccf01e5897a3cf120e7ba3e53472c937c` for PP3/PP4 native
acceptance, using the same shared-source revision as the master candidate above.
[All eight CI jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42385507344)
and [all five packaging jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42385529178)
passed. The manual packaging workflow accepts `profile: plusplus`; the
default was `master` at that delivery. Following the unified promotion to `main`,
the workflow defaults to `unified`. Source and binaries record the selected profile in
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
appended for the archive's 453 locked registry sources. Every file listed in their
Cargo vendor checksums matches, and each package checksum matches Cargo.lock.
Each binary retains all 463 dependency
license/attribution records. All ten published asset sizes and digests match the
verified local files. The packaged CLI ran on each native build runner, locally
on Windows, and under WSL Ubuntu with glibc 2.35. Both Mac jobs verified their
ad-hoc bundle signatures. The packaged tester entry point and its guide links
are verified for offline use.

Direct-desktop input/drop/DPI, macOS GUI workflows and the other PP3/PP4 native
acceptance gates remain open. No plusplus parity release is claimed; aesthetics
and original-font matching remain deferred. The master candidate above remains
available separately.

This candidate replaces the [earlier November 7 plusplus packages](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-plusplus-2026-11-07-fc18238)
with a fresh build paired with the master profile. It retains the same functional
fixes; no new GUI acceptance is claimed for this packaging refresh.

This candidate supersedes the [earlier November 6 plusplus build](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-plusplus-2026-11-06-3646262),
which could crash when restoring fullscreen on Wayland. The
[startup ordering fix](compatibility.md#native-wayland-window-restart)
passes native fullscreen/maximized restart and normal-size restoration checks,
with an independent protocol regression. The downloaded `10019ad` Linux release binary
also opened successfully from the previously failing fullscreen settings,
canceled Map Options and exited cleanly on isolated Weston. These checks do not
close directly hosted desktop, mixed-DPI or macOS GUI acceptance.

This candidate also supersedes [the `10019ad` build](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-plusplus-2026-11-06-10019ad).
It includes the [frame-focus fix](compatibility.md#native-wayland-frame-focus-and-placement)
for keyboard shortcuts lost after a Wayland client-titlebar click. The independent
protocol regression and native F11/Alt+F checks pass. The remaining compositor
focus/placement limitations are recorded separately in that evidence; these
packages do not close broader native acceptance.

## Final release gate

Before publishing a parity/production release:

- Complete the applicable compatibility matrix and native smoke tests.
- Include upstream and port notices, LGPL v3 and GPL v3 texts, and all required
  dependency/asset notices; verify individual asset provenance.
- Include original application/message XPMs, decoded pixels, `app.ico`,
  `crates/gui/assets/artwork/NOTICE.md` and `docs/artwork-inventory.json`. Run
  `python3 tools/artwork_assets.py` on the source archive. Package the original
  multi-resolution ICO as Windows executable/application resources (implemented
  in M9 and verified byte-for-byte by `tools/windows_icon_resources.py`); derive
  macOS bundle icon metadata from the attributed application picture. macOS
  metadata and clean-machine OS display checks remain P2/P4.
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

## Unified development packaging

The [latest unified development build](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-unified-2026-11-11-6d8a2ea)
is published from `6d8a2ea173db791a2a83f4f560de56d6059d0a92` for Windows x86-64,
Linux x86-64, Apple Silicon and Intel macOS. It includes the
[November 10–11 review corrections](compatibility.md#november-1011-unified-review):
aligned Project options, unclipped multiline event fields, theme-matched menus
and a toolbar-consistent status bar. The earlier Hex field alignment and
November 8–9 fixes are retained; `paletteswap` remains deferred.
All eight [CI jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/43130466411)
and all five [packaging jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/43139014277)
passed. This is a development prerelease, not a parity or latest stable release.

Complete matching source, licenses, lockfile, rebuild/relink instructions and
five SHA-256 files accompany the four platform archives. The source audit checks
every tracked file against Git and all 460 vendored registry sources against
Cargo.lock/checksums. Binary audits check architecture, executable modes,
launcher/commit identity, tester instructions, notices, 470 dependency records,
macOS bundle/signature metadata and all five Windows icon resources. No external
project/game fixtures are distributed. All ten uploaded asset sizes and digests
match the audited files; the release tag resolves to the exact source commit.

Each binary was built offline from the matching source archive. Packaged CLI
smoke checks pass on all native build runners and locally on Windows/WSL. macOS
bundles are ad-hoc signed, not Developer ID signed or notarized. The older Intel
Mac startup/Print retest and broader native acceptance remain with human testers;
the historical native GUI checks below do not establish acceptance of this build.
The memory allocation hint has no measured RAM savings yet; see the
[footprint investigation](#november-9-unified-footprint-investigation).

The previous `603d038` unified prerelease was deleted at the maintainer's request
after this replacement was published and audited. The four older unified
prereleases (`0350275`, `66ca4b6`, `5332c83` and `4edcbc6`) had already been
removed. Their Git tags and source history remain; separate master/plusplus
releases are unchanged. Historical evidence below refers to older commits, not
currently downloadable binary releases.

### Historical November 8 candidate

The [November 8 unified development build](https://github.com/vulcandth/polished-map-ng/commit/66ca4b6cd528d0859b30b6b8ba05291cd00ccd6b)
was published from `66ca4b6cd528d0859b30b6b8ba05291cd00ccd6b` for Windows x86-64,
Linux x86-64, Apple Silicon and Intel macOS. Complete source and all five SHA-256
files accompanied the four binaries. The retained tag resolves to that exact
commit; all ten uploaded sizes and server-reported digests matched the locally
verified files. It was a prerelease and was not marked latest stable.

This refresh fixes native Ctrl+C/Cmd+C in Print Options. Print and block editing
reuse a helper that handles translated Copy events and raw shortcuts without
duplicate consumption. Native Windows Print exports and clipboard images saved
by Paint match exactly under classic and attributed storage; the downloaded
package also passes the Ctrl+C check. See
[native and regression evidence](compatibility.md#unified-windows-native-print-exchange).
The fixed parity launchers remain in the shared source; their previously
published packages remain available separately for comparison.

The Development packages workflow accepts `unified` in addition to the two
parity launchers. It packages `polished-map-unified`, uses the macOS bundle name
`Polished Map NG Unified.app` and a separate bundle identifier, and includes the
[unified tester guide](testing-unified.md). Archive names include `-unified-`;
the source archive and each binary must agree on their launcher and commit.
All three launchers remain in source, and a missing selected executable fails
packaging rather than substituting another launcher.

All five [package jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42488164390)
and all eight [CI jobs](https://github.com/vulcandth/polished-map-ng/actions/runs/42488165621)
passed for this commit. The source archive matches Git and all 460 vendored registry
source checksums. Every binary archive passes architecture, executable mode,
launcher identity, tester-guide, notices and 470-entry dependency inventory checks;
Mac bundle/signature metadata and all five Windows icon resources are verified.
The binaries are rebuilt offline from the matching source archive. Nine packaging
regression tests cover all targets and mismatch rejection.

The packaged CLI runs on all native build runners and locally on Windows/WSL.
The November 8 Windows package reopens the generated mixed-event map through Recent,
retains all five Print flags, copies successfully with Ctrl+C and closes normally.
The Linux package also passes [X11 original-app clipboard comparisons and
file-manager drops](compatibility.md#unified-linux-x11-original-app-exchange-and-file-drops)
under Xvfb/WSL, including classic → attributed → classic transitions in one
running app. These checks do not establish window-manager, portal, mixed-DPI
or Wayland acceptance. Broader Windows evidence includes the
[five-project cycle](compatibility.md#unified-windows-five-project-native-checks),
[mixed-storage saves](compatibility.md#unified-windows-mixed-storage-native-checks)
and [event/clipboard workflows](compatibility.md#unified-windows-event-and-image-clipboard-checks).
Full native acceptance remains open; no parity or clean-machine release claim is
made. Offline source, licensing, lockfile and checksum requirements apply unchanged.

The [previous unified release, `5332c83`](https://github.com/vulcandth/polished-map-ng/commit/5332c83e74676a95e07b6ceebcb22658513460e1),
preserved compound tileset context in Save As and rejected ambiguous
plain/compound filename hints using the current inventory. Its native compound
reopen check is retained in the [mixed-storage evidence](compatibility.md#unified-windows-mixed-storage-native-checks).
The November 8 release retains those changes and adds the Print shortcut correction.

The [first unified release, `4edcbc6`](https://github.com/vulcandth/polished-map-ng/commit/4edcbc6289b4d7d84729c419d5a05e92dc906670),
is retained in source history. It corrected stale simple Save As hints but did
not retain compound tileset hints. Candidate `1baa7a1` passed CI but was not
published: its packages were superseded by the shared plain/compound hint guard.
The earlier `254d57e` candidate was also not published after native verification
found stale Save As hints.

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


### November 9 unified footprint investigation

The published Windows `66ca4b6` unified executable is 19,182,080 bytes
(18.29 MiB): PE sections contain 13,565,952 bytes of machine code and
5,209,600 bytes of read-only data, with the rest in unwind/resource/relocation
sections and headers. The release profile already uses thin LTO, one codegen
unit and symbol stripping. This is not an accidentally distributed debug build.
Rust dependencies, egui/eframe, wgpu, shader translation, platform adapters and
bundled fonts contribute to the linked executable. These section sizes do not
attribute an exact number of bytes to each dependency. Both format adapters
share one GUI stack; the unified binary does not embed two independent apps.

The refreshed `0350275` Windows executable is 19,213,312 bytes (18.32 MiB),
essentially unchanged after the review fixes. The allocator hint targets runtime
allocation; no executable-size reduction is claimed.

The reported 165 MB versus 14.9 MB process memory is not yet a controlled
measurement. The newer GPU renderer introduces driver/resource allocations,
font atlases and rendering caches absent from the original FLTK design. Resident
memory, private committed memory and GPU memory are different measurements;
a single Task Manager reading does not establish a leak or an exact breakdown.

The small review change selects wgpu 30.0.1
[`MemoryHints::MemoryUsage`](https://docs.rs/wgpu/30.0.1/wgpu/enum.MemoryHints.html),
keeping every backend and the existing limits. In wgpu-hal's allocator policy,
initial device/host block sizes change from 128/64 MiB to 8/4 MiB. Those are
allocation-policy parameters, **not measured process savings**; some backends
ignore the hint. No before/after resident-memory or latency result is claimed.
Human testing should compare the same release configuration, project, map and
window size on one machine, after idle and after repeated map changes.

The current block renderer also uploads one 32×32 texture per block. An atlas
is a concrete profiling candidate for reducing per-texture overhead, but is not
part of this small policy change.

Further work, if warranted by measurement, is a dependency size profile and a
comparison of size-oriented compiler optimization against render/edit speed.
Changing renderer, removing backends or deleting font coverage is deferred;
compatibility takes priority over matching the original executable size.
