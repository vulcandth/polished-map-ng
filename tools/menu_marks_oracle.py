"""Compile unchanged pinned FLTK/master drawing functions into a command oracle.

Generated source remains in ignored scratch storage. This records geometry and
colors, not native rasterization. FLTK copyright/terms: LICENSES/FLTK.txt;
Polished Map themes: Rangi/contributors, LGPL-3.0-only.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import urllib.request

MASTER = "8b9bdc2b8aac8c865809516c2123af69c498da0b"
FLTK = "a9b1113516ffd15fc7602a6d425a317df30f4720"
FILES = {
    "fl_draw.cxx": "acd8d21aab871c8484ba10a2010fad86186200d882bc1e031dcafcdfcceafd40",
    "fl_contrast.cxx": "95f8ba993735c7ee202ee4f84afdb2bd91e966c920e8b8c9b7a721edf5883b8f",
    "fl_color.cxx": "683c75655dd6a19f2335216c5f99dad2abc207f9be3119867bba38aab5655150",
    "Fl_Menu.cxx": "5c7b2863624c33959c0bc2dce014f4692f1a29692ed6f0bff802fd0823c5aa36",
    "Fl_get_system_colors.cxx": "91618fb5ef1d64a2b8d4a5cdbe1e5d93ff028cefa7c74e5749c7d545d1a305e5",
    "Enumerations.H": "a27dae9f64b46570b154a318f6eb63d8669de8f105a1500a6a6564c4cab4d2a3",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--fltk-cache", type=Path, required=True)
    parser.add_argument("--cxx", default="g++")
    parser.add_argument("--git", default="git")
    args = parser.parse_args()
    args.fltk_cache.mkdir(parents=True, exist_ok=True)
    sources = {}
    for name, expected in FILES.items():
        path = args.fltk_cache / name
        if not path.exists():
            folder = "FL" if name == "Enumerations.H" else "src"
            with urllib.request.urlopen(f"https://raw.githubusercontent.com/fltk/fltk/{FLTK}/{folder}/{name}", timeout=30) as response:
                path.write_bytes(response.read())
        raw = path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != expected:
            raise ValueError(f"Unexpected pinned FLTK hash: {name}")
        sources[name] = raw.decode("utf-8")
    def original(path):
        return subprocess.check_output([args.git, "-C", str(args.checkout), "show", f"{MASTER}:{path}"], encoding="utf-8")
    sources["themes.cpp"] = original("src/themes.cpp")
    menu = sources["Fl_Menu.cxx"]
    functions, evidence = [], []
    constants = []
    for name in ["FL_BLACK", "FL_WHITE", "FL_BACKGROUND_COLOR", "FL_BACKGROUND2_COLOR",
                 "FL_SELECTION_COLOR", "FL_DARK3", "FL_LIGHT3"]:
        line = next(line for line in sources["Enumerations.H"].splitlines() if line.startswith(f"const Fl_Color {name} "))
        constants.append(line)
    for name in ["FL_GRAY_RAMP", "FL_NUM_GRAY", "FL_GRAY"]:
        constants.append(next(line for line in sources["Enumerations.H"].splitlines() if line.startswith(f"#define {name} ")))
    constants.append(next(line for line in sources["Enumerations.H"].splitlines() if line.startswith("inline Fl_Color fl_gray_ramp(")))
    evidence.append({"file": "Enumerations.H", "block": "color constants and fl_gray_ramp", "sha256": hashlib.sha256("\n".join(constants).encode()).hexdigest(), "changes": []})
    def extract(file, signature):
        source = sources[file]
        start = source.index(signature)
        function = source[start:source.index("\n}", start)+2]
        functions.append(function)
        evidence.append({"file": file, "function": signature, "sha256": hashlib.sha256(function.encode()).hexdigest(), "changes": []})
    for sig in ["Fl_Color fl_color_average(", "Fl_Color fl_inactive("]:
        extract("fl_color.cxx", sig)
    extract("Fl_get_system_colors.cxx", "void Fl::background(")
    for sig in ["double fl_luminance(", "double fl_lightness(", "static Fl_Color fl_contrast_cielab("]:
        extract("fl_contrast.cxx", sig)
    functions.append("Fl_Color fl_contrast(Fl_Color a,Fl_Color b) {return fl_contrast_cielab(a,b,0,0);}")
    for sig in ["static Fl_Color activated_color(", "static void vertical_gradient(", "static void horizontal_gradient(",
                "static void greybird_check_down_frame(", "static void greybird_check_down_box(",
                "static void greybird_radio_round_down_frame(", "static void greybird_radio_round_down_box(",
                "static void greybird_panel_thin_up_frame(", "static void greybird_panel_thin_up_box(",
                "static void dark_button_up_frame(", "static void dark_button_up_box(",
                "static void dark_panel_thin_up_frame(", "static void dark_panel_thin_up_box(",
                "static void dark_radio_round_down_frame(", "static void dark_radio_round_down_box("]:
        extract("themes.cpp", sig)
    for sig in ["void fl_draw_check(", "void fl_draw_radio("]:
        extract("fl_draw.cxx", sig)
    start = menu.index("  if (flags & (FL_MENU_TOGGLE|FL_MENU_RADIO)) {", menu.index("void Fl_Menu_Item::draw("))
    block = menu[start:menu.index("\n  if (!fl_draw_shortcut)", start)]
    evidence.append({"file": "Fl_Menu.cxx", "block": "indicator", "sha256": hashlib.sha256(block.encode()).hexdigest(), "changes": []})
    start = menu.index("    fl_color(FL_DARK3);", menu.index("void menuwindow::drawentry("))
    divider = menu[start:menu.index("\n  }", start)]
    evidence.append({"file": "Fl_Menu.cxx", "block": "divider", "sha256": hashlib.sha256(divider.encode()).hexdigest(), "changes": []})
    harness = r'''
#include <cmath>
#include <fstream>
#include <iostream>
#include <vector>
#include <sstream>
using uchar = unsigned char; using Fl_Color = unsigned;
'''+"\n".join(constants)+r'''
enum { FL_MENU_TOGGLE=1, FL_MENU_RADIO=2, FL_DOWN_BOX=3, FL_ROUND_DOWN_BOX=4, FL_NORMAL_SIZE=14 };
unsigned fl_cmap[256]; bool dark; Fl_Color current; std::ostringstream calls;
unsigned get_color(Fl_Color c) {return (c & 0xffffff00) ? c : fl_cmap[c];}
Fl_Color fl_rgb_color(uchar r,uchar g,uchar b) {return r||g||b ? ((unsigned)r<<24)|((unsigned)g<<16)|((unsigned)b<<8) : FL_BLACK;}
int fl_contrast_level() {return 50;}
Fl_Color fl_color() {return current;} void fl_color(Fl_Color c) {current=c;}
struct Fl_Screen_Driver {static int bg_set;}; int Fl_Screen_Driver::bg_set=0;
struct Fl {static bool is_scheme(const char*) {return true;} static bool draw_box_active() {return true;}
static int box_dw(int) {return dark?2:6;} static int menu_linespacing(){return 4;}
static void background(uchar,uchar,uchar);
static void set_color(Fl_Color i,uchar r,uchar g,uchar b){fl_cmap[i]=fl_rgb_color(r,g,b);}};
void prefix(const char *kind) {unsigned c=get_color(current); calls << kind << ',' << (c>>24) << ',' << ((c>>16)&255) << ',' << ((c>>8)&255);}
void fl_rectf(int x,int y,int w,int h) {prefix("rect"); calls<<','<<x<<','<<y<<','<<w<<','<<h<<';';}
void fl_xyline(int x,int y,int e) {fl_rectf(std::min(x,e),y,std::abs(e-x)+1,1);}
void fl_yxline(int x,int y,int e) {fl_rectf(x,std::min(y,e),1,std::abs(e-y)+1);}
void fl_xyline(int x,int y,int e,int y2) {fl_xyline(x,y,e);fl_yxline(e,y,y2);}
void fl_yxline(int x,int y,int e,int x2) {fl_yxline(x,y,e);fl_xyline(x,e,x2);}
void fl_pie(int x,int y,int w,int h,double,double) {prefix("pie");calls<<','<<x<<','<<y<<','<<w<<','<<h<<';';}
void fl_arc(int x,int y,int w,int h,double a,double b) {prefix("arc");calls<<','<<x<<','<<y<<','<<w<<','<<h<<','<<a<<','<<b<<';';}
void fl_draw_circle(int x,int y,int d,Fl_Color c) {Fl_Color old=current;fl_color(c);fl_pie(x,y,d,d,0,360);fl_color(old);}
void fl_begin_complex_polygon() {prefix("polygon");}
void fl_vertex(int x,int y) {calls<<','<<x<<','<<y;}
void fl_end_complex_polygon() {calls<<';';}
struct Fl_Rect {int X,Y,W,H;Fl_Rect(int x,int y,int w,int h):X(x),Y(y),W(w),H(h){}
int x(){return X;}int y(){return Y;}int w(){return W;}int h(){return H;}};
'''
    driver = r'''
void fl_draw_box(int box,int x,int y,int w,int h,Fl_Color c) {
 if(box==FL_ROUND_DOWN_BOX) {if(dark)dark_radio_round_down_box(x,y,w,h,c);else greybird_radio_round_down_box(x,y,w,h,c);}
 else {if(dark)dark_button_up_box(x,y,w,h,c);else greybird_check_down_box(x,y,w,h,c);}
}
struct Item {int flags;Fl_Color labelcolor_;bool on;bool value(){return on;}void draw(int x,int y,int w,int h);};
void Item::draw(int x,int y,int w,int h) {
'''+block+r'''
}
void divider(int BW,int yy,int hh,int W) {
'''+divider+r'''
}
int main(int argc,char**argv) {
 if(argc!=2)return 2;std::ofstream out(argv[1]);
 for(int theme=0;theme<2;++theme){dark=theme;
 fl_cmap[FL_BLACK]=0;fl_cmap[FL_WHITE]=0xffffff00;
 fl_cmap[FL_GRAY]=dark?0x53535300:0xcecece00;
 fl_cmap[FL_BACKGROUND2_COLOR]=dark?0x3a3a3a00:0xfcfcfc00;
 fl_cmap[FL_SELECTION_COLOR]=dark?0xd6d6d600:0x3d7bbb00;
 Fl::background(dark?83:206,dark?83:206,dark?83:206);
 for(int width:{2,17,223})for(int height:{5,41,281}){
 calls.str("");calls.clear();
 if(dark)dark_panel_thin_up_box(0,0,width,height,FL_GRAY);else greybird_panel_thin_up_box(0,0,width,height,FL_GRAY);
 out<<"panel\t"<<theme<<'\t'<<width<<'\t'<<height<<'\t'<<calls.str()<<'\n';}
 for(int width:{19,223})for(int row:{18,19,20,26})for(int index:{0,2}){
 calls.str("");calls.clear();divider(1,2+index*row,row-4,width);
 out<<"divider\t"<<theme<<'\t'<<width<<'\t'<<row<<'\t'<<index<<'\t'<<calls.str()<<'\n';}
 for(int row=18;row<=26;++row)for(int radio=0;radio<2;++radio)for(int on=0;on<2;++on){
 calls.str("");calls.clear();Item item{radio?FL_MENU_RADIO:FL_MENU_TOGGLE,dark?FL_WHITE:0x3c3c3c00u,(bool)on};item.draw(0,1,200,row-4);
 out<<theme<<'\t'<<row<<'\t'<<radio<<'\t'<<on<<'\t'<<calls.str()<<'\n';}
 for(int enabled=0;enabled<2;++enabled)for(int hover=0;hover<2;++hover){
 Fl_Color fg=dark?FL_WHITE:0x3c3c3c00;
 fg=!enabled?fl_inactive(fg):hover?fl_contrast(fg,FL_SELECTION_COLOR):fg;
 out<<"color\t"<<theme<<'\t'<<enabled<<'\t'<<hover<<'\t'<<(get_color(fg)>>8)<<'\n';}
 }return out.good()?0:3;
}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness + "\n".join(functions) + driver, encoding="utf-8")
    args.output.with_suffix(".provenance.json").write_text(json.dumps({"master": MASTER, "fltk": FLTK, "files": FILES,
        "functions": evidence}, indent=2)+"\n", encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    subprocess.run([str(args.output.resolve()), str(args.output.with_suffix(".tsv"))], check=True)
    print("Generated 72 indicator, 18 panel, 32 divider and 8 text-color cases")


if __name__ == "__main__":
    main()
