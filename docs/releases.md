# Releases and distribution

Development packages are not parity or production releases. Native acceptance
and final delivery gates remain open in the [roadmap](roadmap.md). The links below
identify published builds; later source/CI artifacts are not automatically releases.

## Development packages

| Application | Current published build | Source revision | Tester guide |
|---|---|---|---|
| Unified | [November 11](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-unified-2026-11-11-6d8a2ea) | `6d8a2ea173db791a2a83f4f560de56d6059d0a92` | [Unified](testing-unified.md) |
| Fixed master | [November 7](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-2026-11-07-618aefd) | `618aefd47f29bd6b0025ff3db6d0faf5a1e5c454` | [Master](testing.md) |
| Fixed plusplus | [November 7](https://github.com/vulcandth/polished-map-ng/releases/tag/dev-plusplus-2026-11-07-618aefd) | `618aefd47f29bd6b0025ff3db6d0faf5a1e5c454` | [Plusplus](testing-plusplus.md) |

Each release provides Windows x86-64 MSVC ZIP, Linux x86-64 tarball, Apple Silicon
and Intel macOS app-bundle ZIPs, plus matching complete source and SHA-256 files.
See [platform prerequisites and verification](testing.md). Older release narratives
remain in Git history and [GitHub release notes](https://github.com/vulcandth/polished-map-ng/releases).

## Unified development packaging

The published unified build contains the November 10–11 Project options, event
field, menu and status-bar fixes. Its
[CI](https://github.com/vulcandth/polished-map-ng/actions/runs/43130466411) and
[packaging](https://github.com/vulcandth/polished-map-ng/actions/runs/43139014277)
passed. Source/archives, uploaded digests, executable architectures, Windows
icons and macOS bundle signatures were audited. It uses the older full-lockfile
notice inventory; the streamlined policy below applies to subsequent builds.
Previous unified prereleases were removed at the maintainer's request; their
source history remains. Human acceptance is still open.

## Plusplus development packages

Master and plusplus were built from the same shared-source revision with separate
launchers, settings and macOS identities. The November 7 review fixes passed
[shared CI](https://github.com/vulcandth/polished-map-ng/actions/runs/42426696741),
[master packaging](https://github.com/vulcandth/polished-map-ng/actions/runs/42426711300)
and [plusplus packaging](https://github.com/vulcandth/polished-map-ng/actions/runs/42426713272).
All platform archives and matching source passed identity/hash/license checks;
packaged CLI smoke checks and macOS ad-hoc signature verification passed on their
build hosts. This does not close either launcher's native parity gates.

## Creating a development package

The [manual workflow](../.github/workflows/development.yml) creates complete source
and builds all four targets **offline from that archive**. It defaults to unified;
explicit `master` and `plusplus` profiles remain supported. The branch name does
not select a profile. Source and binaries must agree on profile and exact commit.
BUILD-INFO records toolchain, build host, target, source archive and signing status.
Run the [rebuild/packaging commands](rebuild.md) from a clean, committed checkout.

Each candidate must pass CI and package checks before publication as a prerelease.
Verify archive contents, hashes, source identity, executable modes/architecture,
notices, Windows resources, macOS bundle metadata and native CLI smoke checks.
Publish matching source and binaries together; do not mark development builds
latest stable. Windows packages have no publisher certificate; macOS has local
ad-hoc signatures only, without Developer ID signing or notarization.

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

The November 12 review found internal/test-only documentation, broken local help
links and redundant license copies in the November 11 binaries. It also restored
explicit vendor adaptation notices: the complete SCTK MIT terms were already
present elsewhere, but their adaptation notice/link was missing. The current
package policy fixes these issues without removing corresponding source.

The completed [R3 review](roadmap.md#initial-release-review) verified
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

## Final release gate

Before a parity/production release:

- Close applicable compatibility/native/clean-machine gates, including minimum OS
  claims and both Mac architectures. Keep deferred aesthetics separate from
  functional acceptance. macOS bundle icon metadata remains P2 work.
- Retain project/upstream attribution, LGPL v3 and GPL v3 texts, and applicable
  dependency/asset notices. Keep the [original-artwork rights limits](assets.md)
  visible; attribution does not independently establish third-party permission.
- Publish matching complete source with Cargo.lock, toolchain, all registry
  sources, native patches/manifests, original/converted UI assets and rebuild/relink
  instructions. Preserve the root Cargo patch table and verify an offline rebuild.
- Verify source assets using `artwork_assets.py`, `icon_assets.py`, `font_assets.py`
  and `help_assets.py`; verify patches using `vendor_sources.py`. Retain source
  inventories, original Help, font/FLTK/Fugue/SCTK terms and modification notices.
  Binary packages retain embedded assets and notices under the policy above;
  game-project fixtures are excluded from both, except the authorized README image
  in source documentation.
- Verify all four archives and checksums, packaged CLI, Windows multi-size icon
  resources, executable permissions and macOS bundle metadata. Record the actual
  tested OS/architecture and signing status. Never claim signing/notarization that
  has not occurred.

## Development binary size

Release builds use thin LTO, one codegen unit and symbol stripping, as configured
in [Cargo.toml](../Cargo.toml). Source bundle size, executable size and process
memory measure different things. The archive audit above measures packaging;
it does not establish runtime savings.

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
