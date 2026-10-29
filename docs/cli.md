# Experimental CLI guide

Build and run instructions are in [INSTALL.md](../INSTALL.md). The Rust API and
JSON responses are experimental; stabilization is objective 4 of the
[roadmap](roadmap.md). Run a subcommand with `--help` for its complete options.
Examples use placeholder paths and hashes: substitute values from your project
and the corresponding inspection or preview response.

## Maps and rendering

```text
cargo run -p polished-map-cli -- create --root path/to/project --width 20 --height 18 --output path/to/project/maps/New.20x18.blk --apply
cargo run -p polished-map-cli -- inspect path/to/Map.blk --width 20 --height 18
cargo run -p polished-map-cli -- render path/to/pokecrystal/maps/NewBarkTown.blk --output map.png
cargo run -p polished-map-cli -- render path/to/pokecrystal/maps/NewBarkTown.blk --output print.png --print-grid --print-ids --print-events --print-warp-ids
cargo run -p polished-map-cli -- render path/to/pokecrystal/maps/NewBarkTown.blk --palette nite --output night.png
cargo run -p polished-map-cli -- render path/to/pokecrystal/maps/NewBarkTown.blk --roof violet --output violet-roof.png
cargo run -p polished-map-cli -- edit path/to/Map.blk --width 20 --height 18 --commands edits.json
cargo run -p polished-map-cli -- edit path/to/Map.blk --width 20 --height 18 --commands edits.json --apply
cargo run -p polished-map-cli -- render path/to/Map.20x18.johto.blk --root path/to/project --tileset johto --output map.png
cargo run -p polished-map-cli -- save-as path/to/Imported.map --width 20 --height 18 --output path/to/Imported.20x18.blk --apply
```

Example `edits.json`: `[{"op":"paint","points":[{"x":2,"y":3}],"block":7}]`.

Resize example: `[{"op":"resize","width":21,"height":19,"anchor":"center"}]`.
All nine anchors are supported; new cells use block zero and resizing clears
map history. Update project dimension constants and filename dimensions separately.

PNG loading follows the original platform's behavior. All platforms discard the
low byte of 16-bit samples; Windows additionally clears RGB for fully transparent
expanded RGBA pixels, while Linux/macOS retain it. For reproducible automation,
pass `--png-platform unix` or `--png-platform windows` to `render`,
`inspect-tileset`, `edit-tileset`, `inspect-roof` or `edit-roof`. The default is
the current host. JSON results report the resolved `png_platform`; tileset/roof
preflight hashes include it, so preview and apply must use the same choice.
Rust callers use `MasterGraphicsOptions::png_platform` or
`RoofDocument::open_in_project_with_png_platform`.

`render` defaults to plain pixels. Print overlays are independent:
`--print-grid`, `--print-ids`, `--print-priority`, `--print-events` and
`--print-warp-ids` (warp IDs require `--print-events`). `--print-hex` controls block IDs; warp IDs retain the event's
coordinate base. `--print-dark` selects the dark grid color. Events are discovered
from the project, or supplied with `--events-file` alongside `--print-events`.
Labels use installed bold fonts, without a window or GPU. Supply both
`--print-mono-font` and `--print-sans-font` with TTF/OTF files to select explicit
faces (the CLI uses face zero; the Rust API also accepts collection indices and
validated variable-font coordinates). Print shares the editor's font policy;
GUI jobs retain the startup faces. JSON identifies the fonts, instance coordinates
and explicit substitutions. Windows currently substitutes Arial Bold for the
reference's synthetic Microsoft Sans Serif Bold; native synthesis is unfinished.
Linux system discovery requires
Fontconfig and installed fonts; explicit font data bypasses discovery. Native
font rasterization parity is still pending. New PNG exports are staged and never
replace an existing destination, including one created during rendering.
Edit defaults to preview; `--apply` writes after all commands validate. `--expected-hash`
can require the original hash from `inspect`. JSON responses are experimental until
objective four. Map saving currently affects only the map file.

`create` and `save-as` also default to preview. Replacing an existing destination
requires `--expected-output-hash` from the preview response. New Map starts dirty
in memory and Save opens Save As. ASM imports always save to a binary `.blk` copy.

## Blocksets and collisions

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

## Tilesets

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

## Roofs

Roof commands use file-order indices 0–8 (master slots $0A–$12), independent of
the reversed GUI rows. They accept the same pixel/copy commands except `palette`
and `delete`; palette attributes are contextual and are not saved in roof PNGs.
Standalone inspection/editing therefore reports pixels only:

```text
cargo run -p polished-map-cli -- inspect-roof violet --root path/to/project
cargo run -p polished-map-cli -- edit-roof violet --root path/to/project --commands roof.json
cargo run -p polished-map-cli -- edit-roof violet --root path/to/project --commands roof.json --apply --expected-hash HASH_FROM_PREVIEW
```

The roof hash binds its observed source, PNG destination and PNG platform policy.
Preview does not write; apply requires the observed hash. Existing 2bpp/CHR/LZ
sources are retained.

## Palette colors

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

Headless map opens start from default palette colors. Rust callers can explicitly
share a session with `MasterTilesetDocument::begin_for_map` and advance its
`MapPalettePlan`; the GUI carries its current palette session between maps.

## Events

Headless event editing operates on an explicit map script, without opening a map
or creating a window:

```text
cargo run -p polished-map-cli -- inspect-events path/to/project/maps/NewBarkTown.asm
cargo run -p polished-map-cli -- edit-events path/to/project/maps/NewBarkTown.asm --commands events.json
cargo run -p polished-map-cli -- edit-events path/to/project/maps/NewBarkTown.asm --commands events.json --apply --expected-hash HASH_FROM_PREVIEW
```

Commands use zero-based event indices from inspection, for example
`[{"op":"move","index":0,"x":4,"y":7}]`. Coordinates are 16px event cells,
bounded to -6..249. `edit` also accepts optional `prefix` and `suffix` strings
for fields already present in that macro. `translate` takes `block_dx` and
`block_dy` in 32px map blocks, doubles them and clamps event coordinates; it does
not resize a map. GUI map resizing updates the loaded event draft too. Events have no
undo history in the original compatibility model.

Inspection includes source line numbers, macro kinds, one-based warp IDs and
parsed warp destinations. All 22 master macros and legacy X/Y orders are
recognized. Decimal, `$` hexadecimal and `%` binary coordinate literals follow
the original byte wrapping; unsupported expressions fail explicitly. The parser
distinguishes pokered `warp` events from pokecrystal `warp` script commands.
Saving preserves unrelated assembly, untouched event lines, indentation and
line endings. Commands validate as a batch before mutation, preview writes
nothing, and apply requires the preview's content hash. The Rust entry points
are `Events`, `EventCommand`, `EventScript`, `EventDocument` and
`event_script_path`; these APIs and JSON output remain experimental.

## Persistence and recovery

The CLI uses the shared persistence backend. See [saving and recovery](usage.md#saving-and-recovery)
for output formats, preserved source files and interrupted-save journals. General
automatic recovery and a stable batch contract remain roadmap work; do not treat
the current commands as a filesystem-wide transaction.
