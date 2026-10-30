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
  preserve filename bytes with per-URI percent decoding. Preserve a pending
  drop's captured point across later position messages and use numeric X11 None
  for rejected actions, as required by XDND.
- Wayland: bind receive-only data devices on the existing connection/queue,
  retain each offer's surface/coordinates, and read URI data through bounded
  nonblocking calloop sources with a timeout and seat/window-removal cleanup.
  Own wl_data_device/offer lifecycle directly so completed asynchronous receives
  survive a subsequent enter/leave. Keep up to 16 independent receives per seat;
  leave clipboard ownership with its separate adapter. SCTK still supplies
  manager binding and its public pipe helper; its hover ownership is not reused.
- Wayland opacity: bind alpha-modifier on that same connection/queue, multiply
  the root and all client-frame surfaces, reject unsupported/server-frame cases,
  and restore opaque state with a capability-loss event on frame transitions.

The complete file-level modifications are in
[the winit patch](patches/winit-0.30.13.patch). New transfer helpers and tests are
part of that patch. The frame adaptation below retains its separate MIT license.

Additional modifications on 2026-10-30, under Apache-2.0: scoped modal ownership
retains live parent/child windows; Windows restores the owner's prior enabled
state before child destruction; AppKit attaches/detaches floating child windows;
X11 supplies transient/modal hints; Wayland uses the existing connection for
xdg_toplevel parent relationships and optional xdg-dialog hints. Protocol objects
are released before their toplevels. Application input filtering remains required.

## Smithay client toolkit 0.19.2 frame adaptation

The Wayland fallback frame in `winit-0.30.13/src/platform_impl/linux/wayland/types/opacity_frame.rs`
adapts `src/shell/xdg/fallback_frame.rs` from the immutable Cargo archive of
[Smithay client toolkit](https://github.com/Smithay/client-toolkit), copyright
(c) 2018 Victor Berger, under [MIT](licenses/SCTK-MIT.txt). Its complete original
license is retained verbatim. [adaptations.json](adaptations.json) records the
archive checksum, original/adapted file hashes and license hash; the
[separate frame patch](patches/sctk-0.19.2-opacity-frame.patch) isolates NG's changes.

Modified for Polished Map NG on 2026-10-29: externalize toolkit imports; attach
one alpha-modifier resource per frame surface; synchronize opacity commits with
the parent; restore resources when frame parts are recreated; destroy alpha
resources before surfaces. Original drawing, input and geometry remain intact.
The production graph uses this fallback frame. The optional Adwaita frame is
not adapted and explicitly rejects opacity while visible.

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
The same patch adds a single outstanding opacity request/reply per viewport,
dispatches it through the live owned Wayland window, and reports capability loss.
See [the egui-winit patch](patches/egui-winit-0.37.0.patch).

Additional modifications on 2026-10-30, under MIT OR Apache-2.0: a context-local
weak window registry and scoped modal leases connect viewport IDs to live winit
windows. Renderer-state destruction releases the leases before windows die;
replacement registrations reject stale teardown. Parent input is filtered without
discarding lifecycle events, and native calls run outside egui's data lock.

## eframe 0.37.0

Source: <https://github.com/emilk/egui> and the Cargo archive identified in
[sources.json](sources.json). The archive records VCS revision
`3c6eb8261c3293ad9c7ea1ed58487e038860ebef`; the verified archive checksum is the
source identity. All original files are retained. Like egui-winit, this archive
omits repository-root license texts; [LICENSE-MIT](eframe-0.37.0/LICENSE-MIT) and
[LICENSE-APACHE](eframe-0.37.0/LICENSE-APACHE) reproduce the same immutable upstream
license texts identified above, with original copyrights intact.

Modified on 2026-10-30, under MIT OR Apache-2.0: three registration calls in the
WGPU/Glow native integrations associate renderer-owned window Arcs with their
egui-winit State. No renderer or public App API is replaced. See the complete
[eframe patch](patches/eframe-0.37.0.patch).

These dependencies retain their own licenses; the application's LGPL-3.0-only
license does not replace them. Binary source bundles must include these patched
trees, all patches, both manifests and all retained license/notice files.
