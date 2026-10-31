# Rebuild and relink a development package

Every development release includes `polished-map-ng-dev-COMMIT-source.tar.gz`.
Match its commit to `BUILD-INFO.json` in the binary package and verify the adjacent
SHA-256 files before extracting. This is complete corresponding source, including
Cargo.lock, the Rust toolchain pin, modified native dependencies, all registry
dependency sources, original/converted UI assets, licenses and packaging scripts.
No game project assets or system SDKs are included.

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

To reproduce release packaging from Git, check out the exact BUILD-INFO commit,
build with `--target TARGET`, and run
`python tools/development_package.py binary --target TARGET`.
Run `python tools/development_package.py source` to regenerate the source bundle.
These packaging commands require Git metadata; the extracted source archive can
be compiled and modified without Git. Rebuilds need not be bit-for-bit identical
across SDK/compiler/host environments. See BUILD-INFO for the original toolchain
and host and LICENSE.md/NOTICE.md for licensing and attribution.
