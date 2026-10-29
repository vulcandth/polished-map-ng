// Synthetic driver for unchanged upstream functions inserted by print_oracle.py.
// Copyright 2026 Polished Map NG contributors. LGPL-3.0-only.
#include <array>
#include <cstdint>
#include <cstdio>
#include <fstream>
#include <iterator>
#include <string>
#include <unordered_map>
#include <vector>
#define METATILE_PX_SIZE 32
#define EVENT_PX_SIZE 16
#define OS_FONT 0
#define OS_FONT_SIZE 13
using Fl_Align = int;
using Fl_Color = std::array<unsigned char,3>;
const Fl_Color FL_WHITE{255,255,255}, FL_BLACK{0,0,0};
Fl_Color FL_INACTIVE_COLOR;
enum { FL_COURIER_BOLD=1, FL_HELVETICA_BOLD=2, FL_ALIGN_TOP_LEFT=4, FL_ALIGN_INSIDE=8, FL_ALIGN_CENTER=0 };
static int flags, font, mask_variant;
static bool hex;
static Fl_Color color;
static std::vector<unsigned char> pixels(64*64*3);
struct Config {
    static bool print_grid() {return flags&1;}
    static bool print_ids() {return flags&2;}
    static bool print_priority() {return flags&4;}
    static bool print_events() {return flags&8;}
    static bool print_warp_ids() {return flags&16;}
};
void put(int x, int y, Fl_Color rgb, int alpha=255) {
    if(x<0 || y<0 || x>=64 || y>=64) return;
    // FLTK Xlib software fallback's 0..256 alpha normalization.
    alpha += alpha>>7;
    for(int c=0;c<3;++c) {
        auto &dst=pixels[(y*64+x)*3+c];
        dst=(rgb[c]*alpha+dst*(256-alpha))>>8;
    }
}
void fl_font(int f,int) {font=f;}
void fl_color(Fl_Color c) {color=c;}
void fl_xyline(int x,int y,int x1,int y2) {
    for(int xx=x;xx<=x1;++xx) put(xx,y,color);
    for(int yy=y2;yy<=y;++yy) put(x1,yy,color);
}
// These are test glyphs, NOT an emulation of FLTK font metrics or shaping.
void fl_draw(const char *text,int x,int y,int w,int h,Fl_Align align) {
    int length=std::string(text).size();
    int mw=mask_variant ? (font==FL_COURIER_BOLD ? 40 : 35) : length*3;
    int mh=mask_variant ? 35 : 5;
    if(align==FL_ALIGN_CENTER) {x+=(w-mw)/2; y+=(h-mh)/2;}
    for(int yy=0;yy<mh;++yy) for(int xx=0;xx<mw;++xx) {
        static const int coverage[]={0,64,128,255};
        int a=coverage[(xx+yy+static_cast<unsigned char>(text[xx%length]))%4];
        put(x+xx,y+yy,color,a);
    }
}
struct Image {
    int width; std::vector<unsigned char> rgba;
    void draw(int x,int y,int w,int h,int cx=0,int cy=0) const {
        for(int yy=0;yy<h;++yy) for(int xx=0;xx<w;++xx) {
            int i=((yy+cy)*width+xx+cx)*4;
            put(x+xx,y+yy,{rgba[i],rgba[i+1],rgba[i+2]},rgba[i+3]);
        }
    }
};
Image texture_images[6],warp_digits_image,priority_image;
enum class EventTexture {TX_RED,TX_GREEN,TX_BLUE,TX_PURPLE,TX_ORANGE,TX_AZURE};
struct Event_Meta {EventTexture texture; char symbol; uint8_t skip; bool yx,id_map;};
struct Event {
    Event_Meta _meta; int _event_x,_event_y,_warp_id; bool _hex_coords;
    void print(bool) const; void draw_warp_id(int,int) const;
};
struct Main_Window;
struct Block {
    Main_Window *parent; int _col,_row,_id; std::string text;
    void *user_data() {return parent;}
    const char *label() {return text.c_str();}
    void print();
};
struct Map {std::vector<Block> values; size_t size() {return values.size();} Block *block(size_t i) {return &values[i];}};
struct Map_Events {std::vector<Event> values; size_t size() {return values.size();} Event *event(size_t i) {return &values[i];}};
struct Main_Window {
    Map _map; Map_Events _map_events;
    void print_map();
    void print_metatile(int x,int y,int id) {
        for(int yy=0;yy<32;++yy) for(int xx=0;xx<32;++xx) {
            Fl_Color rgb = id==0 ? Fl_Color{20,40,60} : id==255 ? Fl_Color{171,205,239} :
                (xx/8)%2 ? Fl_Color{100,180,220} : Fl_Color{200,100,50};
            put(x+xx,y+yy,rgb);
        }
        if(Config::print_priority() && id==1)
            for(int row=0;row<4;++row) for(int col : {1,3}) priority_image.draw(x+col*8,y+row*8,8,8);
    }
};
// INSERT_ORIGINAL
int main(int argc,char **argv) {
    if(argc!=4) return 2;
    std::ifstream assets(argv[1],std::ios::binary),events(argv[2]);
    std::ofstream out(argv[3],std::ios::binary);
    for(auto &image:texture_images) {image.width=16; image.rgba.resize(16*16*4); assets.read(reinterpret_cast<char*>(image.rgba.data()),image.rgba.size());}
    warp_digits_image.width=80; warp_digits_image.rgba.resize(80*7*4);
    assets.read(reinterpret_cast<char*>(warp_digits_image.rgba.data()),warp_digits_image.rgba.size());
    priority_image.width=8; priority_image.rgba.resize(8*8*4);
    assets.read(reinterpret_cast<char*>(priority_image.rgba.data()),priority_image.rgba.size());
    if(!assets) return 3;
    Main_Window mw;
    std::string name; int x,y,is_hex,warp=0;
    while(events>>name>>x>>y>>is_hex) {
        auto meta=events_meta.at(name);
        mw._map_events.values.push_back({meta,x,y,meta.texture==EventTexture::TX_PURPLE ? ++warp : 0,bool(is_hex)});
    }
    for(mask_variant=0;mask_variant<2;++mask_variant)
    for(int grid : {85,38}) for(int base=0;base<2;++base) for(flags=0;flags<32;++flags) {
        hex=base; FL_INACTIVE_COLOR={static_cast<unsigned char>(grid),static_cast<unsigned char>(grid),static_cast<unsigned char>(grid)};
        mw._map.values.clear();
        int ids[]={0,1,255,1};
        for(int i=0;i<4;++i) {
            char label[4]; if(hex) snprintf(label,sizeof(label),"%02X",ids[i]); else snprintf(label,sizeof(label),"%d",ids[i]);
            mw._map.values.push_back({&mw,i%2,i/2,ids[i],label});
        }
        mw.print_map();
        out.write(reinterpret_cast<char*>(pixels.data()),pixels.size());
    }
    return out.good() ? 0 : 4;
}
