"""Exercise standard Wayland clipboard transfers against an independent wire peer.

No X11, data-control, compositor, GPU or user's clipboard is used. Exact bytes,
MIME ordering, fragmentation, source replacement and bounded failures are checked.
Integrated application exchange remains a separate native acceptance gate.
"""
import array
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import time

from wayland_opacity_probe import Peer, ROOT, blob, word, schemas, run_scenario

PAYLOAD = bytes(i % 251 for i in range(700_013))
SCENARIOS = ("png", "bmp", "no-mime", "copy", "source-snapshot", "oversize", "timeout", "seat-removed", "cancel", "external-png", "external-empty")


class ClipboardPeer(Peer):
    poll_interval = 0.002

    def __init__(self, connection, definitions, scenario):
        super().__init__(connection, definitions, scenario)
        self.globals = ["wl_compositor", "wl_seat", "wl_data_device_manager"]
        self.registries = []
        self.surface = None
        self.devices = []
        self.keyboards = []
        self.offers = {}
        self.offer_devices = {}
        self.source_formats = {}
        self.sources = []
        self.reads = []
        self.writes = []
        self.last_fd = None
        self.next_id = 0xff000000
        self.received_mimes = []
        self.finished = False

    def consume_fd(self, interface, request, fd):
        assert (interface, request) == ("wl_data_offer", "receive")
        os.set_blocking(fd, False)
        self.last_fd = fd

    def offer(self, device, done=False):
        ident = self.next_id
        self.next_id += 1
        self.offers[ident] = done
        self.offer_devices[ident] = device
        self.emit(device, "data_offer", ident)
        formats = ["application/x-done"] if done else ["application/x-ready"]
        if not done and self.scenario != "no-mime":
            formats += ["image/bmp"]
            if self.scenario != "bmp":
                formats += ["image/png"]  # Advertise BMP first to test caller preference.
        for mime in formats:
            self.emit(ident, "offer", mime)
        self.emit(device, "selection", ident)

    def request_source(self, source, mime, expected):
        reader, writer = os.pipe()
        os.set_blocking(reader, False)
        events = self.definitions["wl_data_source"].findall("event")
        opcode = next(i for i, event in enumerate(events) if event.attrib["name"] == "send")
        body = blob(mime.encode("utf-8") + b"\0")
        self.connection.sendmsg([word(source) + word(((len(body) + 8) << 16) | opcode) + body],
                                [(socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array("i", [writer]))])
        os.close(writer)
        self.reads.append(dict(fd=reader, expected=expected, data=bytearray()))

    def handle_request(self, object_id, call, v):
        if call == "wl_display.get_registry":
            self.registries.append(v["registry"])
        if call == "wl_registry.bind" and self.objects[v["id"]][0] == "wl_seat":
            self.emit(v["id"], "capabilities", 2)
            self.emit(v["id"], "name", "clipboard-test-seat")
        elif call == "wl_compositor.create_surface":
            self.surface = v["id"]
        elif call == "wl_seat.get_keyboard":
            self.keyboards.append(v["id"])
            assert self.surface is not None
            self.emit(v["id"], "enter", 321, self.surface, b"")
        elif call == "wl_data_device_manager.get_data_device":
            self.devices.append(v["id"])
            self.offer(v["id"])
        elif call == "wl_data_offer.receive":
            fd, self.last_fd = self.last_fd, None
            assert fd is not None
            mime = v["mime_type"]
            self.received_mimes.append(mime)
            data = b"done" if self.offers[object_id] else b"ready"
            if mime.startswith("image/"):
                if self.scenario == "external-png":
                    assert len(self.devices) == 2 and self.offer_devices[object_id] == self.devices[-1], "Read did not use the host adapter offer"
                assert mime == ("image/bmp" if self.scenario == "bmp" else "image/png")
                data = PAYLOAD
                if self.scenario == "oversize":
                    data = b"x" * (64 * 1024 * 1024 + 1)
                elif self.scenario in ("timeout", "cancel", "seat-removed"):
                    data = None
                if self.scenario == "seat-removed":
                    for registry in self.registries:
                        self.emit(registry, "global_remove", 2)
            self.writes.append(dict(fd=fd, data=data, offset=0))
        elif call == "wl_data_source.offer":
            self.source_formats.setdefault(object_id, []).append(v["mime_type"])
        elif call == "wl_data_device.set_selection":
            assert v["serial"] == 321, "Copy must use the focused seat's serial"
            self.sources.append(v["source"])
            if len(self.sources) == (2 if self.scenario == "source-snapshot" else 1):
                original = self.sources[0]
                assert self.source_formats[original] == ["image/png", "image/bmp"]
                self.request_source(original, "image/png", PAYLOAD)
                self.request_source(original, "image/bmp", b"BMP payload")
                if len(self.sources) == 2:
                    self.request_source(self.sources[1], "image/png", b"new selection")
                # The source can be cancelled while an already-requested transfer
                # completes; its captured bytes must stay alive independently.
                self.emit(original, "cancelled")
        if call != "wl_surface.commit":
            super().handle_request(object_id, call, v)

    def tick(self):
        for transfer in list(self.writes):
            if transfer["data"] is None:
                continue
            try:
                n = os.write(transfer["fd"], transfer["data"][transfer["offset"]:transfer["offset"] + 65536])
                transfer["offset"] += n
                complete = transfer["offset"] == len(transfer["data"])
            except BlockingIOError:
                continue
            except BrokenPipeError:
                assert self.scenario == "oversize"
                complete = True
            if complete:
                os.close(transfer["fd"])
                self.writes.remove(transfer)
        for transfer in list(self.reads):
            try:
                data = os.read(transfer["fd"], 65536)
            except BlockingIOError:
                continue
            if data:
                transfer["data"].extend(data)
            else:
                assert transfer["data"] == transfer["expected"], "Copied bytes changed or truncated"
                os.close(transfer["fd"])
                self.reads.remove(transfer)
        if len(self.sources) == (2 if self.scenario == "source-snapshot" else 1) and not self.reads and not self.finished:
            self.finished = True
            for device in self.devices:
                self.offer(device, done=True)

    def verify(self):
        assert not self.pending and not self.fds and self.last_fd is None
        assert self.keyboards and self.devices and not self.surfaces
        if self.scenario in ("copy", "source-snapshot"):
            assert self.finished and not self.reads
        elif self.scenario in ("no-mime", "external-empty"):
            assert not any(m.startswith("image/") for m in self.received_mimes)
        else:
            assert any(m.startswith("image/") for m in self.received_mimes)
        for transfer in self.writes:
            assert self.scenario in ("cancel", "timeout", "seat-removed")
            try:
                os.write(transfer["fd"], b"x")
            except BrokenPipeError:
                pass
            else:
                raise AssertionError("Cancelled transfer retained its read descriptor")

    def close(self):
        for transfer in self.reads + self.writes:
            os.close(transfer["fd"])
        super().close()


def main():
    if sys.platform != "linux":
        raise SystemExit("This native clipboard probe runs on Linux")
    result = subprocess.run(["cargo", "test", "--manifest-path", "vendor/smithay-clipboard-0.7.3/Cargo.toml", "--locked", "--lib", "--no-run", "--message-format=json"],
                            cwd=ROOT, capture_output=True, text=True, encoding="utf-8")
    if result.returncode:
        raise SystemExit(result.stderr + result.stdout)
    artifacts = [json.loads(line) for line in result.stdout.splitlines()]
    binary = next(Path(a["executable"]) for a in artifacts if a.get("reason") == "compiler-artifact"
                  and a.get("executable") and a["target"]["name"] == "smithay_clipboard")
    subprocess.run([str(binary)], check=True, cwd=ROOT)
    definitions = schemas(artifacts)
    for scenario in SCENARIOS:
        run_scenario(binary, definitions, scenario, ClipboardPeer,
                     "state::binary::tests::native_wayland_binary_clipboard",
                     "POLISHED_MAP_WAYLAND_CLIPBOARD_PROBE", "clipboard")


if __name__ == "__main__":
    main()
