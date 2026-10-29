# Preference persistence

The implemented settings follow the pinned master's `Main_Window` constructor,
`exit_cb` and `Resize_Dialog::initialize`. Settings are captured by eframe's
shutdown `App::save` hook. Periodic autosave is disabled to retain the source's
exit-time behavior. A canceled exit leaves the current session running.

## Stored settings and defaults

| Setting | Default |
|---|---|
| Portable dark appearance | Off (light) |
| Editing mode | Blocks |
| Grid | On |
| Rulers, zoom, block IDs, hexadecimal, Game Boy screen | Off |
| Show Events, Show Warp IDs | On |
| Selected palette set | Day |
| Roof Palettes layout | Morn + Day, Night |
| Auto-Load Events, Auto-Load Special Palettes, Auto-Load Roof Colors | On |
| Resize anchor | Center |
| Tile Priority | Off; reading PRIORITY_* palette colors enables it |
| Monochrome, 256 Tiles | Off; missing palette maps enable Monochrome |
| Show Priority | On; control enabled only while Tile Priority is on |
| Recent maps | Empty; at most ten paths |
| Overworld Map Size | 1300 bytes; accepted range 1–4096 |

The three auto-load toggles affect subsequent opens, not the current document.
Special palettes and roof colors are independent; turning either off skips
that optional import entirely, including its parsing and prompts. Manual palette
and roof-color imports remain available. The default project palette import is
independent of these toggles. Contextual palette selection on a later map open
continues to follow upstream; restoring a selected set does not pin it forever.

Overworld Map Size controls a post-save warning when the map's padded byte count
exceeds the configured WRAM capacity. Changing it does not resize the map, edit
WRAM source or immediately warn. Cancel retains the previous value; Close keeps
the accepted value. Out-of-range persisted values preserve the original settings
record and restore defaults, like other invalid preferences.

Documents, unsaved colors, palette dirty state, undo history, clipboard contents,
selection, scroll positions and open dialogs are not serialized. The selected
palette set is saved, but the color bank starts from defaults at the next launch.
egui's general widget-memory persistence is disabled. Native window placement
uses eframe's separate built-in persistence, including its monitor-bound checks.

## Format and location

The stable app ID is `polished-map-ng`, independent of the development window
title. eframe 0.37 stores `app.ron` in its platform data directory:

| Platform | Default path |
|---|---|
| Windows | `%APPDATA%\polished-map-ng\data\app.ron` |
| Linux | `$XDG_DATA_HOME/polished-map-ng/app.ron`, or `~/.local/share/polished-map-ng/app.ron` if no absolute XDG data directory is configured |
| macOS | `~/Library/Application Support/polished-map-ng/app.ron` |

The `polished-map-ng.preferences` storage key contains a version-1 JSON object.
Its named enum values and booleans are independent of the native window record.
Missing fields receive defaults. Unknown versions, invalid values, unknown
fields or records exceeding 4 MiB restore defaults and disable writes to that
key for the session, preserving the existing record. Startup status reports this.
This is an internal settings schema, not the stable automation API promised by
the roadmap.

Recent paths use tagged UTF-8 strings, with native Unix bytes or Windows UTF-16
units for names that cannot be represented as UTF-8. Native encodings from the
other OS are rejected and the existing record is preserved. The ten-entry limit,
nonempty paths and exact duplicate exclusion are checked on restoration. Startup
does not probe paths or reopen a map. The larger record cap accommodates long
paths; writing also checks the cap before replacing the saved record.

The outer RON file is owned by eframe. Unreadable/malformed outer files and write
errors follow its logged, best-effort behavior; this is not a power-loss-safe
transaction or recovery journal. To reset settings, close the app and back up
or rename its `app.ron` before relaunching.

## Remaining parity and verification

The legacy FLTK user store and executable-adjacent `.prefs` files are not imported
or modified. Native theme variants, transparency/fullscreen controls,
drag-and-drop and print preferences still need their complete menu/persistence work.

Tests inject an in-memory eframe Storage into the real app constructor and use
its save callback. They cover restoration of all implemented settings, 270
palette/layout/anchor combinations, defaults, malformed/future-record retention,
and exclusion of edited documents/colors. Menu tests exercise all four special/
roof auto-load combinations and prove disabled imports skip malformed files.
Actual filesystem persistence, window restoration and shutdown delivery still
require native checks on Windows, Linux and macOS. Headless tests do not prove
those platform behaviors.
