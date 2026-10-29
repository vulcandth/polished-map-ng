"""Compile original palette clipboard, roof-color import and export functions.

The export function is unmodified. Its incomplete set-name table is replaced by
six correct labels to avoid upstream's null %s; provenance records that change.
This does not verify FLTK input dispatch or native color chooser behavior.
Generated source is LGPL-3.0-only. No game assets are used.
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
    functions, provenance = [], []
    for path, names in [
        ("src/palette-window.cpp", ["void Abstract_Palette_Window::copy_color_cb(", "void Abstract_Palette_Window::paste_color_cb(", "void Abstract_Palette_Window::swap_colors_cb("]),
        ("src/colors.cpp", ["bool Color::write_palettes(", "Hue Color::ordered_hue(", "bool Color::read_roof_colors(",
                            "void Color::color(Palettes l, Palette p, Hue h, ColorArray",
                            "void Color::color(Palettes l, Palette p, Hue h, Fl_Color",
                            "Fl_Color Color::fl_color("]),
    ]:
        source = subprocess.check_output([args.git, "-C", str(args.checkout), "show", f"{REVISION}:{path}"], text=True, encoding="utf-8")
        for name in names:
            start = source.index(name)
            function = source[start:source.index("\n}\n", start) + 2]
            functions.append(function)
            provenance.append({"source": path, "function": name, "sha256": hashlib.sha256(function.encode()).hexdigest(), "changes": []})
    harness = r'''
// Copyright Rangi42/Polished Map contributors; harness: Polished Map NG. LGPL-3.0-only.
#include <cstdio>
#include <cstdint>
#include <fstream>
#include <string>
#include <array>
#include <vector>
using uchar = unsigned char;
using Fl_Color = uint32_t;
#define RGB5C(x) (uchar)((x) * 33 / 4)
#define CRGB5(c) (uchar)((c) / 8)
#define NUM_PALETTE_SETS 6
#define NUM_GAME_PALETTES 8
#define NUM_HUES 4
#define NUM_CHANNELS 3
enum class Hue { WHITE, DARK, LIGHT, BLACK };
static const Hue hue_order[4] = {Hue::WHITE, Hue::LIGHT, Hue::DARK, Hue::BLACK};
enum class Palettes { MORN, DAY, NITE, DARKNESS, INDOOR, CUSTOM };
enum class Palette { GRAY, RED, GREEN, WATER, YELLOW, BROWN, ROOF, TEXT, MONOCHROME };
enum class Roof_Palettes { ROOF_CUSTOM, ROOF_DAY_NITE, ROOF_MORN_DAY_NITE, ROOF_DAY_NITE_CUSTOM, ROOF_MORN_DAY_NITE_CUSTOM };
using ColorArray = std::array<uchar,3>;
using HueArray = std::array<ColorArray,4>;
using PalVec = std::vector<HueArray>;
PalVec roof_table;
Fl_Color fl_rgb_color(uchar r, uchar g, uchar b) { return uint32_t(r)<<16 | uint32_t(g)<<8 | b; }
namespace Fl { void get_color(Fl_Color c, uchar &r, uchar &g, uchar &b) {r=c>>16; g=c>>8; b=c;} }
struct Fl_Widget {};
struct Button { Fl_Color c=0; Fl_Color color() {return c;} void color(Fl_Color v) {c=v;} };
struct Window { void redraw() {} };
struct Abstract_Palette_Window {
    Button *_selected=nullptr, *_chosen=nullptr; Window window; Window *_window=&window;
    uchar _clipboard[3]={}; bool _copied=false;
    void select(Button *) {}
    static void copy_color_cb(Fl_Widget *, Abstract_Palette_Window *);
    static void paste_color_cb(Fl_Widget *, Abstract_Palette_Window *);
    static void swap_colors_cb(Fl_Widget *, Abstract_Palette_Window *);
};
namespace Config { bool mono=false; bool monochrome() {return mono;} }
FILE *fl_fopen(const char *path, const char *mode) {return fopen(path,mode);}
uchar values[6][9][4][3];
struct Color {
    static Hue ordered_hue(int);
    static const uchar *color(Palettes l, Palette p, Hue h) { return values[int(l)][int(p)][int(h)]; }
    static bool write_palettes(const char *);
    static PalVec parse_palettes(const char *) { return roof_table; }
    static uchar *colors(Palettes l, Palette p, Hue h) { return values[int(l)][int(p)][int(h)]; }
    static void color(Palettes, Palette, Hue, ColorArray);
    static void color(Palettes, Palette, Hue, Fl_Color);
    static Fl_Color fl_color(Palettes, Palette, Hue);
    static bool read_roof_colors(const char *,uint8_t,Roof_Palettes);
};
// Explicit safety correction to DATA, not to the extracted export function.
static const char *palettes_names[6] = {"morn", "day", "nite", "darkness", "indoor", "custom"};
static const char *palette_names[8] = {"gray", "red", "green", "water", "yellow", "brown", "roof", "text"};
'''
    driver = r'''
void emit(std::ofstream &out, Fl_Color color) {
    uchar r,g,b; Fl::get_color(color,r,g,b); out.put(r);out.put(g);out.put(b);
}
int main(int argc, char **argv) {
    if(argc!=3) return 2;
    std::ofstream clipboard(argv[1], std::ios::binary);
    for(int i=0;i<256;i++) {
        Abstract_Palette_Window w; Button a,b;
        a.color(fl_rgb_color(i,i*37,i*91)); w._selected=&a;
        Abstract_Palette_Window::copy_color_cb(nullptr,&w);
        a.color(fl_rgb_color(i^0x55,i^0xaa,i^0xcc)); w._selected=&b;
        Abstract_Palette_Window::paste_color_cb(nullptr,&w); emit(clipboard,b.color());
        Abstract_Palette_Window::swap_colors_cb(nullptr,&w); emit(clipboard,a.color());emit(clipboard,b.color());
    }
    for(int seed=0;seed<16;seed++) {
        for(int l=0;l<6;l++) for(int p=0;p<9;p++) for(int h=0;h<4;h++) for(int c=0;c<3;c++)
            values[l][p][h][c]=(l*71+p*37+h*19+c*13+seed*31)&255;
        for(bool mono: {false,true}) {
            Config::mono=mono;
            std::string path=std::string(argv[2])+"-"+std::to_string(seed)+(mono?"-mono.pal":"-color.pal");
            if(!Color::write_palettes(path.c_str())) return 3;
        }
    }
    // The parser stub supplies complete four-color palettes in internal hue
    // order. This checks the unmodified roof selection/overlay function, not
    // FLTK or the upstream ASM parser.
    std::ofstream roofs(std::string(argv[2])+"-roofs.bin", std::ios::binary);
    for(int count: {0,1,2,3,4,512}) {
        roof_table.assign(count,HueArray{});
        for(int i=0;i<count*4;i++) for(int c=0;c<3;c++)
            roof_table[i/4][int(Color::ordered_hue(i%4))][c]=(i*7+c*11)%32;
        for(int layout=0;layout<5;layout++) for(int group: {0,1,2,255}) {
            for(int l=0;l<6;l++) for(int p=0;p<9;p++) for(int h=0;h<4;h++) for(int c=0;c<3;c++)
                values[l][p][h][c]=(l*71+p*37+h*19+c*13)&255;
            roofs.put(Color::read_roof_colors("synthetic",group,Roof_Palettes(layout))?1:0);
            roofs.write(reinterpret_cast<const char *>(values),sizeof(values));
        }
    }
    return clipboard.good() && roofs.good()?0:4;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness + "\n".join(functions) + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION, "functions": provenance,
        "data_changes": ["Replace incomplete palettes_names table with morn/day/nite/darkness/indoor/custom to avoid null %s"],
        "stubs": ["Roof parse_palettes supplies complete synthetic RGB5 palettes; upstream parser/file I/O is not exercised"]
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
