"""Compile pinned plusplus tile addressing and block placement without FLTK.

Original method bodies are unchanged; the harness supplies widget values only.
Generated source is LGPL-3.0-only and stays in ignored scratch storage.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

REVISION = "27c1f8479107c7495ac0c1818900190a137da6e1"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cxx", default="g++")
    parser.add_argument("--git", default="git")
    args = parser.parse_args()

    def source(path):
        return subprocess.check_output(
            [args.git, "-C", str(args.checkout), "show", f"{REVISION}:{path}"],
            encoding="utf-8",
        )

    header = source("src/tile.h").replace('#include "colors.h"', "")
    tile = source("src/tile.cpp")
    tile = "\n".join(line for line in tile.splitlines() if not line.startswith("#include"))
    block = source("src/block-window.cpp")
    start = block.index("void Block_Window::change_chip_cb(")
    callback = block[start:block.index("\n}\n", start) + 2]
    prelude = r'''
// Copyright Rangi42/Polished Map contributors; harness Polished Map NG. LGPL-3.0-only.
#include <cstdint>
#include <fstream>
#include <string>
using uchar = unsigned char;
enum class Palette { GRAY = 0 };
struct Config { static bool order; static bool arrange_0_before_1() { return order; } };
bool Config::order = false;
'''
    widgets = r'''
#define FL_LEFT_MOUSE 1
#define FL_RIGHT_MOUSE 3
#define TILES_PER_ROW 16
#define METATILE_SIZE 4
#define MAX_NUM_TILES 512
struct Fl {
    static int modifiers;
    static int event_button() { return FL_LEFT_MOUSE; }
    static bool event_shift() { return modifiers & 1; }
    static bool event_ctrl() { return modifiers & 2; }
    static bool event_alt() { return modifiers & 4; }
};
int Fl::modifiers = 0;
struct Value { int v = 0; int value() const { return v; } };
struct Window { void redraw() {} };
struct Chip {
    int idx = 0, r = 0, c = 0;
    Palette pal = Palette::GRAY;
    bool xf = false, yf = false, pri = false;
    int index() const { return idx; } void index(int i) { idx = i; }
    int row() const { return r; } int col() const { return c; }
    Palette palette() const { return pal; } void palette(Palette p) { pal = p; }
    bool x_flip() const { return xf; } void x_flip(bool b) { xf = b; }
    bool y_flip() const { return yf; } void y_flip(bool b) { yf = b; }
    bool priority() const { return pri; } void priority(bool b) { pri = b; }
    void damage(int) {}
};
struct Block_Window {
    Chip selected, *_selected = &selected, chips[16];
    Value x, y, p, pal, *_x_flip = &x, *_y_flip = &y, *_priority = &p, *_palette = &pal;
    Window window, *_window = &window;
    Chip *chip(int x, int y) { return &chips[y * 4 + x]; }
    void select(Chip *) {}
    static void change_chip_cb(Chip *, Block_Window *);
};
'''
    driver = r'''
int main(int argc, char **argv) {
    if (argc != 3) return 2;
    std::ofstream out(argv[2], std::ios::binary);
    if (std::string(argv[1]) == "address") {
        for (int order = 0; order < 2; order++)
        for (int offset = 0; offset < 256; offset++)
        for (int attr = 0; attr < 256; attr++) {
            Config::order = order;
            Tile t; t.offset(offset); t.attribute(attr);
            out.put(t.index() & 255); out.put(t.index() >> 8); out.put(t.attribute());
        }
    } else if (std::string(argv[1]) == "brush") {
        for (int order = 0; order < 2; order++)
        for (int mods = 0; mods < 8; mods++)
        for (int tile = 0; tile < 512; tile++)
        for (int at = 0; at < 16; at++)
        for (int flips = 0; flips < 4; flips++) {
            Config::order = order; Fl::modifiers = mods;
            Block_Window bw; bw.selected.idx = tile;
            bw.x.v = flips & 1; bw.y.v = flips & 2;
            bw.p.v = tile & 8; bw.pal.v = tile & 7;
            for (int i = 0; i < 16; i++) { bw.chips[i].r = i / 4; bw.chips[i].c = i % 4; }
            Block_Window::change_chip_cb(&bw.chips[at], &bw);
            for (const Chip &c : bw.chips) {
                Tile t; t.copy(c); out.put(t.offset()); out.put(t.attribute());
            }
        }
    } else return 4;
    return out.good() ? 0 : 3;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(prelude + header + widgets + tile + callback + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION,
        "snippets": {name: hashlib.sha256(body.encode("utf-8")).hexdigest() for name, body in
                     [("src/tile.h", header), ("src/tile.cpp", tile), ("Block_Window::change_chip_cb", callback)]},
        "changes": "Only includes removed; original method bodies unchanged.",
        "scope": "Bank/attribute conversion and left-click placement; no FLTK event delivery or painting.",
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
