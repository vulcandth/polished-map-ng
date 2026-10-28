# Polished Map NG

A Rust rewrite of [Polished Map](https://github.com/Rangi42/polished-map),
using egui/eframe. **Under development; not yet a replacement for either
upstream editor.** See [compatibility status](docs/compatibility.md).

The roadmap preserves master and plusplus behavior before unifying them and
stabilizing a GUI-independent Rust library and JSON CLI. Windows, Linux and
macOS are intended targets. See [the full roadmap](docs/roadmap.md),
[architecture](docs/architecture.md), and [contributor instructions](AGENTS.md).

## Development

Install Rust through rustup and the native platform C/C++ build tools. The
repository pins the toolchain; Cargo.lock pins dependencies. On Linux, install
the X11/Wayland development dependencies required by eframe and rfd.

```text
cargo test --workspace --locked
cargo run -p polished-map-gui --
cargo run -p polished-map-cli -- --help
```

The initial slice opens binary maps using project metadata, explicit dimensions,
or names such as `Map.20x18.johto.blk`. It finds the nearest Makefile and recognizes
master map constants/headers and numbered tileset aliases. The GUI prefills these
options for review. Shared block files still require explicit choices. The project
adapter supports simple master tilesets with PNG/2bpp/LZ/CHR graphics, binary
metatiles, tilepal mappings, all six palette sets, project/special palettes and
map-group roof colors. Compound graphics, roof tile replacement, collisions, events and
specialist editors are not complete. Work on copies
while parity validation is incomplete.

```text
cargo run -p polished-map-cli -- create --root path/to/project --width 20 --height 18 --output path/to/project/maps/New.20x18.blk --apply
cargo run -p polished-map-cli -- inspect path/to/Map.blk --width 20 --height 18
cargo run -p polished-map-cli -- render path/to/pokecrystal/maps/NewBarkTown.blk --output map.png
cargo run -p polished-map-cli -- render path/to/pokecrystal/maps/NewBarkTown.blk --palette nite --output night.png
cargo run -p polished-map-cli -- edit path/to/Map.blk --width 20 --height 18 --commands edits.json
cargo run -p polished-map-cli -- edit path/to/Map.blk --width 20 --height 18 --commands edits.json --apply
cargo run -p polished-map-cli -- render path/to/Map.20x18.johto.blk --root path/to/project --tileset johto --output map.png
cargo run -p polished-map-cli -- save-as path/to/Imported.map --width 20 --height 18 --output path/to/Imported.20x18.blk --apply
```

Example `edits.json`: `[{"op":"paint","points":[{"x":2,"y":3}],"block":7}]`.
Edit defaults to preview; `--apply` writes after all commands validate. `--expected-hash`
can require the original hash from `inspect`. JSON responses are experimental until
objective four. Map saving currently affects only the map file.

`create` and `save-as` also default to preview. Replacing an existing destination
requires `--expected-output-hash` from the preview response. New Map starts dirty
in memory and Save opens Save As. ASM imports always save to a binary `.blk` copy.

Resize is available in Edit → Resize Map and through commands such as
`{"op":"resize","width":21,"height":19,"anchor":"center"}`. All nine anchors
are supported. Resizing fills new cells with block zero and clears undo history,
matching upstream. Dimension constants, filename dimensions and event coordinates
must be updated separately; event integration is still pending.

Mouse controls implemented: paint/drag, right-click select, Shift fill, Ctrl
replace, Alt swap. One drag is one history entry. Ctrl/Cmd+O/S/Z/Y opens, saves,
undoes and redoes. Ctrl/Cmd+N creates a new map; Ctrl/Cmd+Shift+S opens Save As.
Print exports an unannotated PNG; clipboard export is pending.

## License

LGPL-3.0-only, preserving upstream attribution. See [NOTICE.md](NOTICE.md),
[LICENSE.md](LICENSE.md), and [release obligations](docs/releases.md).
