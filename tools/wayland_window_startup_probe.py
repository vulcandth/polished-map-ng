"""Verify production window startup requests, without a desktop or GPU.

Fullscreen must take precedence over maximization throughout creation. A native
Weston 9 restart exposed an unmaximize request after fullscreen configuration;
that compositor then configured a zero-sized fullscreen surface and rejected it.
"""
import sys
from wayland_opacity_probe import Peer, build_probe, run_scenario, word


class StartupPeer(Peer):
    def __init__(self, connection, definitions, scenario):
        super().__init__(connection, definitions, "missing-protocol")
        self.startup = scenario
        self.requests = []
        self.created = 0

    def configure(self, server_frame=False, width=800, height=600):
        fullscreen = self.startup.startswith("fullscreen")
        state = 2 if fullscreen else 1 if self.startup == "maximized" else None
        self.serial += 1
        for decoration in self.decorations:
            self.emit(decoration, "configure", 2)
        for top in self.toplevels:
            self.emit(top, "configure", 1100 if state else width, 800 if state else height,
                      word(state) if state else b"")
        for xdg in self.xdg:
            self.emit(xdg, "configure", self.serial)

    def handle_request(self, object_id, call, values):
        if call in {"xdg_toplevel.set_fullscreen", "xdg_toplevel.unset_fullscreen",
                    "xdg_toplevel.set_maximized", "xdg_toplevel.unset_maximized"}:
            self.requests.append(call.removeprefix("xdg_toplevel."))
            if self.startup.startswith("fullscreen"):
                assert self.requests == ["set_fullscreen"], (
                    "startup must not override fullscreen with a maximization request", self.requests)
        if call == "xdg_surface.get_toplevel":
            self.created += 1
        super().handle_request(object_id, call, values)

    def verify(self):
        super().verify()
        assert self.created == 1
        expected = ("set_fullscreen" if self.startup.startswith("fullscreen") else
                    "set_maximized" if self.startup == "maximized" else "unset_maximized")
        assert self.requests and set(self.requests) == {expected}, self.requests


def main():
    if sys.platform != "linux":
        raise SystemExit("This protocol probe runs on Linux")
    binary, definitions = build_probe("wayland_window_startup")
    for scenario in ("fullscreen", "fullscreen-and-maximized", "maximized", "normal"):
        run_scenario(binary, definitions, scenario, peer_type=StartupPeer,
                     test_name="native_wayland_window_startup",
                     env_name="POLISHED_MAP_WAYLAND_STARTUP_PROBE", label="window startup")


if __name__ == "__main__":
    main()
