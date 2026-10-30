"""Verify native modal ownership on a private Wayland protocol peer.

Checks all parent identities, optional modal hints, nesting, recreation, and
teardown. No desktop, GPU or GUI automation; compositor placement is separate.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import xml.etree.ElementTree as ET
from wayland_opacity_probe import Peer, ROOT, run_scenario, schemas


class ModalPeer(Peer):
    def __init__(self, connection, definitions, scenario):
        super().__init__(connection, definitions, "missing-protocol")
        self.scenario = scenario
        if scenario == "dialog-protocol":
            self.globals.append("xdg_wm_dialog_v1")
        self.configured_surfaces = set()
        self.created = []
        self.parents = {}
        self.parent_changes = []
        self.dialogs = {}
        self.modals = []
        self.dialog_destroyed = 0

    def surface_commit(self, surface):
        xdg = next((x for x, s in self.xdg.items() if s == surface), None)
        if xdg is not None and surface not in self.configured_surfaces:
            self.configured_surfaces.add(surface)
            self.serial += 1
            top = next(t for t, x in self.toplevels.items() if x == xdg)
            for decoration, t in self.decorations.items():
                if t == top:
                    self.emit(decoration, "configure", 2)
            self.emit(top, "configure", 640, 480, b"")
            self.emit(xdg, "configure", self.serial)

    def handle_request(self, object_id, call, v):
        if call == "xdg_toplevel.set_parent":
            assert v["parent"] == 0 or v["parent"] in self.toplevels
            assert object_id != v["parent"]
            self.parents[object_id] = v["parent"]
            self.parent_changes.append((object_id, v["parent"]))
        elif call == "xdg_wm_dialog_v1.get_xdg_dialog":
            assert v["toplevel"] not in self.dialogs.values()
            assert self.parents[v["toplevel"]] != 0
            self.dialogs[v["id"]] = v["toplevel"]
        elif call == "xdg_dialog_v1.set_modal":
            self.modals.append(self.dialogs[object_id])
        elif call == "xdg_dialog_v1.destroy":
            self.dialogs.pop(object_id)
            self.dialog_destroyed += 1
        elif call == "xdg_toplevel.destroy":
            assert object_id not in self.dialogs.values(), "hint must die before its toplevel"
            assert not self.parents.get(object_id), "dialog must detach before destruction"
            self.toplevels.pop(object_id)
        elif call == "xdg_surface.destroy":
            self.xdg.pop(object_id)
        elif call == "zxdg_toplevel_decoration_v1.destroy":
            self.decorations.pop(object_id)
        else:
            super().handle_request(object_id, call, v)
            if call == "xdg_surface.get_toplevel":
                self.created.append(v["id"])

    def verify(self):
        assert len(self.created) == 4, self.created
        root, child, nested, replacement = self.created
        assert self.parent_changes == [(child, root), (nested, child), (nested, 0),
                                       (child, 0), (replacement, root), (replacement, 0)], self.parent_changes
        expected = [child, nested, replacement] if self.scenario == "dialog-protocol" else []
        assert self.modals == expected, self.modals
        assert self.dialog_destroyed == len(expected)
        assert not self.dialogs and not self.toplevels and not self.xdg and not self.surfaces
        assert not self.pending and not self.fds


def main():
    if sys.platform != "linux":
        raise SystemExit("This native probe runs on Linux")
    result = subprocess.run(["cargo", "build", "-p", "polished-map-platform", "--locked", "--example",
                             "native_modals", "--message-format=json"], cwd=ROOT,
                            capture_output=True, text=True, encoding="utf-8")
    if result.returncode:
        raise SystemExit(result.stderr + result.stdout)
    artifacts = [json.loads(line) for line in result.stdout.splitlines()]
    binary = next(Path(a["executable"]) for a in artifacts if a.get("reason") == "compiler-artifact"
                  and a.get("executable") and a["target"]["name"] == "native_modals")
    definitions = schemas(artifacts)
    protocol = next(Path(a["manifest_path"]).parent for a in artifacts
                    if a.get("reason") == "compiler-artifact" and a["target"]["name"] == "wayland_protocols")
    xml = protocol / "protocols/staging/xdg-dialog/xdg-dialog-v1.xml"
    definitions.update({i.attrib["name"]: i for i in ET.parse(xml).getroot().findall("interface")})
    os.environ["POLISHED_MAP_NATIVE_MODAL_PROBE"] = "1"
    for scenario in ("dialog-protocol", "missing-protocol"):
        run_scenario(binary, definitions, scenario, peer_type=ModalPeer,
                     env_name="POLISHED_MAP_MODAL_WAYLAND", label="modal ownership")


if __name__ == "__main__":
    main()
