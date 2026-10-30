"""Compile the pinned, unchanged FLTK slider handler for drag differentials.

This records numeric interaction, not desktop events. Generated derivatives stay
in ignored scratch storage. FLTK copyright/terms: LICENSES/FLTK.txt.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import urllib.request

FLTK = "a9b1113516ffd15fc7602a6d425a317df30f4720"
FILES = {
    "Fl_Slider.cxx": "7c15115be10cbafd6799b775e0d5a57a3722d93a56b46a1327ad40176de0f3d2",
    "Fl_Valuator.cxx": "a9a95971896ee6ba3a6543bbfff405aae624624d13a9e423a2ec34a68d1ee8d4",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--fltk-cache", type=Path, required=True)
    parser.add_argument("--cxx", default="g++")
    args = parser.parse_args()
    args.fltk_cache.mkdir(parents=True, exist_ok=True)
    functions, evidence = [], []
    for name, expected in FILES.items():
        path = args.fltk_cache / name
        if not path.exists():
            with urllib.request.urlopen(f"https://raw.githubusercontent.com/fltk/fltk/{FLTK}/src/{name}", timeout=30) as response:
                path.write_bytes(response.read())
        raw = path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != expected:
            raise ValueError(f"Unexpected pinned FLTK hash: {name}")
        source = raw.decode("utf-8")
        signatures = ["int Fl_Slider::handle(int event, int X,"] if name == "Fl_Slider.cxx" else ["double Fl_Valuator::round(", "double Fl_Valuator::clamp(", "double Fl_Valuator::increment("]
        for signature in signatures:
            start = source.index(signature)
            function = source[start:source.index("\n}", start) + 2]
            functions.append(function)
            evidence.append({"file": name, "function": signature, "sha256": hashlib.sha256(function.encode()).hexdigest(), "changes": []})
    harness = r'''
#include <algorithm>
#include <cmath>
#include <fstream>
bool windows_round;
// FL/math.h's Windows replacement versus the Unix default rounding mode.
double reference_rint(double v) {return windows_round?std::floor(v+.5):std::rint(v);}
#define rint reference_rint
enum {FL_PUSH=1,FL_DRAG,FL_RELEASE,FL_KEYBOARD,FL_FOCUS,FL_UNFOCUS,FL_ENTER,FL_LEAVE,
FL_Up,FL_Down,FL_Left,FL_Right,FL_HOR_FILL_SLIDER,FL_VERT_FILL_SLIDER,FL_VERT_NICE_SLIDER,FL_HOR_NICE_SLIDER};
struct Fl {static int mx; static bool event_inside(int X,int Y,int W,int H){return mx>=X&&mx<X+W;}
static int event_x(){return mx;}static int event_y(){return 0;}static int event_key(){return 0;}
static bool visible_focus(){return true;}};int Fl::mx;
struct Fl_Widget_Tracker {Fl_Widget_Tracker(void*){}bool deleted(){return false;}};
struct Fl_Valuator {double A=1,B=1,min=0,max=31,v=0;double value(){return v;}double minimum(){return min;}double maximum(){return max;}
double round(double);double clamp(double);double increment(double,int);
void handle_push(){}void handle_release(){}void handle_drag(double n){v=n;}void redraw(){}};
struct Fl_Slider:Fl_Valuator {float slider_size_=0;bool horizontal(){return true;}int type(){return FL_HOR_NICE_SLIDER;}
int handle(int,int,int,int,int);};
'''
    driver = r'''
int main(int argc,char**argv){if(argc!=2)return 2;std::ofstream out(argv[1]);
for(int mode=0;mode<2;++mode){windows_round=mode;
for(int width:{28,92,101})for(int initial=0;initial<32;++initial)for(int press=0;press<width;++press){
Fl_Slider slider;slider.v=initial;
int step=0;for(int mouse:{press,press+1,-20,width+20,width/2,0}){
Fl::mx=mouse;slider.handle(step==0?FL_PUSH:FL_DRAG,0,0,width,22);
out<<mode<<'\t'<<width<<'\t'<<initial<<'\t'<<press<<'\t'<<step<<'\t'<<mouse<<'\t'<<slider.v<<'\n';++step;}
}}return out.good()?0:3;}
'''
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text(harness + "\n".join(functions) + driver, encoding="utf-8")
    args.output.with_suffix(".provenance.json").write_text(json.dumps({"fltk": FLTK, "files": FILES, "functions": evidence}, indent=2) + "\n", encoding="utf-8")
    subprocess.run([args.cxx, "-std=c++17", "-O2", str(cpp), "-o", str(args.output)], check=True)
    subprocess.run([str(args.output.resolve()), str(args.output.with_suffix(".tsv"))], check=True)
    print("Generated 84,864 drag observations for Windows and Unix rounding")


if __name__ == "__main__":
    main()
