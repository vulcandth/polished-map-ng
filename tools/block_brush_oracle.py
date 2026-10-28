"""Compile the unmodified master block placement callback with a minimal UI harness.

The harness supplies clicks, tile IDs and cleared chips; this tests placement math,
not FLTK dispatch or visual behavior. Generated code is LGPL-3.0-only.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

REVISION = "8b9bdc2b8aac8c865809516c2123af69c498da0b"

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--checkout", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--cxx", default="g++")
    p.add_argument("--git", default="git")
    args = p.parse_args()
    source = subprocess.check_output([args.git, "-C", str(args.checkout), "show", f"{REVISION}:src/block-window.cpp"], text=True)
    start = source.index("void Block_Window::change_chip_cb(")
    end = source.index("\n}\n", start) + 2
    function = source[start:end]
    harness = r'''
// Copyright Rangi42/Polished Map contributors; harness: Polished Map NG. LGPL-3.0-only.
#include <cstdint>
#include <fstream>
#define FL_LEFT_MOUSE 1
#define FL_RIGHT_MOUSE 3
#define TILES_PER_ROW 16
#define METATILE_SIZE 4
struct Fl {
    static int modifiers;
    static int event_button() { return FL_LEFT_MOUSE; }
    static bool event_shift() { return modifiers & 1; }
    static bool event_ctrl() { return modifiers & 2; }
    static bool event_alt() { return modifiers & 4; }
};
int Fl::modifiers = 0;
struct Config { static bool allow; static bool allow_256_tiles() { return allow; } };
bool Config::allow = false;
struct Tile_Button {
    uint8_t value = 0;
    uint8_t id() const { return value; }
    void setonly() {}
    void do_callback() {}
};
struct Chip {
    uint8_t value = 7, r = 0, c = 0;
    uint8_t id() const { return value; }
    void id(uint8_t v) { value = v; }
    uint8_t row() const { return r; }
    uint8_t col() const { return c; }
    void damage(int) {}
};
struct Block_Window {
    Tile_Button selected, *_selected = &selected, *_tile_buttons[256] = {};
    Chip chips[16];
    Chip *chip(int x, int y) { return &chips[y * 4 + x]; }
    static void change_chip_cb(Chip *, Block_Window *);
};
'''
    driver = r'''
int main(int argc, char **argv) {
    if (argc != 2) return 2;
    std::ofstream out(argv[1], std::ios::binary);
    for (int allow = 0; allow < 2; allow++)
    for (int modifiers = 0; modifiers < 8; modifiers++)
    for (int tile = 0; tile < 256; tile++)
    for (int at = 0; at < 16; at++) {
        Config::allow = allow;
        Fl::modifiers = modifiers;
        Block_Window bw;
        bw.selected.value = tile;
        for (int i = 0; i < 16; i++) { bw.chips[i].r = i / 4; bw.chips[i].c = i % 4; }
        Block_Window::change_chip_cb(&bw.chips[at], &bw);
        for (const Chip &c : bw.chips) out.put(c.id());
    }
    return out.good() ? 0 : 3;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness + function + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION, "source": "src/block-window.cpp",
        "function": "Block_Window::change_chip_cb",
        "function_sha256": hashlib.sha256(function.encode()).hexdigest(),
        "changes_to_function": [], "harness_scope": "left-click placement math only",
    }, indent=2) + "\n", encoding="utf-8")

if __name__ == "__main__":
    main()
