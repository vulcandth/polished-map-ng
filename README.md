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
adapter supports master tilesets with PNG/2bpp/LZ/CHR graphics, binary
metatiles, tilepal mappings, all six palette sets, project/special palettes and
map-group roof colors, compound graphics and optional roof tile replacement.
Block tile/collision editing is available; events and the other specialist editors
are not complete. Work on copies
while parity validation is incomplete.

```text
cargo run -p polished-map-cli -- create --root path/to/project --width 20 --height 18 --output path/to/project/maps/New.20x18.blk --apply
cargo run -p polished-map-cli -- inspect path/to/Map.blk --width 20 --height 18
cargo run -p polished-map-cli -- render path/to/pokecrystal/maps/NewBarkTown.blk --output map.png
cargo run -p polished-map-cli -- render path/to/pokecrystal/maps/NewBarkTown.blk --palette nite --output night.png
cargo run -p polished-map-cli -- render path/to/pokecrystal/maps/NewBarkTown.blk --roof violet --output violet-roof.png
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

Open/New options accept an optional roof name from `gfx/tilesets/roofs`, without
the extension. Leave it empty to use the base tileset. Compound names such as
`main.before,after` retain the full main filename and load optional companion
graphics, with upstream's 16-tile row padding. Tileset pixel editing and saving
use the shared backend in both the GUI and CLI.

Edit → Edit Tileset (Ctrl/Cmd+T) opens a draft. Click/drag to paint; Shift fills,
Ctrl replaces a hue, Alt swaps hues, and right-click picks a hue. Keys 1–4 choose
white/light/dark/black; Alt+1–8 chooses a palette. Ctrl/Cmd+C/V/X copies a tile
snapshot, pastes it, or swaps with its live source. Delete clears a tile. Drag
tiles to swap; Ctrl-drag copies. The image buttons and Ctrl/Cmd+Shift+C/V use the
system image clipboard, with grayscale export and bounded RGB import. OK commits
the draft and marks the tileset dirty even if unchanged; Cancel discards it.
Tile edits have no map undo entries. File → Save Tileset saves PNG/palette files;
Save also processes the map and blockset. Native clipboard transport and child
window behavior remain unverified; the current dialog is embedded.

With a roof loaded, Edit → Edit Roof (Ctrl/Cmd+F) opens its nine-tile draft.
It shares pixel gestures and clipboard commands, with the original reversed
display rows and no palette/delete controls. File → Save Roof writes only its
24×24 two-bit grayscale PNG; Save processes it after the map, blockset and tileset.
Roof dirty state is independent of base graphics. Accepting a roof draft does
not add map undo entries. Image paste affects only the selected 8×8 tile;
larger images are clipped and smaller images retain uncovered pixels.

Tools → Change Roof (Ctrl/Cmd+Shift+H) switches the loaded overlay or removes it
with `(none)`. Unsaved roof changes require confirmation; canceling either dialog
preserves them. Map, blockset, base tileset and palette edits stay intact. Selecting
the current roof reloads it from disk; a failed load retains the previous roof and
preview. Successful replacement preserves the original sticky roof dirty flag;
removing the roof clears that domain. Switching alone writes no files.

Right-click a sidebar block (or choose Edit → Edit Block) to edit its 4×4 tile
definition. Left-click paints and right-click picks a tile. Shift/Ctrl/Alt select
the original eight placement patterns. OK applies the draft; Cancel discards it.
Block edits have separate dirty state and no map undo, matching upstream.
File → Save Blockset writes definitions and loaded collisions; Save also writes
the map. Named collision fields or binary hex spinners follow the project format.
Drag a sidebar block onto another to swap them; Ctrl-drag copies instead.
Edit → Copy Block stores a snapshot; Paste Block pastes that snapshot, while
Swap Block swaps with the copied source's current contents. Ctrl/Cmd+C/V/X
invoke these internal block operations; they do not replace the system clipboard.
Resize Blockset (Ctrl/Cmd+B) accepts 1–256 blocks. New definitions/collisions
are empty, and shrinking does not rewrite map IDs. These operations have no
map undo entries. Native clipboard shortcut verification remains pending.

Headless blockset commands support `set`, `paint`, `copy`, `swap`, `resize` and
`set_collisions`. Use `--collisions path/to/collision.asm` (or `.bin`) to attach
collision data. Copy/swap/resize carry that data with the block:

```text
cargo run -p polished-map-cli -- inspect-blockset path/to/johto_metatiles.bin
cargo run -p polished-map-cli -- edit-blockset path/to/johto_metatiles.bin --commands blocks.json
cargo run -p polished-map-cli -- edit-blockset path/to/johto_metatiles.bin --commands blocks.json --apply
```

Example `blocks.json`:
`[{"op":"paint","block":0,"at":{"x":0,"y":0},"tile":32,"brush":{"ctrl":true}}]`.
Preview is the default. `--expected-hash` checks the input; `--output` saves a
binary copy, requiring `--expected-output-hash` before overwriting an existing
destination. Imported `.CEL` files require a binary copy so assembly is preserved.

Example collision command:
`{"op":"set_collisions","block":0,"collision":{"format":"asm","values":["FLOOR","FLOOR","WALL","WALL"]}}`.
Binary rows use `"format":"binary"` and four byte values. `--expected-collision-hash`
checks collision input. Paired copies also require `--output-collisions`;
overwrites require its `--expected-output-collision-hash` from preview.

Headless tileset commands support `paint`, `fill`, `replace`, `swap_hues`,
`palette`, `set`, `copy`, `swap`, `delete` and `import_rgb`:

```text
cargo run -p polished-map-cli -- inspect-tileset johto --root path/to/project
cargo run -p polished-map-cli -- edit-tileset johto --root path/to/project --commands tiles.json
cargo run -p polished-map-cli -- edit-tileset johto --root path/to/project --commands tiles.json --apply --expected-hash HASH_FROM_PREVIEW
```

Example `tiles.json`:
`[{"op":"paint","tile":0,"points":[{"x":0,"y":0}],"hue":2},{"op":"palette","tile":0,"palette":3}]`.
Hues are white=0, dark=1, light=2, black=3. Palette IDs are 0–7, with bit 7
for priority. Preview returns the edited draft and every planned destination,
byte count and hash. Apply requires `previous_hash` from preview (or
`content_hash` from inspect), binding the inputs, destinations, block definitions
and mode to the observed local paths. It is not a portable asset identifier or
the hash of the unsaved draft. Use the same `--allow-256`/`--monochrome` flags
for preview and apply.

Roof commands use file-order indices 0–8 (master slots $0A–$12), independent of
the reversed GUI rows. They accept the same pixel/copy commands except `palette`
and `delete`; palette attributes are contextual and are not saved in roof PNGs.
Standalone inspection/editing therefore reports pixels only:

```text
cargo run -p polished-map-cli -- inspect-roof violet --root path/to/project
cargo run -p polished-map-cli -- edit-roof violet --root path/to/project --commands roof.json
cargo run -p polished-map-cli -- edit-roof violet --root path/to/project --commands roof.json --apply --expected-hash HASH_FROM_PREVIEW
```

The roof hash binds its observed source and PNG destination. Preview does not
write; apply requires the observed hash. Existing 2bpp/CHR/LZ sources are retained.

Headless palette-color editing imports the original RGB macro layouts into all
six sets. Commands are `set_rgb`, `channel`, `hex`, `copy` and `swap`. A color
slot names `set` (`morn`, `day`, `nite`, `darkness`, `indoor`, `custom`), `palette`
(0–7 game colors, 8 monochrome), and internal `hue` (0 white, 1 dark, 2 light,
3 black). Channel controls use 0–31; hex and `set_rgb` preserve exact RGB8.
Copy quantizes to RGB5 as upstream does; swap preserves exact live colors.

```text
cargo run -p polished-map-cli -- inspect-palettes path/to/bg_tiles.pal
cargo run -p polished-map-cli -- edit-palettes path/to/bg_tiles.pal --commands colors.json --output edited.pal
cargo run -p polished-map-cli -- edit-palettes path/to/bg_tiles.pal --commands colors.json --output edited.pal --apply --expected-hash HASH_FROM_PREVIEW
```

Example `colors.json`:
`[{"op":"hex","slot":{"set":"day","palette":6,"hue":2},"value":"037ff9"}]`.
Preview includes the six RGB8 sets and exact `export_rgb` text. Export writes all
six sets as RGB5; add `--monochrome` to write their monochrome palettes only.
Apply requires an explicit output and observed input hash; replacing an existing
output also requires `--expected-output-hash` from preview. Export quantization
does not mutate the draft, so its fingerprint can differ after reopening.
Palette files are exported separately from map/tileset saves.

Edit → Edit Current Palettes (Ctrl/Cmd+Shift+L) opens the color or monochrome
draft. Select a swatch, use the 0–31 RGB controls, or enter hex and press Enter.
Ctrl/Cmd+C/V/X copy, paste, and swap swatches when a text field is not focused.
OK accepts; Cancel discards. View → Palettes selects any of the six sets.
File → Load Palettes (Ctrl/Cmd+L) prompts before replacing edited colors;
Export Palettes writes them independently. As upstream, palette edits alone
do not mark the map dirty or trigger its close warning: export them explicitly.
Windows' preview swatch opens the native color chooser. Linux and macOS have
the same RGB/hex editing controls; a shared egui graphical picker is planned
for the improvement phase after parity.

Saving uses upstream's PNG destinations, including separate compound outputs,
and writes two-bit grayscale PNGs. Imported 2bpp/CHR/LZ files remain intact;
subsequent loads prefer the new PNG. Roof overlays remain separate. Palette-map
ASM retains unrelated content and comments; ambiguous structural rewrites fail
before any file is changed. Monochrome mode saves pixels only, so palette changes
in that mode do not persist. Unsafe upstream compound export layouts are rejected.

Block/collision and tileset saves stage before/after bytes and a manifest in a temporary
`.polished-map-recovery-*` directory beside the first output. Normal completion or
successful rollback removes it. Interrupted saves or failed rollback can leave
it for inspection and manual recovery; preserve it until recovery is resolved.
Automatic journal discovery/replay is still pending. Unrelated collision ASM
lines and comments are preserved.

## License

LGPL-3.0-only, preserving upstream attribution. See [NOTICE.md](NOTICE.md),
[LICENSE.md](LICENSE.md), and [release obligations](docs/releases.md).
