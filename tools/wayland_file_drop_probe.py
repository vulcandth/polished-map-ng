"""Linux native file-drop protocol acceptance on a private socket, without a GUI.

The actual winit adapter receives real wl_data_device events and SCM_RIGHTS
pipes. The peer supplies adversarial timing/data and checks lifetime/negotiation;
the Rust probe checks delivered paths, coordinates, identity and responsiveness.
This does not replace real file-manager, compositor or mixed-DPI acceptance.
"""
import os
import sys
import time

from wayland_opacity_probe import Peer, build_probe, run_scenario

MAX_BYTES = 4 * 1024 * 1024
VALID = b"# ordered transfer\r\nfile:///tmp/one%20space.blk\r\nfile://localhost/tmp/%CF%80.blk\nfile:///tmp/%FF-last.blk\n"
SCENARIOS = ("ordered", "legacy", "overlap", "leave", "no-mime", "no-copy", "decoration",
             "invalid", "oversize", "timeout", "seat-removed", "window-closed", "queue-limit")


class DropPeer(Peer):
    poll_interval = 0.005

    def __init__(self, connection, definitions, scenario):
        super().__init__(connection, definitions, scenario)
        self.globals += ["wl_seat", "wl_data_device_manager"]
        self.registry = None
        self.device = None
        self.scale_surface = None
        self.offers = {}
        self.open_fds = set()
        self.next_server_id = 0xff000000
        self.ready = False
        self.responsive = False
        self.closed = False
        self.resize_sent = False
        self.end_action_sent = False
        self.payloads_started = []
        self.payloads_finished = []

    def global_version(self, name):
        if name == "wl_data_device_manager" and self.scenario == "legacy":
            return 1
        return super().global_version(name)

    def consume_fd(self, interface, request, fd):
        if (interface, request) == ("wl_data_offer", "receive"):
            os.set_blocking(fd, False)
            self.open_fds.add(fd)
        else:
            super().consume_fd(interface, request, fd)

    def close(self):
        for fd in self.open_fds:
            os.close(fd)
        super().close()

    def close_window(self):
        if not self.closed:
            self.closed = True
            for toplevel in self.toplevels:
                self.emit(toplevel, "close")

    def offer(self, label, payload=b"", selection=False):
        object_id = self.next_server_id
        self.next_server_id += 1
        self.offers[object_id] = dict(label=label, payload=payload, sent=0, fd=None,
                                     eof=False, received=False, finished=False, destroyed=False,
                                     accepted=False, began=None)
        self.emit(self.device, "data_offer", object_id)
        self.emit(object_id, "offer", "text/plain" if self.scenario == "no-mime" and not selection else "text/uri-list")
        if selection:
            self.emit(self.device, "selection", object_id)
        else:
            surface = self.root
            if self.scenario == "decoration":
                surface = next(key for key, value in self.surfaces.items() if value["parent"] == self.root)
            self.emit(self.device, "enter", 100 + object_id % 100, surface, int(2.5 * 256), int(3.25 * 256), object_id)
            if self.objects[object_id][1] >= 3:
                self.emit(object_id, "source_actions", 3)
        return object_id

    def start(self):
        assert self.device is not None
        self.offer("clipboard", selection=True)
        payload = VALID
        if self.scenario == "invalid":
            payload = b"file:///tmp/good.blk\nfile://remote-host/tmp/bad.blk\n"
        elif self.scenario == "oversize":
            payload = b"#" + b"a" * (MAX_BYTES + 1024 * 1024)
        elif self.scenario == "overlap":
            payload = b"file:///tmp/first.blk\n"
        self.offer("first", payload)

    def after_negotiation(self, object_id):
        offer = self.offers[object_id]
        if offer["label"] == "clipboard":
            raise AssertionError("Clipboard selection must not be negotiated as a drop")
        if self.objects[object_id][1] >= 3:
            self.emit(object_id, "action", 2 if self.scenario == "no-copy" else 1)
        if self.scenario == "leave":
            self.emit(self.device, "leave")
            return
        second = offer["label"] == "second"
        x, y = (41.25, 55.5) if second else (24.5, 36.75)
        self.emit(self.device, "motion", 1000, int(x * 256), int(y * 256))
        self.emit(self.device, "drop")
        self.emit(self.device, "leave")

    def handle_request(self, object_id, call, v):
        super().handle_request(object_id, call, v)
        if call == "wl_display.get_registry":
            self.registry = v["registry"]
        elif call == "wl_registry.bind" and self.objects[v["id"]][0] == "wl_seat":
            self.emit(v["id"], "capabilities", 0)
            if self.objects[v["id"]][1] >= 2:
                self.emit(v["id"], "name", "private-probe-seat")
        elif call == "wl_data_device_manager.get_data_device":
            assert self.device is None
            self.device = v["id"]
        elif call == "wl_compositor.create_surface" and self.scale_surface is None:
            self.scale_surface = v["id"]
            self.emit(v["id"], "preferred_buffer_scale", 2)
        elif call == "xdg_toplevel.set_title":
            if v["title"] == "drop-probe-ready":
                assert not self.ready
                self.ready = True
                self.start()
            elif v["title"] == "drop-probe-responsive":
                assert self.resize_sent, "Responsiveness must acknowledge our mid-transfer configure"
                self.responsive = True
        elif call == "wl_data_offer.accept":
            offer = self.offers[object_id]
            supported = self.scenario not in ("no-mime", "decoration")
            assert v["mime_type"] == ("text/uri-list" if supported else None)
            offer["accepted"] = supported
            if self.objects[object_id][1] < 3:
                self.after_negotiation(object_id)
        elif call == "wl_data_offer.set_actions":
            supported = self.scenario not in ("no-mime", "decoration")
            assert v["dnd_actions"] == v["preferred_action"] == (1 if supported else 0)
            self.after_negotiation(object_id)
        elif call == "wl_data_offer.receive":
            offer = self.offers[object_id]
            assert offer["label"] != "clipboard" and offer["accepted"]
            assert not offer["received"], "Duplicate receive on a completed drag"
            assert self.scenario not in ("leave", "no-mime", "no-copy", "decoration")
            assert v["mime_type"] == "text/uri-list"
            offer.update(received=True, fd=v["fd"], began=time.monotonic())
            self.payloads_started.append(offer["label"])
            if self.scenario == "overlap" and offer["label"] == "first":
                self.offer("second", b"file:///tmp/second.blk\n")
            elif self.scenario == "queue-limit" and len(self.payloads_started) <= 16:
                self.offer(f"pending-{len(self.payloads_started) + 1}", VALID)
        elif call == "wl_data_offer.finish":
            offer = self.offers[object_id]
            assert offer["received"] and offer["eof"] and not offer["finished"]
            assert self.scenario in ("ordered", "overlap")
            offer["finished"] = True
            self.payloads_finished.append(offer["label"])
        elif call == "wl_data_offer.destroy":
            offer = self.offers[object_id]
            assert not offer["destroyed"], "Duplicate offer destruction"
            offer["destroyed"] = True
            if self.scenario in ("ordered", "legacy", "overlap") and offer["label"] != "clipboard":
                assert offer["eof"], "Accepted drop destroyed while its asynchronous read was pending"
                assert offer["finished"] == (self.scenario != "legacy")
            if self.scenario == "timeout" and offer["received"]:
                assert 9.0 <= time.monotonic() - offer["began"] < 18
            if self.scenario == "oversize" and offer["received"]:
                assert MAX_BYTES < offer["sent"] <= MAX_BYTES + 131072, "Byte bound did not stop the pending read promptly"
            if offer["fd"] is not None:
                try:
                    os.write(offer["fd"], b"x")
                except BrokenPipeError:
                    pass
                else:
                    raise AssertionError("Canceled offer was destroyed before its receive pipe was closed")
            if self.scenario == "queue-limit" and offer["label"] == "pending-17":
                assert not offer["received"] and len(self.payloads_started) == 16
                # Wait for a native resize acknowledgment before closing below.
            elif all(o["destroyed"] for o in self.offers.values()):
                self.close_window()
        elif call in ("wl_data_device.set_selection", "wl_data_device.start_drag"):
            raise AssertionError("Receive-only adapter changed clipboard/drag ownership")
        elif call == "wl_surface.destroy" and object_id == self.root:
            assert all(o["destroyed"] for o in self.offers.values()), "Window closed with outstanding drag resources"

    def tick(self):
        # The first write is deliberately incomplete. Until the Rust window
        # acknowledges a configure, keep every pipe open. Blocking reads would
        # deadlock here, so this checks event-loop progress rather than timing.
        for object_id, offer in list(self.offers.items()):
            if offer["fd"] is None or offer["destroyed"]:
                continue
            if offer["sent"] == 0:
                sent = os.write(offer["fd"], offer["payload"][:11])
                offer["sent"] += sent
                if not self.resize_sent:
                    self.resize_sent = True
                    self.configure(width=660)
                continue
            if not self.responsive:
                continue
            if self.scenario in ("timeout", "seat-removed", "window-closed", "queue-limit"):
                continue
            if self.scenario == "overlap" and offer["label"] == "first" and "second" not in self.payloads_finished:
                continue
            remaining = offer["payload"][offer["sent"]:offer["sent"] + 65536]
            if remaining:
                try:
                    offer["sent"] += os.write(offer["fd"], remaining)
                except BlockingIOError:
                    pass
                except BrokenPipeError:
                    assert self.scenario == "oversize"
            elif self.scenario != "oversize":
                os.close(offer["fd"])
                self.open_fds.remove(offer["fd"])
                offer.update(fd=None, eof=True)
        if self.responsive and not self.end_action_sent:
            if self.scenario == "seat-removed":
                self.end_action_sent = True
                self.emit(self.registry, "global_remove", self.globals.index("wl_seat") + 1)
            elif self.scenario == "window-closed" or (self.scenario == "queue-limit" and any(
                    o["label"] == "pending-17" and o["destroyed"] for o in self.offers.values())):
                self.end_action_sent = True
                self.close_window()

    def verify(self):
        super().verify()
        assert self.ready and self.closed
        assert self.offers and all(o["destroyed"] for o in self.offers.values())
        for offer in self.offers.values():
            if offer["fd"] is not None:
                try:
                    os.write(offer["fd"], b"x")
                except BrokenPipeError:
                    pass
                else:
                    raise AssertionError("Canceled transfer still owns its read descriptor")
        if self.scenario == "overlap":
            assert self.payloads_finished == ["second", "first"]
        if self.scenario in ("seat-removed", "window-closed", "queue-limit"):
            assert self.end_action_sent


def main():
    if sys.platform != "linux":
        raise SystemExit("This native protocol probe runs on Linux")
    binary, definitions = build_probe("wayland_file_drop")
    for scenario in SCENARIOS:
        run_scenario(binary, definitions, scenario, DropPeer, "native_wayland_file_drop",
                     "POLISHED_MAP_WAYLAND_DROP_PROBE", "file-drop")


if __name__ == "__main__":
    main()
