# Preference persistence

The implemented settings follow the pinned master's `Main_Window` constructor,
`exit_cb` and `Resize_Dialog::initialize`. Settings are captured by eframe's
shutdown `App::save` hook. Periodic autosave is disabled to retain the source's
exit-time behavior. A canceled exit leaves the current session running.

## Stored settings and defaults

| Setting | Default |
|---|---|
| Theme | Greybird; Dark is the other supported initial choice |
| Transparent | Off; requests 75% whole-window opacity |
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
| 128-block warning acknowledged (`meta128`) | Off; set only when its warning is dismissed |
| Overworld Map Size | 1300 bytes; accepted range 1–4096 |
| Drag and Drop | On; internal block, tile, roof and palette transfers |
| Print Grid, IDs, Priority, Events, Warp IDs | All off; independent of editor view flags |

The Theme submenu retains the existing v1 `dark` boolean (false for Greybird,
true for Dark), so earlier settings need no migration. Choosing either theme
updates its radio mark and the shared UI style without changing document colors.

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

Saving a blockset with collisions and more than 128 blocks shows the source
warning after success, until acknowledged. OK, Enter and Escape all acknowledge
it. The `meta128` setting is retained across maps and saved at normal shutdown,
matching the source preference flush lifecycle; merely displaying the warning
does not change it. Older records without the field default to false. Save
continuations and their dialogs are transient and never restored.

Documents, unsaved colors, palette dirty state, undo history, clipboard contents,
selection, scroll positions and open dialogs are not serialized. The selected
palette set is saved, but the color bank starts from defaults at the next launch.
Print options are retained even when Cancel or Escape dismisses the form, as in
the source. Canceling the subsequent file chooser also retains those values.
Print jobs, output images and success dialogs are transient and are not stored.
egui's general widget-memory persistence is disabled. Native window placement,
maximized state and fullscreen use eframe's separate built-in persistence,
including its monitor-bound checks. Full Screen defaults off on a fresh launch.
The View menu and F11 send viewport requests; the menu reads the reported native
state. No duplicate fullscreen value is stored in the application JSON record.
The patched adapter tracks normal root-window geometry on move, resize and scale
changes, independently of periodic saves. Child windows and minimized/fullscreen/
maximized geometry do not replace it. Shutdown while minimized also retains the
normal geometry. It is restored before applying the saved native state. Tracker
regressions are automated; actual compositor transitions and exit/restart behavior
remain a native acceptance gate.

## Format and location

Transparent uses the application JSON boolean `transparent`. An enabled startup
preference queues one native request after the window exists; an opaque startup
does not change its styles. The menu reflects accepted requests. Failed changes
leave that state unchanged; a failed startup application preserves the saved
intent so moving between desktop backends does not erase it. Errors are reported
once per request, without an automatic retry loop. Native acceptance does not
prove that an X11 compositor honors its opacity property. The old JSON record
without this field restores the default off value.

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

## Legacy import

When no NG `app.ron` exists, startup imports the first selected original store,
read-only. An existing NG file takes precedence even if malformed or from a newer
version. There is no merging and no write to a legacy file or directory.

1. `polishedmap.prefs` beside the NG executable.
2. `Polished Map.prefs` beside that executable.
3. The original user's `Rangi/Polished Map.prefs` store below.

| Platform | Original user-store root |
|---|---|
| Windows | Current redirected roaming AppData, queried through the shell |
| macOS | `~/Library/Preferences` |
| Linux | `$XDG_CONFIG_HOME`, otherwise `~/.config`; falls back to `~/.fltk` only if the modern `Rangi` directory does not exist and the old one does |

On Linux the **vendor directory**, not the preference file, decides the modern/
old precedence. XDG `~/`, `${HOME}` and `$HOME/` expansion follows pinned FLTK.
Home discovery uses Rust's OS home-directory lookup. The original Mac fallback
through `NSHomeDirectory` when an explicitly configured HOME is nonexistent is
not reproduced; unusual account/home configurations remain a native check.
To import an executable-adjacent store from an older installation, copy it beside
the NG executable before first launch. NG does not search other installations.

The reader supports FLTK 1.0 root groups, repeated keys, continuation lines,
CRLF, escaped backslashes/newlines/octal bytes and source integer-prefix parsing.
Recent paths retain Unix bytes or valid UTF-8 on Windows. All settings in the
above table, ten recent paths, normal window geometry, fullscreen and maximized
state are mapped. Themes other than Greybird/Dark fall back to Greybird with a
startup message, consistent with the initial two-theme scope.

Normal geometry is created first; fullscreen (preferred over maximized) is
requested after the first paint and native visibility. Removed-monitor placement
is corrected where absolute positioning is available; Wayland placement belongs
to the compositor. Size is bounded and the adapter clamps it to monitor limits.
Mixed-DPI coordinates and real window-manager restoration still need M6 evidence.

Invalid headers, oversized files, malformed escapes, integer overflow and
out-of-range settings reject the entire import. Defaults are used, an error is
reported, and writes to NG's application preference key are disabled for that
session. Legacy bytes remain untouched. Eframe may still create its own window
record, so retrying an import requires closing NG and backing up/renaming its
`app.ron` after correcting the source. This is an explicit first-launch migration,
not a continuous synchronization mechanism. See [intentional safety differences](deviations.md#legacy-preference-import-safety).

## Remaining parity and verification

Legacy discovery/import, the `meta128` workflow and application-record roundtrip
are implemented. M5 remains in progress for native storage/restart/shutdown,
placement and restored fullscreen/maximization/transparency; M6 supplies the
integrated platform evidence. Remaining theme/font aesthetics are deferred by
user direction and are not a functional acceptance requirement.

Tests inject an in-memory eframe Storage into the real app constructor and use
its save callback. They cover all implemented settings, 270 palette/layout/anchor
combinations, defaults, malformed/future-record retention, exclusion of edited
documents/colors, first-launch precedence and byte-preserving legacy reads.
Parser tests cover source selection, continuations, duplicate keys, escaped
paths and malformed input; Unix adds non-UTF-8 paths. The exact vendored geometry
tracker is tested for resize/move followed by maximize/fullscreen/minimize,
child-window isolation and invalid-size rejection without an intervening save.
These tests do not establish native filesystem shutdown delivery, compositor
placement or integrated restart behavior on Windows, Linux and macOS.
