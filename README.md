# Polished Map NG

A map and tileset editor being rewritten in Rust with egui/eframe, based on
[Polished Map](https://github.com/Rangi42/polished-map) by Rangi42.

**Development version; compatibility is still being verified.** Windows, Linux
and macOS are the target platforms. See the [roadmap](docs/roadmap.md) for supported
scope and remaining work.

See [INSTALL.md](INSTALL.md) to build and run the development version. Work on
project copies while compatibility testing is incomplete; project game assets are not
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

Contributors should read [AGENTS.md](AGENTS.md) for project instructions and
development documentation.

## License and credits

LGPL-3.0-only, preserving upstream attribution. See [LICENSE.md](LICENSE.md),
[NOTICE.md](NOTICE.md) and [release obligations](docs/releases.md).
Original UI artwork is retained from
[Polished Map](https://github.com/Rangi42/polished-map/tree/8b9bdc2b8aac8c865809516c2123af69c498da0b/res);
see the [artwork notices](crates/gui/assets/artwork/NOTICE.md).
Some toolbar/editor icons adapt Fugue Icons by
[Yusuke Kamiyamane](https://p.yusukekamiyamane.com/) under
[CC BY 3.0](https://creativecommons.org/licenses/by/3.0/), with Polished Map
adaptations and NG storage/disabled-state changes. See the
[icon notices](crates/gui/assets/icons/NOTICE.md); asset licenses remain separate.
