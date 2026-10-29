"""Compile unchanged master Print functions with a deterministic raster driver.

This verifies composition, not native FLTK text or display-driver rasterization.
Font masks are deliberately synthetic, including overhang and partial coverage.
Original event/priority/warp PNGs are extracted, decoded by the Rust test, and
passed back as raw RGBA. No game assets or native fonts are used/distributed.
Generated code/assets remain ignored; original portions are LGPL-3.0-only.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

REVISION = "8b9bdc2b8aac8c865809516c2123af69c498da0b"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cxx", default="g++")
    parser.add_argument("--git", default="git")
    args = parser.parse_args()
    sources = {}
    for path in ["utils.cpp", "map-buttons.cpp", "event.cpp", "main-window.cpp", "map-events.cpp", "tile.cpp"]:
        sources[path] = subprocess.check_output(
            [args.git, "-C", str(args.checkout), "show", f"{REVISION}:src/{path}"],
            text=True, encoding="utf-8")
    functions, provenance = [], []
    for path, name in [
        ("utils.cpp", "void draw_outlined_text("),
        ("map-buttons.cpp", "void Block::print("),
        ("event.cpp", "void Event::draw_warp_id("),
        ("event.cpp", "void Event::print("),
        ("main-window.cpp", "void Main_Window::print_map("),
    ]:
        source = sources[path]
        start = source.index(name)
        function = source[start:source.index("\n}\n", start)+2]
        functions.append(function)
        provenance.append({"source": "src/"+path, "function": name,
                           "sha256": hashlib.sha256(function.encode()).hexdigest(), "changes": []})
    source = sources["map-events.cpp"]
    start = source.index("static const std::unordered_map")
    metadata = source[start:source.index("\n};", start)+3]
    provenance.append({"source": "src/map-events.cpp", "table": "events_meta",
                       "sha256": hashlib.sha256(metadata.encode()).hexdigest(), "changes": []})
    args.output.parent.mkdir(parents=True, exist_ok=True)
    assets = args.output.with_suffix(".assets")
    assets.mkdir(exist_ok=True)
    for path, name in [("event.cpp", f"texture_{color}16") for color in
                       ["red", "green", "blue", "purple", "orange", "azure"]] + [
                           ("event.cpp", "warp_digits"), ("tile.cpp", "small_priority")]:
        body = re.search(name + r"_png_buffer\[\] = \{(.*?)\};", sources[path], re.S).group(1)
        png = bytes(int(n, 16) for n in re.findall(r"0x([0-9a-f]{2})", body))
        (assets / f"{name}.png").write_bytes(png)
        provenance.append({"source": "src/"+path, "asset": name,
                           "sha256": hashlib.sha256(png).hexdigest(), "changes": []})
    harness = Path(__file__).with_name("print_oracle_driver.cpp").read_text(encoding="utf-8")
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness.replace("// INSERT_ORIGINAL", metadata + "\n" + "\n".join(functions)), encoding="utf-8")
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION, "entries": provenance,
        "scope": "unchanged composition functions; synthetic fonts and software raster driver",
    }, indent=2) + "\n", encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)


if __name__ == "__main__":
    main()
