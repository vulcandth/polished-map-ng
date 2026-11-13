# Rebuild and relink a development package

Every development release includes a complete source archive named in
`BUILD-INFO.json` (`source_archive`). Plusplus packages use a
`polished-map-ng-plusplus-dev-` prefix; master packages use `polished-map-ng-dev-`;
unified packages use `polished-map-ng-unified-dev-`.
Match its commit to `BUILD-INFO.json` in the binary package and verify the adjacent
SHA-256 files before extracting. This is complete corresponding source, including
Cargo.lock, the Rust toolchain pin, modified native dependencies, all registry
dependency sources, original/converted UI assets, licenses and packaging scripts.
No game project fixtures or system SDKs are included. The README's attributed
Cherrygrove City screenshot is the documented exception; it is not a reusable
game asset bundle.

Install Rust/rustup and the platform prerequisites in INSTALL.md. Extract the
source archive, enter its root, and run:

```text
rustup show
cargo build --offline --locked --release -p polished-map-gui -p polished-map-cli
```

The toolchain and system SDKs must already be installed for a fully offline build.
The archive's `.cargo/config.toml` uses `third-party/` for registry dependencies;
the root Cargo patches still select the modified `vendor/` native adapters.
Do not substitute unmodified crates for those patches. The normal repository
checkout instead downloads the exact dependencies in Cargo.lock.

To modify or replace the library, edit `crates/core`, `crates/project` or the
workspace dependency paths, then repeat the build. Both GUI and CLI are relinked
with your changes; there are no application-imposed signing keys or installation
locks. The executable outputs are in `target/release`. For a Mac bundle, replace
`Polished Map NG.app/Contents/MacOS/polished-map-gui` and apply a local ad-hoc
signature with `codesign --force --sign - "Polished Map NG.app"`.
For plusplus, replace `Polished Map NG++.app/Contents/MacOS/polished-map-plusplus`
and sign `Polished Map NG++.app` instead. For unified, replace
`Polished Map NG Unified.app/Contents/MacOS/polished-map-unified` and sign
`Polished Map NG Unified.app`. All three launchers are built from the same
workspace; choose the one matching the package's `profile` field.

To reproduce release packaging from Git, use a clean checkout of the exact BUILD-INFO commit,
build with `--target TARGET`, and run
`python tools/development_package.py binary --target TARGET --profile PROFILE`,
where PROFILE is `master`, `plusplus` or `unified`.
New Git packages default to `unified` if `--profile` is omitted. Commit local
changes before packaging so the archive identifies their actual source revision.
Run `python tools/development_package.py source --profile PROFILE` to regenerate
the source bundle. Source packaging requires Git metadata. Binary packaging from
an extracted source archive inherits its profile and commit from BUILD-INFO.json
and rejects a conflicting profile override. The extracted source can be compiled
and modified without Git. Rebuilds need not be bit-for-bit identical
across SDK/compiler/host environments. See BUILD-INFO for the original toolchain
and host and LICENSE.md/NOTICE.md for licensing and attribution.
