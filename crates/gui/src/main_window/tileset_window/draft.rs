//! Explicit asset drafts behind the shared tile/roof controls. Adapts the pinned
//! master/plusplus Tileset_Window and Roof_Window editing domains; LGPL-3.0-only.
use polished_map_core::{MasterTilesetDocument, attributed};
use polished_map_core::{
    Palette, TileCommand, TileSnapshot, UNDEFINED_COLORS,
    attributed::{BankOrder, RawTileCommand, RawTileSnapshot, TileReference},
};

pub(crate) enum Draft {
    Classic {
        document: Box<MasterTilesetDocument>,
        roof: bool,
    },
    AttributedTiles {
        document: Box<attributed::TilesetDocument>,
        allow_512: bool,
        order: BankOrder,
    },
    AttributedRoof(Box<attributed::RoofDocument>),
}
#[derive(Clone)]
pub(crate) enum Snapshot {
    Classic(TileSnapshot),
    Raw(RawTileSnapshot),
}

const MONO: Palette = [[255; 3], [85; 3], [170; 3], [0; 3]];
fn small(id: u16) -> Result<u8, String> {
    u8::try_from(id).map_err(|_| "Master tile index exceeds 255".into())
}

impl Draft {
    pub fn is_attributed(&self) -> bool {
        !matches!(self, Self::Classic { .. })
    }
    pub fn classic(&self) -> Option<&MasterTilesetDocument> {
        match self {
            Self::Classic { document, .. } => Some(document),
            _ => None,
        }
    }
    #[cfg(test)]
    pub fn classic_mut(&mut self) -> Option<&mut MasterTilesetDocument> {
        match self {
            Self::Classic { document, .. } => Some(document),
            _ => None,
        }
    }
    fn classic_model(&self) -> Option<&polished_map_core::Tileset> {
        match self {
            Self::Classic {
                document,
                roof: true,
            } => document.roof().map(|r| r.roof().tileset()),
            Self::Classic {
                document,
                roof: false,
            } => Some(document.tileset()),
            _ => None,
        }
    }
    pub fn count(&self) -> usize {
        match self {
            Self::Classic { .. } => self
                .classic_model()
                .expect("validated master draft")
                .tiles()
                .len(),
            Self::AttributedTiles { document, .. } => document.model().tiles().len(),
            Self::AttributedRoof(_) => 9,
        }
    }
    pub fn visible_count(&self) -> usize {
        match self {
            Self::AttributedTiles {
                allow_512: false, ..
            } => 256,
            _ => self.count(),
        }
    }
    pub fn allow_extended(&self) -> bool {
        match self {
            Self::Classic { document, .. } => document.allow_256(),
            Self::AttributedTiles { allow_512, .. } => *allow_512,
            Self::AttributedRoof(_) => true,
        }
    }
    pub fn enabled(&self, id: u16) -> bool {
        if usize::from(id) >= self.visible_count() {
            return false;
        }
        match self {
            Self::Classic {
                document,
                roof: false,
            } => {
                document.allow_256()
                    || id < 0x60
                    || (!document.monochrome() && (0x80..0xe0).contains(&id))
            }
            _ => true,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Classic {
                document,
                roof: true,
            } => document.roof().expect("validated roof draft").name(),
            Self::Classic {
                document,
                roof: false,
            } => document.name(),
            Self::AttributedTiles { document, .. } => document.name(),
            Self::AttributedRoof(d) => d.name(),
        }
    }
    pub fn label(&self, id: u16, roof: bool) -> String {
        let id = id + if roof { 0x0a } else { 0 };
        if self.is_attributed() {
            let order = match self {
                Self::AttributedTiles { order, .. } => *order,
                _ => BankOrder::default(),
            };
            let t = TileReference::from_index(id, order).expect("visible tile ID");
            format!("${}:{:02X}", u8::from(t.bank1()), t.offset())
        } else {
            format!("${id:02X}")
        }
    }
    pub fn attribute(&self, id: u16) -> Option<u8> {
        self.classic_model()
            .and_then(|m| m.attributes().get(usize::from(id)).copied())
    }
    pub fn colors(&self, id: u16) -> Palette {
        if let Some(d) = self.classic() {
            let attr = self.attribute(id).expect("selected tile");
            if attr == 9 {
                UNDEFINED_COLORS
            } else {
                d.colors()[usize::from(attr & 15)]
            }
        } else {
            MONO
        }
    }
    pub fn pixels(&self, id: u16) -> [u8; 64] {
        match self {
            Self::Classic { .. } => *self
                .classic_model()
                .expect("validated master draft")
                .tiles()[usize::from(id)]
            .hues(),
            Self::AttributedTiles { document, .. } => {
                *document.model().tiles()[usize::from(id)].hues()
            }
            Self::AttributedRoof(d) => *d.tiles()[usize::from(id)].hues(),
        }
    }
    pub fn render_tile(&self, id: u16) -> Result<polished_map_core::RgbImage, String> {
        match self {
            Self::Classic { document, .. } => self
                .classic_model()
                .expect("validated master draft")
                .render_tile(small(id)?, document.colors())
                .map_err(|e| e.to_string()),
            Self::AttributedTiles { document, .. } => {
                document.model().render_tile(id).map_err(|e| e.to_string())
            }
            Self::AttributedRoof(d) => d.render_tile(id).map_err(|e| e.to_string()),
        }
    }
    pub fn snapshot(&self, id: u16) -> Result<Snapshot, String> {
        match self {
            Self::Classic { .. } => self
                .classic_model()
                .expect("validated master draft")
                .snapshot(small(id)?)
                .map(Snapshot::Classic)
                .map_err(|e| e.to_string()),
            Self::AttributedTiles { document, .. } => document
                .model()
                .snapshot(id)
                .map(Snapshot::Raw)
                .map_err(|e| e.to_string()),
            Self::AttributedRoof(d) => d.snapshot(id).map(Snapshot::Raw).map_err(|e| e.to_string()),
        }
    }
    pub fn set_snapshot(&mut self, id: u16, value: Snapshot) -> Result<(), String> {
        match value {
            Snapshot::Classic(value) if !self.is_attributed() => {
                self.apply_classic(TileCommand::Set {
                    tile: small(id)?,
                    value,
                })
            }
            Snapshot::Raw(value) if self.is_attributed() => {
                self.apply(RawTileCommand::Set { tile: id, value })
            }
            _ => Err("Tile snapshot belongs to another editor profile".into()),
        }
    }
    pub fn set_palette(&mut self, id: u16, palette: u8) -> Result<(), String> {
        self.apply_classic(TileCommand::Palette {
            tile: small(id)?,
            palette,
        })
    }
    fn apply_classic(&mut self, command: TileCommand) -> Result<(), String> {
        let Self::Classic { document, roof } = self else {
            return Err("Raw tiles have no per-tile palette attribute".into());
        };
        if *roof {
            document
                .roof_mut()
                .expect("validated roof draft")
                .roof_mut()
                .apply_batch(&[command])
                .map_err(|e| e.to_string())
        } else {
            document.apply_batch(&[command]).map_err(|e| e.to_string())
        }
    }
    pub fn apply(&mut self, command: RawTileCommand) -> Result<(), String> {
        match self {
            Self::AttributedTiles { document, .. } => {
                return document.apply_batch(&[command]).map_err(|e| e.to_string());
            }
            Self::AttributedRoof(d) => return d.apply_batch(&[command]).map_err(|e| e.to_string()),
            Self::Classic { .. } => {}
        }
        // Pixel operations are shared, but master's palette-bearing snapshots
        // use set_snapshot so copy/paste never drops their attributes.
        let command = match command {
            RawTileCommand::Paint { tile, points, hue } => TileCommand::Paint {
                tile: small(tile)?,
                points,
                hue,
            },
            RawTileCommand::Fill { tile, at, hue } => TileCommand::Fill {
                tile: small(tile)?,
                at,
                hue,
            },
            RawTileCommand::Replace { tile, from, to } => TileCommand::Replace {
                tile: small(tile)?,
                from,
                to,
            },
            RawTileCommand::SwapHues { tile, a, b } => TileCommand::SwapHues {
                tile: small(tile)?,
                a,
                b,
            },
            RawTileCommand::Copy { from, to } => TileCommand::Copy {
                from: small(from)?,
                to: small(to)?,
            },
            RawTileCommand::Swap { a, b } => TileCommand::Swap {
                a: small(a)?,
                b: small(b)?,
            },
            RawTileCommand::Delete { tile } => TileCommand::Delete { tile: small(tile)? },
            RawTileCommand::ImportRgb {
                tile,
                width,
                height,
                pixels,
                ..
            } => TileCommand::ImportRgb {
                tile: small(tile)?,
                width,
                height,
                pixels,
                allow_256: self.allow_extended(),
            },
            RawTileCommand::Set { .. } => {
                return Err("Master Set requires a palette-bearing snapshot".into());
            }
        };
        self.apply_classic(command)
    }
}
