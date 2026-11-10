# Testing unified development builds

The unified application selects editing controls from the opened map and its
tileset. There is no mode selector. The separate [master](testing.md) and
[plusplus](testing-plusplus.md) launchers remain available for parity comparisons.
These are development builds; native acceptance is still open. See
[release availability and evidence](releases.md) before downloading a package.

Human testers own the remaining native acceptance checks, per the November 8
handoff. Use the checklist below and report functional issues; the handoff does
not mark untested platforms or workflows as passing.

For the November 8–9 fixes, prioritize these checks:

- On the older Intel Mac, launch normally without the previous manual environment
  workaround. With Events enabled in Print, verify Copy and Export and the labels
  in both outputs. Report the macOS version and selected font names on failure.
- Alternate classic and attributed maps. Verify both Options groups stay visible,
  only applicable controls work, and Project options identifies the format.
  Check inferred Roof palettes and an explicit override; Cancel must preserve the
  active map and its settings.
- Check long tileset names, large dropdown lists, full-sheet previews, numeric
  text alignment, F9 and the filename/project-root title at your normal DPI.
- For memory comparisons, record the GPU/backend, project/map, window dimensions,
  memory metric, idle value and value after repeated map changes. Compare matching
  conditions; the allocator change has no promised reduction in resident RAM.

| Platform | Package suffix | Start the editor |
|---|---|---|
| Windows x86-64 | `x86_64-pc-windows-msvc.zip` | `polished-map-unified.exe` |
| Linux x86-64 | `x86_64-unknown-linux-gnu.tar.gz` | `./polished-map-unified` |
| macOS Apple Silicon | `aarch64-apple-darwin.zip` | `Polished Map NG Unified.app` |
| macOS Intel | `x86_64-apple-darwin.zip` | `Polished Map NG Unified.app` |

Check that `BUILD-INFO.json` says `"profile": "unified"` and that its commit
matches the complete source archive. Platform dependencies, checksum verification
and unsigned/ad-hoc signing limitations are described in the
[platform setup guide](testing.md). Packages contain no game projects. The included
CLI remains experimental; use its documented commands in [CLI usage](cli.md).
Rebuilding and relinking are covered in [REBUILD.md](rebuild.md).

Use disposable copies of pokered, pokeyellow, pokegold, pokecrystal and Polished
Crystal. For reproducible results use the commits in
[upstream.lock.toml](../upstream.lock.toml); record any different revision.

1. Open a representative map in each project, alternating between Gen 1, classic
   Gen 2 and Polished Crystal in one session. Check graphics, events, palette
   selection, roof overlays where present, and controls appropriate to its assets.
   Cancel an Open and try a malformed asset; the previous document should survive.
2. Edit map blocks, block contents, supported collisions, tile pixels, roofs,
   palettes and event coordinates. Save, close and reopen. Inspect Git differences
   and compare supported outputs with the corresponding original application.
   Confirm unrelated files and source text remain unchanged.
3. Open a project with mixed classic and attributed tilesets. Change Tileset in
   both directions. Map edits, undo history and events should survive; each storage
   model retains its palette session. A storage change requires an explicit Save
   As destination. The suggested name includes current dimensions and tileset
   hints where representable; save and reopen it to check automatic detection.
   It does not convert existing map headers or asset files.
4. Exercise dirty-document Cancel/Discard decisions and external-write conflicts.
   Missing dimensions or ambiguous tileset evidence should require corrected
   context. Do not accept guessed dimensions for short maps without checking the
   project; the [compatibility evidence](compatibility.md#five-project-compatibility-matrix)
   describes known pinned-project cases.
5. Check native file dialogs, clipboard exchange, desktop drag/drop, focus and
   keyboard shortcuts, settings restart, and moving between monitors at different
   scales. Record Windows, Linux X11/Wayland and both macOS architectures separately.

Report the build commit, OS/backend, display scale, project revision, exact steps,
expected and observed behavior in a
[bug report](https://github.com/vulcandth/polished-map-ng/issues). Attach only
redistributable samples. Aesthetics and original-font matching remain deferred.
Passing automated tests or launching successfully does not close
[unified native acceptance](roadmap.md#3-unified-editor--in-progress).
