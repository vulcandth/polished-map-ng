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

Each binary package carries original asset/source notices, license texts and a
lockfile-wide dependency license inventory. The matching source archive includes
the full tracked repository, all registry dependencies, lockfile, toolchain,
patches and [rebuild/relink instructions](rebuild.md). No external project/game
fixtures or ignored working files are included. Source and binaries must be
published together as a **prerelease**, with successful CI for the packaged commit.
Do not mark a development release as the latest stable release.

Windows uses an MSVC ZIP; Linux uses an Ubuntu 22.04-built tarball; macOS uses
separate app-bundle ZIPs. The Mac runner labels follow
[GitHub's runner documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job).
The current package run and downloaded archive verification are pending.
Native clean-machine GUI launch, original executable/bundle icon resources,
minimum-OS verification and full M5–M7/P4 acceptance remain open. This packaging
work does not resume deferred M4 aesthetics.

[Tester instructions](testing.md) explain platform requirements, unsigned-build
handling, test scope and useful bug reports. [INSTALL](../INSTALL.md) retains
local build instructions. Before publishing a candidate, verify archive contents,
matching commits/checksums, executable modes, licensing/source files and the
successful native CLI smoke checks; record the workflow and release links here.

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
