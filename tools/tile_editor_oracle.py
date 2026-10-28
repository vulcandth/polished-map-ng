"""Compile pinned, unmodified tile pixel algorithms with minimal UI stubs.

This verifies pixel transformations and RGB paste placement, not FLTK dispatch.
Generated source is LGPL-3.0-only; no game assets are used.
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
    functions = []
    provenance = []
    for path, names in [
        ("src/tileset-window.cpp", ["void Tileset_Window::flood_fill(", "void Tileset_Window::substitute_hue(", "void Tileset_Window::swap_hues(", "void Tileset_Window::paste_tile_graphics_cb("]),
        ("src/colors.cpp", ["uchar Color::desaturated(", "Hue Color::mono_hue("]),
    ]:
        source = subprocess.check_output([args.git, "-C", str(args.checkout), "show", f"{REVISION}:{path}"], text=True)
        for name in names:
            start = source.index(name)
            function = source[start:source.index("\n}\n", start) + 2]
            functions.append(function)
            provenance.append({"source": path, "function": name, "sha256": hashlib.sha256(function.encode()).hexdigest(), "changes": []})
    harness = r'''
// Copyright Rangi42/Polished Map contributors; harness: Polished Map NG. LGPL-3.0-only.
#include <algorithm>
#include <cstdint>
#include <fstream>
#include <queue>
#include <vector>
#define TILE_SIZE 8
#define TILES_PER_ROW 16
#define TILES_PER_COL 16
#define NUM_HUES 4
using uchar = unsigned char;
enum class Hue { WHITE, DARK, LIGHT, BLACK };
enum class Palette { GRAY, RED, GREEN, WATER, YELLOW, BROWN, ROOF, TEXT, MONOCHROME, UNDEFINED };
static Hue mono_hues[] = {Hue::BLACK, Hue::DARK, Hue::LIGHT, Hue::WHITE};
struct Color {
    static uchar desaturated(uchar, uchar, uchar);
    static Hue mono_hue(uchar);
    static const uchar *color(int, Palette, Hue) { static uchar rgb[3] = {}; return rgb; }
};
struct Config { static bool allow; static bool allow_256_tiles() { return allow; } };
bool Config::allow = false;
struct Dummy { void redraw() {} int palettes() { return 0; } };
struct Toolbar_Button {};
struct Fl_Image {
    int width, height;
    std::vector<char> rgb;
    const char *pointer;
    Fl_Image(int w, int h): width(w), height(h), rgb(w*h*3), pointer(nullptr) {
        for (int i=0;i<w*h*3;i++) rgb[i] = char((i*37 + i/11) & 255);
        pointer = rgb.data();
    }
    int w() const { return width; } int h() const { return height; } int d() const { return 3; }
    const char **data() { return &pointer; }
};
struct Fl {
    static Fl_Image *image;
    static constexpr int clipboard_image = 1;
    static bool clipboard_contains(int) { return true; }
    static void paste(Dummy &, int, int) {}
    static void *event_clipboard() { return image; }
};
Fl_Image *Fl::image = nullptr;
struct Pixel_Button {
    int r, c; Hue h;
    int row() const { return r; } int col() const { return c; }
    Hue hue() const { return h; } void hue(Hue value) { h = value; }
};
struct Deep_Tile_Button {
    uint8_t index; Hue pixels[64] = {}; Palette p = Palette::UNDEFINED;
    uint8_t id() const { return index; }
    Palette palette() const { return p; } void palette(Palette value) { p = value; }
    void pixel(int x, int y, Hue h, uchar, uchar, uchar) { pixels[y*8+x] = h; }
};
struct Tileset_Window {
    Pixel_Button cells[64]; Pixel_Button *_pixels[64];
    Deep_Tile_Button tiles[256]; Deep_Tile_Button *_deep_tile_buttons[256], *_selected;
    Dummy dummy, *_window=&dummy, *_tileset=&dummy;
    Tileset_Window() {
        for(int i=0;i<64;i++) { cells[i].r=i/8; cells[i].c=i%8; cells[i].h=Hue::WHITE; _pixels[i]=&cells[i]; }
        for(int i=0;i<256;i++) { tiles[i].index=i; _deep_tile_buttons[i]=&tiles[i]; }
        _selected=&tiles[0];
    }
    void select(Deep_Tile_Button *) {}
    void flood_fill(Pixel_Button *, Hue, Hue) const;
    void substitute_hue(Hue, Hue) const;
    void swap_hues(Hue, Hue) const;
    static void paste_tile_graphics_cb(Toolbar_Button *, Tileset_Window *);
};
'''
    driver = r'''
int main(int argc, char **argv) {
    if(argc != 3) return 2;
    std::ofstream edits(argv[1], std::ios::binary), images(argv[2], std::ios::binary);
    for(int seed=0;seed<40;seed++) for(int op=0;op<3;op++) for(int at=0;at<64;at++) for(int to=0;to<4;to++) {
        Tileset_Window tw;
        for(int i=0;i<64;i++) {
            int value = seed<4 ? seed : seed<8 ? ((i/8+i%8+seed)%4) : ((i*seed + i/8 + (i%8)*(i%8))%4);
            tw.cells[i].h=Hue(value);
        }
        Hue from=tw.cells[at].h;
        if(op==0) tw.flood_fill(&tw.cells[at],from,Hue(to));
        else if(op==1) tw.substitute_hue(from,Hue(to));
        else tw.swap_hues(from,Hue(to));
        for(const auto &p:tw.cells) edits.put(char(p.h));
    }
    int starts[] = {0,15,0x50,0x5f,0x60,0x7f,0x80,0xd0,0xdf,0xe0,0xff};
    int sizes[][2] = {{1,1},{9,9},{128,128},{200,120}};
    for(int allow=0;allow<2;allow++) for(int start:starts) for(auto &size:sizes) {
        Config::allow=allow;
        Tileset_Window tw; tw._selected=&tw.tiles[start];
        Fl::image=new Fl_Image(size[0],size[1]);
        Tileset_Window::paste_tile_graphics_cb(nullptr,&tw);
        for(auto &tile:tw.tiles) { for(Hue h:tile.pixels) images.put(char(h)); images.put(char(tile.p)); }
    }
    return edits.good() && images.good() ? 0 : 3;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness + "\n".join(functions) + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({"revision": REVISION, "functions": provenance, "scope": "pixel operations and RGB paste; UI and color rendering are stubbed"}, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
