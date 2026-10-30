# Building and running Polished Map NG

These instructions build the development version. Compatibility acceptance and
distributable packages remain unfinished; see the [roadmap](docs/roadmap.md).

## Prerequisites

Install Rust with rustup. The repository's [rust-toolchain.toml](rust-toolchain.toml)
selects the development toolchain, and `Cargo.lock` pins dependencies. The
workspace's minimum supported Rust version is checked separately in CI.

- Windows: install Visual Studio Build Tools with Desktop development with C++
  and a Windows SDK; use the MSVC Rust toolchain.
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
Redistribution must also satisfy the [release requirements](docs/releases.md).
