# Release and compliance checklist

Development artifacts are not parity releases. Before publishing binaries:

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
