use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use polished_map_core::{Graphics, Palette, RgbImage, Tile, day_palettes};

use crate::{
    MapDocument, MasterGraphicsOptions, ProjectError, codecs, palettes::map_palettes, read_bounded,
};

/// Load graphics with per-project palettes, map metadata and roof colors.
pub fn load_master_map_graphics(
    doc: &MapDocument,
    tileset: &str,
    options: &MasterGraphicsOptions,
) -> Result<Graphics, ProjectError> {
    validate_tileset(doc.path(), tileset)?;
    let root = doc
        .metadata()
        .root
        .as_deref()
        .ok_or_else(|| ProjectError::Format {
            path: doc.path().into(),
            message: "project root is required for graphics".into(),
        })?;
    let mut options = options.clone();
    options.monochrome |= !palette_map_path(root, tileset).is_file();
    let colors = map_palettes(doc, tileset, &options)?;
    load_graphics(
        root,
        tileset,
        options.allow_256,
        &colors,
        options.monochrome,
    )
}

/// Load raw master tilesets with upstream DAY defaults, without map metadata.
pub fn load_master_graphics(
    root: &Path,
    tileset: &str,
    allow_256: bool,
) -> Result<Graphics, ProjectError> {
    load_graphics(root, tileset, allow_256, &day_palettes(), false)
}

fn validate_tileset(root: &Path, tileset: &str) -> Result<(), ProjectError> {
    if tileset.is_empty() || tileset.contains(['/', '\\']) || matches!(tileset, "." | "..") {
        return Err(ProjectError::Format {
            path: root.into(),
            message: "tileset must be a project-local name".into(),
        });
    }
    Ok(())
}

fn load_graphics(
    root: &Path,
    tileset: &str,
    allow_256: bool,
    colors: &[Palette; 9],
    monochrome: bool,
) -> Result<Graphics, ProjectError> {
    validate_tileset(root, tileset)?;
    let gfx = if root.join("MAPDATA").is_dir() {
        root.join("MAPDATA")
    } else {
        root.join("gfx/tilesets")
    };
    let graphics_path = first_existing(&[
        gfx.join(format!("{tileset}.png")),
        gfx.join(format!("{tileset}.2bpp")),
        gfx.join(format!("{tileset}.CHR")),
        gfx.join(format!("{tileset}.2bpp.lz")),
    ]);
    let tiles = read_tiles(&graphics_path)?;
    let base = tileset.split('.').next().unwrap_or(tileset);
    let mut block_paths = Vec::new();
    for name in [tileset, base] {
        block_paths.extend([
            root.join(format!("data/tilesets/{name}_metatiles.bin")),
            root.join(format!("gfx/blocksets/{name}.bst")),
            root.join(format!("MAPDATA/{name}.CEL")),
            root.join(format!("tilesets/{name}_metatiles.bin")),
        ]);
    }
    let blocks_path = first_existing(&block_paths);
    let raw = read_bounded(&blocks_path, 4 * 1024 * 1024)?;
    let bytes = if blocks_path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cel"))
    {
        codecs::parse_asm(&text(&blocks_path, &raw)?)
            .map_err(|e| ProjectError::codec(&blocks_path, e))?
    } else {
        raw
    };
    if bytes.is_empty() || bytes.len() > 4096 || !bytes.len().is_multiple_of(16) {
        return Err(ProjectError::Format {
            path: blocks_path,
            message: "expected 1..=256 complete 16-byte metatiles".into(),
        });
    }
    let metatiles = bytes
        .as_chunks::<16>()
        .0
        .iter()
        .map(|b| {
            let mut block = [0; 16];
            block.copy_from_slice(b);
            block
        })
        .collect();
    let palette_path = palette_map_path(root, tileset);
    let ids = if !monochrome && palette_path.is_file() {
        let bytes = read_bounded(&palette_path, 1024 * 1024)?;
        codecs::parse_palette_map(&text(&palette_path, &bytes)?)
            .map_err(|e| ProjectError::codec(&palette_path, e))?
    } else {
        vec![8; if allow_256 { 256 } else { 96 }]
    };
    let undefined: Palette = [
        [0xab, 0xcd, 0xef],
        [0x45, 0x67, 0x89],
        [0x78, 0x9a, 0xbc],
        [0x12, 0x34, 0x56],
    ];
    let blank = Tile::new([0; 64])?;
    let mut mapped_tiles = vec![blank; 256];
    let mut palettes = vec![undefined; 256];
    for (i, tile) in tiles.into_iter().enumerate() {
        let j = if !allow_256 && i >= 0x60 {
            if i >= 0xe0 { i - 0x80 } else { i + 0x20 }
        } else {
            i
        };
        mapped_tiles[j] = tile;
        if let Some(&id) = ids.get(i) {
            palettes[j] = colors[if monochrome { 8 } else { usize::from(id & 15) }];
        }
    }
    Ok(Graphics::new(mapped_tiles, palettes, metatiles)?)
}

fn palette_map_path(root: &Path, tileset: &str) -> PathBuf {
    let base = tileset.split('.').next().unwrap_or(tileset);
    let mut paths = Vec::new();
    for name in [tileset, base] {
        paths.extend([
            root.join(format!("gfx/tilesets/{name}_palette_map.asm")),
            root.join(format!("gfx/tilesets/{name}_palette_map.inc")),
            root.join(format!("color/tilesets/{name}.asm")),
            root.join(format!("tilesets/{name}_palette_map.asm")),
        ]);
    }
    first_existing(&paths)
}

fn first_existing(paths: &[PathBuf]) -> PathBuf {
    paths
        .iter()
        .find(|p| p.is_file())
        .unwrap_or(&paths[paths.len() - 1])
        .clone()
}

fn text(path: &Path, bytes: &[u8]) -> Result<String, ProjectError> {
    String::from_utf8(bytes.to_vec()).map_err(|e| ProjectError::Format {
        path: path.into(),
        message: e.to_string(),
    })
}

pub fn read_tiles(path: &Path) -> Result<Vec<Tile>, ProjectError> {
    let bytes = read_bounded(path, 8 * 1024 * 1024)?;
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "2bpp" => codecs::decode_2bpp(&bytes).map_err(|e| ProjectError::codec(path, e)),
        "lz" => codecs::decode_lz_tiles(&bytes).map_err(|e| ProjectError::codec(path, e)),
        "chr" => codecs::decode_2bpp(
            &codecs::parse_asm(&text(path, &bytes)?).map_err(|e| ProjectError::codec(path, e))?,
        )
        .map_err(|e| ProjectError::codec(path, e)),
        "png" => {
            let mut reader =
                image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
            let mut limits = image::Limits::default();
            limits.max_image_width = Some(2048);
            limits.max_image_height = Some(2048);
            limits.max_alloc = Some(8 * 1024 * 1024);
            reader.limits(limits);
            let image = reader
                .decode()
                .map_err(|e| ProjectError::Format {
                    path: path.into(),
                    message: e.to_string(),
                })?
                .to_rgb8();
            let (width, height) = image.dimensions();
            if width == 0
                || height == 0
                || width % 8 != 0
                || height % 8 != 0
                || width * height > 256 * 64
            {
                return Err(ProjectError::Format {
                    path: path.into(),
                    message: "PNG must contain 1..=256 complete 8×8 tiles".into(),
                });
            }
            let mut tiles = Vec::new();
            for ty in 0..height / 8 {
                for tx in 0..width / 8 {
                    let hues = std::array::from_fn(|i| {
                        let rgb = image
                            .get_pixel(tx * 8 + (i % 8) as u32, ty * 8 + (i / 8) as u32)
                            .0;
                        let gray = (u16::from(rgb[0]) * 31
                            + u16::from(rgb[1]) * 61
                            + u16::from(rgb[2]) * 8)
                            / 100;
                        [3, 1, 2, 0][usize::from(gray / 64)]
                    });
                    tiles.push(Tile::new(hues)?);
                }
            }
            Ok(tiles)
        }
        _ => Err(ProjectError::Format {
            path: path.into(),
            message: "unsupported graphics extension".into(),
        }),
    }
}

pub fn write_png(path: &Path, raster: &RgbImage) -> Result<(), ProjectError> {
    image::save_buffer_with_format(
        path,
        &raster.pixels,
        raster.width,
        raster.height,
        image::ColorType::Rgb8,
        image::ImageFormat::Png,
    )
    .map_err(|e| ProjectError::Format {
        path: path.into(),
        message: e.to_string(),
    })
}
