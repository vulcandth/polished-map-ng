//! JSON result records and stable error codes. LGPL-3.0-only.
use polished_map_core::PaletteSet;
use polished_map_project::{
    BlocksetDocument, EventDocument, MasterTilesetDocument, PaletteDocument, ProjectError,
    RoofDocument,
};
use serde_json::{Value, json};

pub(crate) fn palette_summary(doc: &PaletteDocument) -> Value {
    json!({"content_hash":doc.content_hash(),"selected":doc.selected(),"sets":PaletteSet::ALL.map(|set| json!({"set":set,"colors":doc.bank().colors(set)}))})
}

pub(crate) fn event_summary(doc: &EventDocument) -> Value {
    json!({"source":doc.source().path(), "content_hash":doc.source().content_hash(),
        "modified":doc.script().events().is_modified(),
        "events":doc.script().events().entries().iter().enumerate().map(|(index,e)|
            json!({"index":index,"definition":e.definition(),"warp_id":e.warp_id(),
                "warp_destination":e.warp_destination(),"edited_text":e.is_rewritten().then(|| e.text())})).collect::<Vec<_>>()})
}

pub(crate) fn roof_summary(doc: &RoofDocument) -> Value {
    json!({"name":doc.name(),"png_platform":doc.png_platform(),"content_hash":doc.content_hash(),"pixels":doc.roof().tileset().tiles().iter().map(|t|t.hues().as_slice()).collect::<Vec<_>>(),"source":doc.source().path(),"output":doc.output().path()})
}

pub(crate) fn tileset_summary(doc: &MasterTilesetDocument, blocks: &BlocksetDocument) -> Value {
    json!({"name":doc.name(), "png_platform":doc.png_platform(), "content_hash":doc.content_hash(blocks.blockset()), "allow_256":doc.allow_256(), "monochrome":doc.monochrome(), "allow_priority":doc.allow_priority(),
        "pixels":doc.tileset().tiles().iter().map(|t| t.hues().as_slice()).collect::<Vec<_>>(), "attributes":doc.tileset().attributes(),
        "sources":doc.source_files().iter().map(|s| json!({"path":s.path(),"content_hash":s.content_hash()})).collect::<Vec<_>>(),
        "outputs":doc.outputs().iter().map(|s| json!({"path":s.path(),"content_hash":s.content_hash()})).collect::<Vec<_>>()})
}

pub(crate) struct Failure {
    pub(crate) code: String,
    pub(crate) message: String,
}

impl Failure {
    pub(crate) fn new(code: &str, message: impl ToString) -> Self {
        Self {
            code: code.into(),
            message: message.to_string(),
        }
    }
}

impl From<ProjectError> for Failure {
    fn from(error: ProjectError) -> Self {
        Self::new(error.code(), error)
    }
}
