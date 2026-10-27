# Release and compliance checklist

Development artifacts are not parity releases. Before publishing binaries:

- Complete the applicable compatibility matrix and native smoke tests.
- Include upstream and port notices, LGPL v3 and GPL v3 texts, and all required
  dependency/asset notices; verify individual asset provenance.
- Publish matching complete source, Cargo.lock, toolchain configuration, patches,
  packaging inputs and instructions to rebuild/relink with a modified core.
- Verify rebuilding from the source archive; include required dependency source
  or another license-compliant means of supplying corresponding source.
- Build Windows MSVC ZIP, Linux tarball and both macOS application bundles.
- Include CLI binaries, SHA-256 checksums and actual tested OS/architecture details.
- Signing/notarization is not yet configured; never claim signed delivery.

No binaries should be presented as production-ready before these gates pass.
