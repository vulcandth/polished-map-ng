# Building and running Polished Map NG

[Download development packages](docs/releases.md#development-packages)
for compatibility testing; see
[tester instructions](docs/testing.md) and the [roadmap](docs/roadmap.md).
The instructions below build the development version from source.

## Prerequisites

Install Rust with rustup. The repository's [rust-toolchain.toml](rust-toolchain.toml)
selects the development toolchain, and `Cargo.lock` pins dependencies. The
workspace's minimum supported Rust version is checked separately in CI.

- Windows: install Visual Studio Build Tools with Desktop development with C++
  and a Windows SDK; use the MSVC Rust toolchain. The SDK resource compiler
  embeds the original multi-resolution application icon during the GUI build.
- macOS: install Xcode Command Line Tools (`xcode-select --install`).
- Linux: install a C/C++ toolchain, pkg-config, X11/Wayland development libraries
  and fonts. On Ubuntu 24.04, the CI dependency set is:

```sh
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libxkbcommon-dev libxkbcommon-x11-0 libwayland-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev libgl1-mesa-dev fonts-liberation xvfb xauth
```

The Linux GUI needs a working X11 or Wayland desktop and graphics driver.
The X11 backend also loads `libxkbcommon-x11.so.0` at runtime; installing only
`libxkbcommon-dev` does not supply that separate library on a minimal system.
Installed-font discovery uses Fontconfig; headless rendering can instead accept
explicit font files. CI compilation/tests do not establish complete native desktop
compatibility on any platform.

## Build and run

From a local checkout of this repository:

```text
rustup show
cargo run -p polished-map-gui --locked --
cargo run -p polished-map-cli --locked -- --help
```

The default GUI is master. To launch the separate experimental plusplus editor:

```text
cargo run -p polished-map-gui --bin polished-map-plusplus --locked --
```

Both parity launchers accept an optional map path after `--`, retain separate
settings and stay fixed to their respective formats. Their parity acceptance is
still in progress; master and plusplus tester packages are published separately.

The in-progress unified application is also available from source:

```text
cargo run -p polished-map-gui --bin polished-map-unified --locked --
```

It detects storage from the map and selected tileset, uses its own settings, and
has no mode selector. Open/New, legacy source inputs, alternating-project saves
and automatic tileset changes have automated coverage, including representative
asset edits and saves in all five target projects. Integrated/native acceptance
and unified package delivery remain open. Follow the [unified roadmap](docs/roadmap.md#3-unified-editor--in-progress)
before using this development launcher for acceptance testing.

For optimized binaries:

```text
cargo build --release --locked -p polished-map-gui -p polished-map-cli
```

Cargo places binaries under `target/release`. Open a project you supply; game
assets are not bundled. See the [editing guide](docs/usage.md) and
[CLI guide](docs/cli.md) for project options and commands.

## Development checks

Follow [AGENTS.md](AGENTS.md) for required checks. The normal workspace suite is:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo doc --workspace --no-deps --locked
```

Additional reference, real-project, visual, native-adapter and license checks are
defined in [CI](.github/workflows/ci.yml). Their fixtures and prerequisites differ
from the normal suite; see [compatibility evidence](docs/compatibility.md).
`python tools/fetch_project_fixtures.py` fetches the five immutable project
fixtures from `upstream.lock.toml` into ignored `.reference/` checkouts. It refuses
to change an existing checkout at another revision or with tracked edits. The
`real-project` CI job lists the environment variables and exact test commands.
Redistribution must also satisfy the [release requirements](docs/releases.md).

Native modal lifetime checks create hidden windows and do not operate the desktop.
On Windows PowerShell run:

```powershell
$env:POLISHED_MAP_NATIVE_MODAL_PROBE = '1'
cargo run -p polished-map-platform --locked --example native_modals
```

On macOS use `POLISHED_MAP_NATIVE_MODAL_PROBE=1 cargo run -p polished-map-platform --locked --example native_modals`.
Linux uses isolated display/protocol peers:

```sh
xvfb-run --auto-servernum env POLISHED_MAP_NATIVE_MODAL_PROBE=1 cargo run -p polished-map-platform --locked --example native_modals
python3 tools/wayland_modal_probe.py
```

These check ownership/lifetime and wire behavior. Integrated native desktop
acceptance remains a separate roadmap gate.

The opt-in native settings probe launches short-lived windows in separate
processes and uses only temporary legacy/application stores. Run on an isolated
Linux display (the CI matrix runs the same probe natively on Windows/macOS):

```sh
xvfb-run --auto-servernum env -u WAYLAND_DISPLAY POLISHED_MAP_NATIVE_PREFERENCES_PROBE=1 cargo run -p polished-map-gui --example native_preferences --features native-tests --locked
```

For a local Windows/macOS run, set `POLISHED_MAP_NATIVE_PREFERENCES_PROBE=1` and
run the same Cargo command without `xvfb-run`/`env`. Expect ten brief application
windows; no desktop input is sent. The `native-tests` feature is disabled in
normal builds. See [scope and results](docs/compatibility.md#native-profile-preference-lifecycle).
