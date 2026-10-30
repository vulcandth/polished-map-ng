"""Exercise the real winit opacity adapter against an isolated Wayland protocol peer.

Linux only. Uses Python's Unix sockets and the locked crates' original protocol
XML; no compositor, GPU, desktop access or additional Python packages. This
verifies wire requests and resource lifetime, NOT rendered compositor pixels.
"""
import array
from collections import deque
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import sys
import tempfile
import time
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
OPAQUE = 0xffffffff
TRANSPARENT = 0xbfffffff


def word(value):
    return struct.pack("=I", value)


def blob(value):
    return word(len(value)) + value + b"\0" * (-len(value) % 4)


def schemas(artifacts):
    # Cargo's actual compiler artifacts identify the exact protocol sources used
    # by this native test. Whole-workspace metadata would unnecessarily fetch
    # every GUI dependency on a fresh machine, even with a platform-only build.
    def package(name):
        roots = {Path(a["manifest_path"]).parent for a in artifacts
                 if a.get("reason") == "compiler-artifact" and a["target"]["name"] == name.replace("-", "_")}
        assert len(roots) == 1, f"Expected one compiled {name} protocol source, got {roots}"
        return roots.pop()
    protocols = package("wayland-protocols") / "protocols"
    paths = [package("wayland-client") / "wayland.xml",
             protocols / "stable/xdg-shell/xdg-shell.xml",
             protocols / "unstable/xdg-decoration/xdg-decoration-unstable-v1.xml",
             protocols / "staging/alpha-modifier/alpha-modifier-v1.xml"]
    return {interface.attrib["name"]: interface for path in paths for interface in ET.parse(path).getroot().findall("interface")}


class Peer:
    poll_interval = 1

    def __init__(self, connection, definitions, scenario):
        self.connection = connection
        self.connection_closed = False
        self.definitions = definitions
        self.scenario = scenario
        self.objects = {1: ("wl_display", 1)}
        self.pending = bytearray()
        self.fds = deque()
        self.surfaces = {}
        self.subsurfaces = {}
        self.xdg = {}
        self.toplevels = {}
        self.decorations = {}
        self.alphas = {}
        self.alpha_created = 0
        self.alpha_destroyed = 0
        self.serial = 0
        self.configured = False
        self.root = None
        self.switched = False
        self.commits = []
        self.trace = deque(maxlen=35)
        self.globals = ["wl_compositor", "wl_subcompositor", "wl_shm", "xdg_wm_base", "zxdg_decoration_manager_v1"]
        if scenario != "missing-protocol":
            self.globals.append("wp_alpha_modifier_v1")

    def emit(self, object_id, name, *values):
        interface, _ = self.objects[object_id]
        events = self.definitions[interface].findall("event")
        opcode, event = next((i, e) for i, e in enumerate(events) if e.attrib["name"] == name)
        args = event.findall("arg")
        assert len(args) == len(values)
        body = b""
        for arg, value in zip(args, values):
            kind = arg.attrib["type"]
            if kind == "string":
                body += blob(value.encode("utf-8") + b"\0") if value is not None else word(0)
            elif kind == "array":
                body += blob(value)
            else:
                if kind == "new_id":
                    assert value not in self.objects
                    self.objects[value] = (arg.attrib["interface"], self.objects[object_id][1])
                assert kind in ("uint", "int", "object", "fixed", "new_id")
                body += word(value & OPAQUE)
        if not self.connection_closed:
            try:
                self.connection.sendall(word(object_id) + word(((len(body) + 8) << 16) | opcode) + body)
            except (BrokenPipeError, ConnectionResetError):
                # A client can close with cleanup replies still unread. Keep
                # parsing requests already received; exit status and complete
                # resource/state assertions below distinguish teardown from a
                # crash or premature disconnect.
                self.connection_closed = True

    def retire(self, object_id):
        if object_id < 0xff000000:  # Server-created IDs use their separate range.
            self.emit(1, "delete_id", object_id)
        del self.objects[object_id]

    def consume_fd(self, interface, request, fd):
        os.close(fd)  # Base peer only receives SHM; it does not inspect pixels.

    def tick(self):
        pass

    def global_version(self, name):
        return int(self.definitions[name].attrib["version"])

    def close(self):
        for fd in self.fds:
            os.close(fd)

    def decode(self, object_id, opcode, payload):
        interface, version = self.objects[object_id]
        request = self.definitions[interface].findall("request")[opcode]
        assert int(request.attrib.get("since", "1")) <= version
        offset = 0

        def number():
            nonlocal offset
            value, = struct.unpack_from("=I", payload, offset)
            offset += 4
            return value

        def data():
            nonlocal offset
            size = number()
            value = payload[offset:offset + size]
            assert len(value) == size
            offset += (size + 3) & ~3
            return value

        values = {}
        for arg in request.findall("arg"):
            kind = arg.attrib["type"]
            if kind == "fd":
                value = self.fds.popleft()
                self.consume_fd(interface, request.attrib["name"], value)
            elif kind in ("string", "array"):
                value = data()
                if kind == "string":
                    assert value.endswith(b"\0") or (not value and arg.attrib.get("allow-null") == "true")
                    value = value[:-1].decode("utf-8") if value else None
            elif kind == "new_id":
                new_interface = arg.attrib.get("interface")
                new_version = version
                if new_interface is None:  # wl_registry.bind's dynamic new_id
                    new_interface = data()[:-1].decode("utf-8")
                    new_version = number()
                value = number()
                assert value not in self.objects
                self.objects[value] = (new_interface, min(new_version, int(self.definitions[new_interface].attrib["version"])))
            else:
                assert kind in ("uint", "int", "object", "fixed")
                value = number()
            values[arg.attrib["name"]] = value
        assert offset == len(payload), (interface, request.attrib["name"], offset, len(payload))
        return interface, request, values

    def configure(self, server_frame=False, width=640, height=480):
        self.serial += 1
        for decoration in self.decorations:
            self.emit(decoration, "configure", 2 if server_frame else 1)
        for toplevel in self.toplevels:
            self.emit(toplevel, "configure", width, height, b"")
        for xdg in self.xdg:
            self.emit(xdg, "configure", self.serial)

    def surface_commit(self, surface):
        state = self.surfaces[surface]
        state["cached"] = state["pending"]
        if state["parent"] == self.root and self.root in self.surfaces:
            root = self.surfaces[self.root]
            if root["alpha"] is not None and state["pending"] != root["applied"]:
                assert state["sync"], "Frame opacity changed before the parent's commit"
        if not state["sync"]:
            state["applied"] = state["cached"]
        if surface != self.root:
            return
        for child in self.surfaces.values():
            if child["parent"] == surface and child["sync"]:
                child["applied"] = child["cached"]
        if not self.configured:
            self.configured = True
            self.configure(self.scenario == "server-frame")
        if state["alpha"] is not None:
            children = [s["applied"] for s in self.surfaces.values() if s["parent"] == surface]
            self.commits.append((state["applied"], children))
            if [factor for factor, _ in self.commits].count(TRANSPARENT) == 2 and not self.switched:
                self.switched = True
                self.configure(server_frame=True)

    def handle(self, object_id, opcode, payload):
        interface, request, v = self.decode(object_id, opcode, payload)
        name = request.attrib["name"]
        call = f"{interface}.{name}"
        self.trace.append((object_id, call, v))
        self.handle_request(object_id, call, v)
        if request.attrib.get("type") == "destructor":
            self.retire(object_id)

    def handle_request(self, object_id, call, v):
        if call == "wl_display.get_registry":
            for i, global_name in enumerate(self.globals, 1):
                self.emit(v["registry"], "global", i, global_name, self.global_version(global_name))
        elif call == "wl_display.sync":
            self.emit(v["callback"], "done", self.serial)
            self.retire(v["callback"])
        elif call == "wl_registry.bind":
            if self.objects[v["id"]][0] == "wl_shm":
                self.emit(v["id"], "format", 0)
                self.emit(v["id"], "format", 1)
        elif call == "wl_compositor.create_surface":
            self.surfaces[v["id"]] = dict(parent=None, sync=False, alpha=None,
                                           pending=OPAQUE, cached=OPAQUE, applied=OPAQUE)
        elif call == "xdg_wm_base.get_xdg_surface":
            self.xdg[v["id"]] = v["surface"]
            self.root = v["surface"]
        elif call == "xdg_surface.get_toplevel":
            self.toplevels[v["id"]] = object_id
        elif call == "zxdg_decoration_manager_v1.get_toplevel_decoration":
            self.decorations[v["id"]] = v["toplevel"]
        elif call == "wl_subcompositor.get_subsurface":
            self.subsurfaces[v["id"]] = v["surface"]
            self.surfaces[v["surface"]].update(parent=v["parent"], sync=True)
        elif call in ("wl_subsurface.set_sync", "wl_subsurface.set_desync"):
            state = self.surfaces[self.subsurfaces[object_id]]
            state["sync"] = call == "wl_subsurface.set_sync"
            if not state["sync"]:
                state["applied"] = state["cached"]
        elif call == "wp_alpha_modifier_v1.get_surface":
            state = self.surfaces[v["surface"]]
            assert state["alpha"] is None, "Duplicate alpha modifier on a live surface"
            state["alpha"] = v["id"]
            self.alphas[v["id"]] = v["surface"]
            self.alpha_created += 1
        elif call == "wp_alpha_modifier_surface_v1.set_multiplier":
            self.surfaces[self.alphas[object_id]]["pending"] = v["factor"]
        elif call == "wp_alpha_modifier_surface_v1.destroy":
            surface = self.alphas.pop(object_id)
            self.surfaces[surface]["alpha"] = None
            self.alpha_destroyed += 1
        elif call == "wl_surface.commit":
            self.surface_commit(object_id)
        elif call == "wl_surface.frame":
            # No renderer requests frames in this probe, but support callbacks.
            self.emit(v["callback"], "done", 0)
            self.retire(v["callback"])
        elif call == "wl_surface.destroy":
            assert self.surfaces[object_id]["alpha"] is None, "Alpha must die before wl_surface"
            del self.surfaces[object_id]
    def receive(self):
        if self.connection_closed:
            return False
        try:
            data, ancillary, flags, _ = self.connection.recvmsg(65536, socket.CMSG_SPACE(128))
        except ConnectionResetError:
            self.connection_closed = True
            return False
        assert not flags & (socket.MSG_TRUNC | socket.MSG_CTRUNC)
        for level, kind, raw in ancillary:
            assert (level, kind) == (socket.SOL_SOCKET, socket.SCM_RIGHTS)
            received = array.array("i")
            received.frombytes(raw[:len(raw) - len(raw) % received.itemsize])
            self.fds.extend(received)
        if not data:
            return False
        self.pending.extend(data)
        while len(self.pending) >= 8:
            object_id, header = struct.unpack_from("=II", self.pending)
            size, opcode = header >> 16, header & 0xffff
            assert size >= 8 and size % 4 == 0
            if len(self.pending) < size:
                break
            payload = bytes(self.pending[8:size])
            del self.pending[:size]
            self.handle(object_id, opcode, payload)
        return True

    def verify(self):
        assert not self.pending and not self.fds
        assert self.configured and not self.surfaces, "Window resources were not destroyed"
        assert self.alpha_created == self.alpha_destroyed
        if self.scenario == "client-frame":
            assert self.alpha_created == 6, "Root plus all five client-frame surfaces must have alpha"
            assert [factor for factor, _ in self.commits] == [TRANSPARENT, OPAQUE, TRANSPARENT, OPAQUE], self.commits
            for factor, children in self.commits[:3]:
                assert children == [factor] * 5, "Frame and content must commit the same multiplier"
            assert self.switched
        else:
            assert self.alpha_created == 0 and not self.commits, "Unsupported request partially changed native state"


def run_scenario(binary, definitions, scenario, peer_type=Peer, test_name="native_wayland_opacity",
                 env_name="POLISHED_MAP_WAYLAND_OPACITY_PROBE", label="opacity"):
    with tempfile.TemporaryDirectory(prefix="polished-map-wayland-") as directory:
        address = str(Path(directory) / "wayland-probe")
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as listener:
            listener.bind(address)
            listener.listen(1)
            listener.settimeout(20)
            env = dict(os.environ, WAYLAND_DISPLAY=address, XDG_RUNTIME_DIR=directory)
            env[env_name] = scenario
            # This inherited FD would take precedence over WAYLAND_DISPLAY and
            # could attach a test to the caller's compositor instead of our peer.
            env.pop("WAYLAND_SOCKET", None)
            with tempfile.TemporaryFile() as output:
                process = subprocess.Popen([str(binary), test_name, "--ignored", "--exact", "--nocapture"],
                                           cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
                peer = None
                try:
                    connection, _ = listener.accept()
                    with connection:
                        connection.settimeout(peer_type.poll_interval)
                        peer = peer_type(connection, definitions, scenario)
                        deadline = time.monotonic() + 25
                        while time.monotonic() < deadline:
                            try:
                                if not peer.receive():
                                    break
                            except socket.timeout:
                                if process.poll() is not None:
                                    break
                            peer.tick()
                        else:
                            raise TimeoutError("Wayland probe exceeded 25 seconds")
                    assert process.wait(timeout=5) == 0, "Native probe failed"
                    peer.verify()
                except BaseException:
                    process.kill()
                    process.wait()
                    output.seek(0)
                    print(output.read().decode("utf-8", errors="replace"), file=sys.stderr)
                    if peer:
                        print("Recent protocol requests:", *peer.trace, sep="\n", file=sys.stderr)
                    raise
                finally:
                    if peer:
                        peer.close()
        print(f"Passed Wayland {label} protocol: {scenario}")


def build_probe(name):
    result = subprocess.run(["cargo", "test", "-p", "polished-map-platform", "--locked", "--test", name,
                             "--no-run", "--message-format=json"], cwd=ROOT,
                            capture_output=True, text=True, encoding="utf-8")
    if result.returncode:
        raise SystemExit(result.stderr + result.stdout)
    artifacts = [json.loads(line) for line in result.stdout.splitlines()]
    binary = next(Path(a["executable"]) for a in artifacts
                  if a.get("reason") == "compiler-artifact" and a.get("executable") and a["target"]["name"] == name)
    return binary, schemas(artifacts)


def main():
    if sys.platform != "linux":
        raise SystemExit("This native protocol probe runs on Linux")
    binary, definitions = build_probe("wayland_opacity")
    for scenario in ("client-frame", "server-frame", "missing-protocol"):
        run_scenario(binary, definitions, scenario)


if __name__ == "__main__":
    main()
