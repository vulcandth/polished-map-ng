# Native adapter source notices

## winit 0.30.13

Source: <https://github.com/rust-windowing/winit> and the immutable Cargo archive
identified in [sources.json](sources.json). The archive records VCS revision
`e9809ef54b18499bb4f2cac945719ecc2a61061b`. Original authors, package metadata,
source comments and [Apache-2.0 license](winit-0.30.13/LICENSE) are retained.

Modified for Polished Map NG on 2026-10-29, under Apache-2.0:

- Add complete native file-transfer events with ordered paths, process-local
  identity and physical client coordinates; retain legacy final path events.
- Windows: correct OLE's by-value POINTL ABI, translate the drop's screen point,
  implement IUnknown/IDropTarget queries, bound CF_HDROP allocation, and release
  STGMEDIUM through its specified owner on both hover and drop.
- macOS: capture draggingLocation in the flipped content view, then apply the
  backing scale once before emitting the complete transfer.
- X11: associate enter/position/drop with source and target windows, request the
  URI selection on drop, await its matching response (including incremental
  property transfers), bound property reads, and
  preserve filename bytes with per-URI percent decoding.
- Wayland: bind receive-only data devices on the existing connection/queue,
  retain each offer's surface/coordinates, and read URI data through bounded
  nonblocking calloop sources with a timeout and seat-removal cleanup.

The complete file-level modifications are in
[the winit patch](patches/winit-0.30.13.patch). New transfer helpers and tests are
part of that patch. It does not yet implement Wayland whole-window opacity.

## egui-winit 0.37.0

Source: <https://github.com/emilk/egui> and the Cargo archive identified in
[sources.json](sources.json). The archive records VCS revision
`7c0b40402271e8d40cda69f87cf3c2617c2e4382`; GitHub did not resolve that revision
during this audit. Archive checksums, rather than that unavailable Git revision,
are the reproducible source identity.

The archive declares MIT OR Apache-2.0 but omits the repository-root license
files referenced by its manifest. Their complete texts are included here as
[LICENSE-MIT](egui-winit-0.37.0/LICENSE-MIT) and
[LICENSE-APACHE](egui-winit-0.37.0/LICENSE-APACHE), retrieved from upstream's
immutable license revision
[`cdab9d777f6d5607614b7d171b36fa1de136c6d8`](https://github.com/emilk/egui/tree/cdab9d777f6d5607614b7d171b36fa1de136c6d8).
Original package authors, metadata and source notices remain intact.

Modified for Polished Map NG on 2026-10-29, under MIT OR Apache-2.0: translate
complete drop events into a bounded, viewport-specific egui Context queue,
converting physical coordinates using native scale × egui zoom exactly once.
The editor consumes this queue independently of path-only RawInput events.
See [the egui-winit patch](patches/egui-winit-0.37.0.patch).

These dependencies retain their own licenses; the application's LGPL-3.0-only
license does not replace them. Binary source bundles must include these patched
trees, both patches, source manifest and all retained license/notice files.
