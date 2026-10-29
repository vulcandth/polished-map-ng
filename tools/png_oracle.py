"""Build unchanged master/FLTK PNG import functions with pinned bundled libpng/zlib.

All generated source, copied library sources and binaries stay in ignored scratch
storage. No display server or native window is needed. See NOTICE.md for terms.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

MASTER = "8b9bdc2b8aac8c865809516c2123af69c498da0b"
FLTK = "a9b1113516ffd15fc7602a6d425a317df30f4720"
PNG_C = "png pngerror pngget pngmem pngpread pngread pngrio pngrtran pngrutil pngset pngtrans pngwio pngwrite pngwtran pngwutil".split()
Z_C = "adler32 compress crc32 deflate gzclose gzlib gzread gzwrite inflate infback inftrees inffast trees uncompr zutil".split()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--checkout", type=Path, required=True)
    p.add_argument("--fltk-checkout", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--git", default="git")
    p.add_argument("--cc", default="gcc")
    p.add_argument("--cxx", default="g++")
    args = p.parse_args()
    build = args.output.with_suffix(".build")
    build.mkdir(parents=True, exist_ok=True)
    evidence = []

    def original(checkout, revision, path):
        return subprocess.check_output([args.git, "-C", str(checkout), "show", f"{revision}:{path}"])

    def snippet(checkout, revision, path, start, end):
        source = original(checkout, revision, path).decode("utf-8")
        a = source.index(start)
        b = source.index(end, a) + len(end)
        text = source[a:b]
        evidence.append({"revision": revision, "file": path, "start": start,
                         "sha256": hashlib.sha256(text.encode()).hexdigest(), "changes": []})
        return text

    def master(path, start, end):
        return snippet(args.checkout, MASTER, "src/" + path, start, end)

    def fltk(path, start, end):
        return snippet(args.fltk_checkout, FLTK, path, start, end)

    paths = subprocess.check_output([args.git, "-C", str(args.fltk_checkout), "ls-tree", "-r", "--name-only", FLTK, "png", "zlib"]).decode().splitlines()
    for name in paths:
        path = Path(name)
        if path.suffix not in [".c", ".h"] and path.name not in ["LICENSE", "README"]:
            continue
        raw = original(args.fltk_checkout, FLTK, name)
        target = build / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
        evidence.append({"revision": FLTK, "file": name, "sha256": hashlib.sha256(raw).hexdigest(), "changes": []})

    constants = "\n".join(master(name, start, "\n") for name, start in [
        ("tile.h", "#define TILE_SIZE "), ("tile.h", "#define TILE_AREA "),
        ("palette-map.h", "#define MAX_NUM_TILES "), ("colors.h", "#define NUM_HUES ")])
    hue = master("colors.h", "enum class Hue", ";")
    result = master("tiled-image.h", "enum class Result", ";")
    mono_table = master("colors.cpp", "static const Hue mono_hues", ";")
    mono = master("colors.cpp", "Hue Color::mono_hue(", "\n}")
    loader = master("tiled-image.cpp", "Tiled_Image::Result Tiled_Image::read_png_graphics(", "\n}")
    read_memory = fltk("src/Fl_PNG_Image.cxx", "typedef struct", '} // extern "C"')
    constructor = fltk("src/Fl_PNG_Image.cxx", "Fl_PNG_Image::Fl_PNG_Image (const char *filename)", "\n}")
    png_load = fltk("src/Fl_PNG_Image.cxx", "void Fl_PNG_Image::load_png_(", "\n}")
    desaturate = fltk("src/Fl_Image.cxx", "void Fl_RGB_Image::desaturate()", "\n}")
    windows = fltk("src/drivers/WinAPI/Fl_WinAPI_System_Driver.cxx", "void Fl_WinAPI_System_Driver::png_extra_rgba_processing(", "\n}")
    header = r'''
// Extracted functions: original FLTK/master notices and terms apply.
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iostream>
#include <vector>
extern "C" {
#include <zlib.h>
#include <png.h>
}
#define HAVE_LIBPNG 1
#define HAVE_LIBZ 1
#define HAVE_PNG_GET_VALID 1
#define HAVE_PNG_SET_TRNS_TO_ALPHA 1
using uchar = unsigned char;
bool windows_mode = false;
struct Fl_System_Driver { virtual void png_extra_rgba_processing(uchar*, int, int) {} };
struct Fl_WinAPI_System_Driver : Fl_System_Driver { void png_extra_rgba_processing(uchar*, int, int) override; };
struct Fl {
    static Fl_System_Driver *system_driver() { static Fl_System_Driver normal; static Fl_WinAPI_System_Driver win; return windows_mode ? &win : &normal; }
    static void warning(const char*, ...) {}
};
FILE *fl_fopen(const char *path, const char *mode) { return std::fopen(path, mode); }
struct Fl_RGB_Image {
    const uchar *array; int alloc_array = 0;
    int width, height, depth, stride = 0;
    enum {ERR_FILE_ACCESS=-2, ERR_FORMAT=-3};
    Fl_RGB_Image(const uchar *a,int w,int h,int d=3):array(a),width(w),height(h),depth(d){}
    ~Fl_RGB_Image(){if(alloc_array) delete[] array;}
    int w()const{return width;} void w(int v){width=v;}
    int h()const{return height;} void h(int v){height=v;}
    int d()const{return depth;} void d(int v){depth=v;}
    int ld()const{return stride;} void ld(int v){stride=v;}
    int data_w()const{return width;} int data_h()const{return height;}
    int count()const{return array?1:0;} int fail()const{return !array || !width || !height || stride<0;}
    static size_t max_size(){return 64*1024*1024;}
    void uncache(){} void desaturate();
};
struct Fl_PNG_Image : Fl_RGB_Image {
    Fl_PNG_Image(const char*);
    void load_png_(const char*,int,const uchar*,int);
};
struct Fl_Shared_Image {Fl_Shared_Image(const char*,Fl_PNG_Image*){} void add(){}};
'''
    declarations = "struct Color {static Hue mono_hue(uchar);};\nstruct Tiled_Image {" + result + r'''
    std::vector<Hue> _tile_hues; size_t _num_tiles=0; Result _result=Result::IMG_NULL;
    Result read_png_graphics(const char*);
};
'''
    driver = r'''
bool read32(std::istream &in,uint32_t &v){unsigned char b[4];if(!in.read((char*)b,4))return false;v=uint32_t(b[0])|uint32_t(b[1])<<8|uint32_t(b[2])<<16|uint32_t(b[3])<<24;return true;}
void write32(std::ostream &out,uint32_t v){for(int i=0;i<4;++i)out.put(char(v>>(8*i)));}
// Fixture preparation only: libpng repacks the same samples as Adam7. This is
// separate from the unchanged decoder under test and never draws a window.
struct PngInput {const std::vector<char> *bytes;size_t offset=0;};
void fixture_read(png_structp p,png_bytep out,png_size_t n){
    auto *input=static_cast<PngInput*>(png_get_io_ptr(p));
    if(n>input->bytes->size()-input->offset)png_error(p,"short fixture");
    std::memcpy(out,input->bytes->data()+input->offset,n);input->offset+=n;
}
void fixture_write(png_structp p,png_bytep bytes,png_size_t n){
    auto *output=static_cast<std::vector<unsigned char>*>(png_get_io_ptr(p));
    output->insert(output->end(),bytes,bytes+n);
}
void fixture_flush(png_structp){}
std::vector<unsigned char> interlace(const std::vector<char> &bytes){
    PngInput input{&bytes};std::vector<unsigned char> output;
    png_structp reader=png_create_read_struct(PNG_LIBPNG_VER_STRING,nullptr,nullptr,nullptr);
    png_infop info=png_create_info_struct(reader);
    if(setjmp(png_jmpbuf(reader)))std::exit(10);
    png_set_read_fn(reader,&input,fixture_read);
    png_read_png(reader,info,PNG_TRANSFORM_IDENTITY,nullptr);
    png_uint_32 w,h;int depth,color,method,compression,filter;
    png_get_IHDR(reader,info,&w,&h,&depth,&color,&method,&compression,&filter);
    png_structp writer=png_create_write_struct(PNG_LIBPNG_VER_STRING,nullptr,nullptr,nullptr);
    if(setjmp(png_jmpbuf(writer)))std::exit(11);
    png_set_write_fn(writer,&output,fixture_write,fixture_flush);
    png_set_IHDR(writer,info,w,h,depth,color,PNG_INTERLACE_ADAM7,compression,filter);
    png_write_png(writer,info,PNG_TRANSFORM_IDENTITY,nullptr);
    png_destroy_write_struct(&writer,nullptr);png_destroy_read_struct(&reader,&info,nullptr);
    return output;
}
int main(int argc,char**argv){
    const bool repack=argc==5 && std::strcmp(argv[4],"--interlace")==0;
    if(argc!=4 && !repack)return 2;
    std::ifstream in(argv[1],std::ios::binary);std::ofstream out(argv[2],std::ios::binary);
    uint32_t count=0;if(!read32(in,count)||count>10000)return 3;
    if(repack)write32(out,count);
    for(uint32_t c=0;c<count;++c){
        uint32_t n=0;if(!read32(in,n)||n>1024*1024)return 4;
        std::vector<char> bytes(n);if(!in.read(bytes.data(),n))return 5;
        if(repack){auto png=interlace(bytes);write32(out,uint32_t(png.size()));out.write((char*)png.data(),png.size());continue;}
        {std::ofstream f(argv[3],std::ios::binary|std::ios::trunc);f.write(bytes.data(),n);if(!f.good())return 6;}
        for(int mode=0;mode<2;++mode){windows_mode=mode;Tiled_Image image;
            write32(out,uint32_t(image.read_png_graphics(argv[3])));write32(out,uint32_t(image._num_tiles));
            write32(out,uint32_t(image._tile_hues.size()));for(Hue h:image._tile_hues)out.put(char(h));
        }
        if(!out.good())return 7;
    }
    return in.peek()==EOF?0:8;
}
'''
    cpp = args.output.with_suffix(".cpp")
    cpp.write_text("\n".join([header, constants, hue, declarations, mono_table, mono, windows, read_memory, constructor, png_load, desaturate, loader, driver]), encoding="utf-8", newline="\n")
    args.output.with_suffix(".provenance.json").write_text(json.dumps({
        "functions_and_libraries": evidence,
        "adapters": ["ASCII std::fopen paths", "storage-only image declarations; no drawing/shared-image cache",
                     "64-MiB allocation cap (valid corpus is much smaller)", "warning messages suppressed; statuses retained",
                     "both original Unix no-op and unchanged Windows RGBA processing", "portable C libpng/zlib, optional SIMD disabled",
                     "separate libpng writer repacks test samples as Adam7 fixtures"]
    }, indent=2) + "\n", encoding="utf-8")
    includes = ["-I" + str(build / "png"), "-I" + str(build / "zlib")]
    objects = []
    for folder, names in [("png", PNG_C), ("zlib", Z_C)]:
        for name in names:
            obj = build / f"{folder}-{name}.o"
            subprocess.run([args.cc, "-O2", "-D_DEFAULT_SOURCE=1", "-DPNG_ARM_NEON_OPT=0", "-DPNG_POWERPC_VSX_OPT=0", "-DPNG_INTEL_SSE_OPT=0", *includes,
                            "-c", str(build / folder / (name + ".c")), "-o", str(obj)], check=True)
            objects.append(str(obj))
    subprocess.run([args.cxx, "-std=c++17", "-O2", *includes, str(cpp), *objects, "-o", str(args.output)], check=True)


if __name__ == "__main__":
    main()
