"""Build an oracle using unmodified bitplane conversion and palette defaults.

Usage: python tools/reference_oracle.py --checkout .reference/master
       --output .scratch/reference-2bpp --cxx g++
The generated source is a derivative of Polished Map (LGPL-3.0-only).
Only valid inputs are compared; this is not a full application parity test.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess

REVISION = "8b9bdc2b8aac8c865809516c2123af69c498da0b"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cxx", default="g++")
    parser.add_argument("--git", default="git")
    args = parser.parse_args()
    source = subprocess.check_output([
        args.git, "-C", str(args.checkout), "show", f"{REVISION}:src/tiled-image.cpp"
    ], text=True)
    start = source.index("static void convert_2bytes_to_8hues(")
    end = source.index("\n}\n", start) + 2
    function = source[start:end]
    colors = subprocess.check_output([
        args.git, "-C", str(args.checkout), "show", f"{REVISION}:src/colors.cpp"
    ], text=True)
    colors_header = subprocess.check_output([
        args.git, "-C", str(args.checkout), "show", f"{REVISION}:src/colors.h"
    ], text=True)
    start = colors.index("static uchar tileset_colors[")
    end = colors.index("\n};", start) + 3
    palette_table = colors[start:end]
    names = ("RGB5C", "RGB5", "NUM_PALETTE_SETS", "NUM_GAME_PALETTES", "NUM_HUES", "NUM_CHANNELS")
    macros = []
    for name in names:
        prefix = f"#define {name}"
        lines = [line for line in (colors_header + colors).splitlines()
                 if line.startswith(prefix + " ") or line.startswith(prefix + "(")]
        assert len(lines) == 1, name
        macros.extend(lines)
    header = """// Original function: Copyright Rangi42/Polished Map contributors.
// Driver: Copyright 2026 Polished Map NG contributors. LGPL-3.0-only.
#include <cstdint>
#include <fstream>
using uchar = unsigned char;
enum class Hue { WHITE, DARK, LIGHT, BLACK };
"""
    driver = """
int main(int argc, char **argv) {
    if (argc != 2 && argc != 3) return 2;
    std::ofstream output(argv[1], std::ios::binary);
    if (!output) return 3;
    for (unsigned a=0; a<256; ++a) {
        for (unsigned b=0; b<256; ++b) {
            Hue hues[8];
            convert_2bytes_to_8hues((uchar)a, (uchar)b, hues);
            for (Hue hue : hues) output.put((char)hue);
        }
    }
    if (!output.good()) return 4;
    if (argc == 3) {
        std::ofstream palettes(argv[2], std::ios::binary);
        palettes.write(reinterpret_cast<const char *>(tileset_colors), sizeof(tileset_colors));
        if (!palettes.good()) return 5;
    }
    return 0;
}
"""
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(header + "\n".join(macros) + "\n" + palette_table + "\n" + function + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION,
        "source": "src/tiled-image.cpp",
        "function": "convert_2bytes_to_8hues",
        "function_sha256": hashlib.sha256(function.encode()).hexdigest(),
        "changes_to_function": [],
        "palette_source": "src/colors.cpp",
        "palette_table_sha256": hashlib.sha256(palette_table.encode()).hexdigest(),
        "changes_to_palette_table": [],
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
