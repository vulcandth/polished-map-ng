"""Verify every original ICO frame is embedded unchanged in a Windows launcher.

Reads PE resource directories, not shell icon caches. Uses only the standard
library, so packaging can verify its executable offline on any build host.
"""
import argparse
from pathlib import Path
import struct

ICON = Path(__file__).resolve().parents[1] / "crates/gui/assets/artwork/app.ico"


def verify(executable, icon=ICON):
    data = Path(executable).read_bytes()
    source = Path(icon).read_bytes()
    pe, = struct.unpack_from("<I", data, 0x3C)
    if data[:2] != b"MZ" or data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("Not a PE executable")
    sections, = struct.unpack_from("<H", data, pe + 6)
    optional_size, = struct.unpack_from("<H", data, pe + 20)
    optional = pe + 24
    magic, = struct.unpack_from("<H", data, optional)
    if magic not in (0x10B, 0x20B):
        raise ValueError("Unknown PE optional header")
    directories = optional + (112 if magic == 0x20B else 96)
    resource_rva, resource_size = struct.unpack_from("<II", data, directories + 16)
    if not resource_rva or not resource_size:
        raise ValueError("Executable has no resources")
    table = optional + optional_size

    def file_offset(rva, size):
        for i in range(sections):
            _, address, raw_size, raw = struct.unpack_from("<IIII", data, table + 40 * i + 8)
            if address <= rva and rva + size <= address + raw_size:
                offset = raw + rva - address
                if offset + size <= len(data):
                    return offset
        raise ValueError("Resource points outside file-backed PE sections")

    base = file_offset(resource_rva, resource_size)

    def entries(relative):
        if not 0 <= relative <= resource_size - 16:
            raise ValueError("Invalid resource directory")
        named, numbered = struct.unpack_from("<HH", data, base + relative + 12)
        if relative + 16 + 8 * (named + numbered) > resource_size:
            raise ValueError("Truncated resource directory")
        return dict(struct.unpack_from("<II", data, base + relative + 16 + i * 8)
                    for i in range(named + numbered))

    def payload(kind, identifier):
        node = 0
        for key in [kind, identifier]:
            child = entries(node)[key]
            if not child & 0x80000000:
                raise ValueError("Expected a resource directory")
            node = child & 0x7FFFFFFF
        leaves = entries(node)
        if len(leaves) != 1:
            raise ValueError("Expected one language for original icon")
        leaf, = leaves.values()
        if leaf & 0x80000000 or leaf + 16 > resource_size:
            raise ValueError("Invalid resource leaf")
        rva, size = struct.unpack_from("<II", data, base + leaf)
        offset = file_offset(rva, size)
        return data[offset:offset + size]

    group = payload(14, 1)  # RT_GROUP_ICON, application resource ID
    if group[:6] != source[:6]:
        raise ValueError("Icon group header/count differs from upstream")
    count, = struct.unpack_from("<H", source, 4)
    sizes = set()
    for i in range(count):
        entry = source[6 + i * 16:22 + i * 16]
        embedded = group[6 + i * 14:20 + i * 14]
        if entry[:12] != embedded[:12]:
            raise ValueError(f"Icon frame {i} metadata differs from upstream")
        length, offset = struct.unpack_from("<II", entry, 8)
        identifier, = struct.unpack_from("<H", embedded, 12)
        if payload(3, identifier) != source[offset:offset + length]:
            raise ValueError(f"Icon frame {i} pixels differ from upstream")
        sizes.add(entry[0] or 256)
    if sizes != {16, 24, 32, 48, 64}:
        raise ValueError(f"Unexpected original icon sizes: {sizes}")
    return count


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executables", nargs="+", type=Path)
    for executable in parser.parse_args().executables:
        print(f"Verified {verify(executable)} unchanged ICO frames: {executable}")
