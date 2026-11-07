"""Route real winit keyboard events through each fallback-frame subsurface.

An independent peer focuses root -> all five frame parts -> root -> outside.
Each transition requires an F11 acknowledgment from the owning winit window.
No compositor, display server, graphics driver or desktop input is used.
"""
import array
import socket
import sys
import tempfile
from wayland_opacity_probe import Peer, build_probe, run_scenario, word

KEYMAP = b'''xkb_keymap {
 xkb_keycodes "probe" { minimum = 8; maximum = 255; <FK11> = 95; };
 xkb_types "probe" { type "ONE_LEVEL" { modifiers = None; level_name[Level1] = "Any"; }; };
 xkb_compatibility "probe" {};
 xkb_symbols "probe" { key <FK11> { type="ONE_LEVEL", [ F11 ] }; };
};\0'''


class FocusPeer(Peer):
    def __init__(self, connection, definitions, scenario):
        super().__init__(connection, definitions, "missing-protocol")
        self.globals.append("wl_seat")
        self.keyboard = None
        self.route = []
        self.index = 0
        self.finished = False

    def keymap(self):
        events = self.definitions["wl_keyboard"].findall("event")
        opcode = next(i for i, event in enumerate(events) if event.attrib["name"] == "keymap")
        body = word(1) + word(len(KEYMAP))
        with tempfile.TemporaryFile() as source:
            source.write(KEYMAP)
            source.flush()
            message = word(self.keyboard) + word(((len(body) + 8) << 16) | opcode) + body
            assert self.connection.sendmsg([message], [
                (socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array("i", [source.fileno()]))
            ]) == len(message)
        self.emit(self.keyboard, "repeat_info", 0, 0)

    def focus_next(self):
        self.serial += 1
        if self.index:
            self.emit(self.keyboard, "leave", self.serial, self.route[self.index - 1])
        if self.index == len(self.route):
            self.finished = True
            self.emit(next(iter(self.toplevels)), "close")
            return
        self.serial += 1
        self.emit(self.keyboard, "enter", self.serial, self.route[self.index], b"")
        self.emit(self.keyboard, "modifiers", self.serial, 0, 0, 0, 0)
        self.emit(self.keyboard, "key", self.serial + 1, 100, 87, 1)
        self.emit(self.keyboard, "key", self.serial + 2, 101, 87, 0)
        self.serial += 2
        self.index += 1

    def handle_request(self, object_id, call, values):
        super().handle_request(object_id, call, values)
        if call == "wl_registry.bind" and self.objects[values["id"]][0] == "wl_seat":
            self.emit(values["id"], "capabilities", 2)
            self.emit(values["id"], "name", "isolated-keyboard")
        elif call == "wl_seat.get_keyboard":
            assert self.keyboard is None
            self.keyboard = values["id"]
            self.keymap()
        elif call == "xdg_toplevel.set_title":
            if values["title"] == "ready":
                parts = [s for s, state in self.surfaces.items() if state["parent"] == self.root]
                assert len(parts) == 5, "test requires all real client-frame subsurfaces"
                assert self.keyboard is not None
                self.route = [self.root, *parts, self.root]
                self.focus_next()
            elif values["title"].startswith("key-"):
                assert values["title"] == f"key-{self.index}"
                self.focus_next()

    def verify(self):
        super().verify()
        assert self.finished and self.index == 7


def main():
    if sys.platform != "linux":
        raise SystemExit("This protocol probe runs on Linux")
    binary, definitions = build_probe("wayland_frame_focus")
    run_scenario(binary, definitions, "frame-focus", peer_type=FocusPeer,
                 test_name="native_wayland_frame_focus",
                 env_name="POLISHED_MAP_WAYLAND_FRAME_FOCUS_PROBE", label="frame keyboard focus")


if __name__ == "__main__":
    main()
