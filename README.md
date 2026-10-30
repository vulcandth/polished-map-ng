# Polished Map NG

A map and tileset editor being rewritten in Rust with egui/eframe, based on
[Polished Map](https://github.com/Rangi42/polished-map) by Rangi42.

**Under development.** Porting the upstream `master` editor is in progress;
`plusplus` support has not started. This is not yet a replacement for either
upstream editor. Windows, Linux and macOS are the target platforms.

See [INSTALL.md](INSTALL.md) to build and run the development version. Work on
project copies while compatibility testing is incomplete; game assets are not
included.

Browse the menus, toolbar and Help window to explore the editor. Basic controls:

| Action | Blocks mode | Events mode | Tileset editor |
|---|---|---|---|
| Click/drag | Place blocks | Move an event | Paint pixels |
| Middle drag | Scroll | Scroll | — |
| Right-click | Select a block | Edit an event | Pick a hue |
| Double-click | — | Open the event script | — |
| Ctrl+click | Replace blocks | — | Replace a hue |
| Shift+click | Flood fill | Follow a warp | Flood fill |
| Alt+click | Swap blocks | — | Swap hues |

Right-click a block in the sidebar to edit its tile definition. See the
[editing guide](docs/usage.md) for saving, specialist editors, palette colors and
event workflows, or the [CLI guide](docs/cli.md) for experimental headless editing.

Development follows the [roadmap](docs/roadmap.md). Contributors should read
[AGENTS.md](AGENTS.md) for project instructions and development documentation.

## License and credits

LGPL-3.0-only, preserving upstream attribution. See [LICENSE.md](LICENSE.md),
[NOTICE.md](NOTICE.md) and [release obligations](docs/releases.md).
Some toolbar/editor icons adapt Fugue Icons by
[Yusuke Kamiyamane](https://p.yusukekamiyamane.com/) under
[CC BY 3.0](https://creativecommons.org/licenses/by/3.0/), with Polished Map
adaptations and NG storage/disabled-state changes. See the
[icon notices](crates/gui/assets/icons/NOTICE.md); asset licenses remain separate.
