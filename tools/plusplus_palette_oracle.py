"""Compile pinned plusplus palette layouts/defaults/roof overlays/export.

Generated source is LGPL-3.0-only. Original functions are unchanged; the
incomplete export label table is corrected to avoid its null %s. A parser stub
supplies valid synthetic RGB5 colors; file parsing is tested separately in Rust.
No project/game assets or GUI are used.
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
    source = subprocess.check_output([
        args.git, "-C", str(args.checkout), "show", f"{REVISION}:src/colors.cpp"
    ]).decode("utf-8").replace("\r\n", "\n")
    start = source.index("static uchar tileset_colors[")
    defaults = source[start:source.index("\n};", start) + 3]
    signatures = [
        "Palettes Color::read_palettes(", "bool Color::read_roof_colors(",
        "bool Color::write_palettes(", "Hue Color::ordered_hue(",
        "const uchar *Color::color(", "Fl_Color Color::fl_color(",
        "void Color::color(Palettes l, Palette p, Hue h, ColorArray",
        "void Color::color(Palettes l, Palette p, Hue h, Fl_Color",
        "void Color::color(Palettes l, Palette p, HueArray",
    ]
    functions, provenance = [], []
    for signature in signatures:
        start = source.index(signature)
        function = source[start:source.index("\n}", start) + 2]
        functions.append(function)
        provenance.append({"signature": signature, "sha256": hashlib.sha256(function.encode()).hexdigest()})
    harness = r'''
#include <array>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>
using uchar = unsigned char;
using Fl_Color = uint32_t;
#define RGB5C(x) (uchar)((x)*33/4)
#define CRGB5(x) (uchar)((x)/8)
#define RGB5(r,g,b) {RGB5C(r),RGB5C(g),RGB5C(b)}
#define RGBX(rgb) {(((rgb)&0xFF0000)>>16),(((rgb)&0xFF00)>>8),((rgb)&0xFF)}
#define NUM_PALETTE_SETS 6
#define NUM_PALETTES 8
#define NUM_HUES 4
#define NUM_CHANNELS 3
enum class Hue { WHITE, DARK, LIGHT, BLACK };
enum class Palettes { MORN, DAY, NITE, DARKNESS, INDOOR, CUSTOM };
enum class Palette { GRAY, RED, GREEN, WATER, YELLOW, BROWN, ROOF, TEXT };
enum class Roof_Palettes { ROOF_CUSTOM, ROOF_DAY_NITE, ROOF_MORN_DAY_NITE, ROOF_DAY_NITE_CUSTOM, ROOF_MORN_DAY_NITE_CUSTOM };
static const Hue hue_order[4] = {Hue::WHITE,Hue::LIGHT,Hue::DARK,Hue::BLACK};
using ColorArray = std::array<uchar,3>;
using HueArray = std::array<ColorArray,4>;
using PalVec = std::vector<HueArray>;
PalVec input;
Fl_Color fl_rgb_color(uchar r, uchar g, uchar b) { return uint32_t(r)<<16|uint32_t(g)<<8|b; }
namespace Fl { void get_color(Fl_Color c, uchar &r, uchar &g, uchar &b) {r=c>>16;g=c>>8;b=c;} }
FILE *fl_fopen(const char *p,const char *m) {return fopen(p,m);}
struct Color {
    static Hue ordered_hue(int);
    static const uchar *color(Palettes,Palette,Hue);
    static void color(Palettes,Palette,Hue,ColorArray);
    static void color(Palettes,Palette,Hue,Fl_Color);
    static void color(Palettes,Palette,HueArray);
    static Fl_Color fl_color(Palettes,Palette,Hue);
    static PalVec parse_palettes(const char *) {return input;}
    static Palettes read_palettes(const char *,Palettes);
    static bool read_roof_colors(const char *,uint8_t,Roof_Palettes);
    static bool write_palettes(const char *);
};
static const char *palettes_names[6] = {"morn","day","nite","darkness","indoor","custom"};
static const char *palette_names[8] = {"gray","red","green","water","yellow","brown","roof","text"};
'''
    driver = r'''
void input_colors(int count) {
    input.assign(count,HueArray{});
    for(int p=0;p<count;p++) for(int h=0;h<4;h++) for(int c=0;c<3;c++)
        input[p][int(Color::ordered_hue(h))][c]=(p*7+h*5+c*11)%32;
}
int main(int argc,char **argv) {
    if(argc!=2) return 2;
    const std::string prefix=argv[1];
    uchar baseline[6][8][4][3]; memcpy(baseline,tileset_colors,sizeof(baseline));
    std::ofstream out(prefix+"-layouts.bin",std::ios::binary);
    for(int count=0;count<=64;count++) for(int selected=0;selected<6;selected++) {
        memcpy(tileset_colors,baseline,sizeof(baseline)); input_colors(count);
        out.put(int(Color::read_palettes("synthetic",Palettes(selected))));
        out.write(reinterpret_cast<const char *>(tileset_colors),sizeof(tileset_colors));
    }
    std::ofstream roofs(prefix+"-roofs.bin",std::ios::binary);
    for(int count: {0,1,2,3,4,512}) for(int layout=0;layout<5;layout++) for(int group: {0,1,2,255}) {
        memcpy(tileset_colors,baseline,sizeof(baseline)); input_colors(count);
        roofs.put(Color::read_roof_colors("synthetic",group,Roof_Palettes(layout))?1:0);
        roofs.write(reinterpret_cast<const char *>(tileset_colors),sizeof(tileset_colors));
    }
    for(int seed=0;seed<16;seed++) {
        for(int l=0;l<6;l++) for(int p=0;p<8;p++) for(int h=0;h<4;h++) for(int c=0;c<3;c++)
            tileset_colors[l][p][h][c]=(l*71+p*37+h*19+c*13+seed*31)&255;
        if(!Color::write_palettes((prefix+"-"+std::to_string(seed)+".pal").c_str())) return 3;
    }
    return out.good() && roofs.good()?0:4;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness + defaults + "\n".join(functions) + driver, encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "revision": REVISION, "source": "src/colors.cpp", "functions": provenance,
        "defaults_sha256": hashlib.sha256(defaults.encode()).hexdigest(),
        "changes_to_functions": [], "data_changes": ["Correct six export set labels"],
        "scope": "synthetic valid RGB5 input; no file parsing or GUI behavior",
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
