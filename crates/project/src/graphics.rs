//! Composition follows master Config::tileset_{before,after}_path and
//! Tileset::{read_graphics,read_roof_graphics}; immutable source in upstream.lock.toml.
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
        options.roof.as_deref(),
    )
}

/// Load raw master tilesets with upstream DAY defaults, without map metadata.
pub fn load_master_graphics(
    root: &Path,
    tileset: &str,
    allow_256: bool,
) -> Result<Graphics, ProjectError> {
    load_graphics(root, tileset, allow_256, &day_palettes(), false, None)
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
    roof: Option<&str>,
) -> Result<Graphics, ProjectError> {
    validate_tileset(root, tileset)?;
    let gfx = if root.join("MAPDATA").is_dir() {
        root.join("MAPDATA")
    } else {
        root.join("gfx/tilesets")
    };
    let main_path = graphics_path(&gfx, tileset);
    let tiles = read_tiles(&main_path)?;
    // The main file keeps the full compound name. Missing companions are ignored,
    // as Config returns false for them; present but malformed files are errors.
    let (before_name, after_name) = tileset
        .split_once('.')
        .map(|(_, suffix)| suffix.split_once(',').unwrap_or((suffix, "")))
        .unwrap_or(("", ""));
    let before = read_companion(&gfx, before_name)?;
    let after = read_companion(&gfx, after_name)?;
    let before_end = before.len().div_ceil(16) * 16;
    let middle_end = before_end
        + if after.is_empty() {
            tiles.len()
        } else {
            tiles.len().div_ceil(16) * 16
        };
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
    let mut mapped_tiles = vec![blank.clone(); 256];
    // Padding gaps retain Tile::clear's white pixels; trailing slots are read
    // from the (possibly absent) after image, whose out-of-range hue is WHITE.
    let mut palettes = vec![[[255; 3]; 4]; 256];
    let palette_for = |i: usize| {
        ids.get(i).map_or(undefined, |id| {
            colors[if monochrome { 8 } else { usize::from(id & 15) }]
        })
    };
    for i in 0..256 {
        let source = if i < before.len() {
            Some(&before[i])
        } else if i >= before_end && i < before_end + tiles.len() {
            Some(&tiles[i - before_end])
        } else if i >= middle_end {
            Some(after.get(i - middle_end).unwrap_or(&blank))
        } else {
            None
        };
        if let Some(tile) = source {
            let j = tile_slot(i, allow_256);
            mapped_tiles[j] = tile.clone();
            palettes[j] = palette_for(i);
        }
    }
    if let Some(roof) = roof {
        validate_tileset(root, roof)?;
        let path = graphics_path(&root.join("gfx/tilesets/roofs"), roof);
        let roof_tiles = read_tiles(&path)?;
        if roof_tiles.len() != 9 {
            return Err(ProjectError::Format {
                path,
                message: "roof graphics must contain exactly nine tiles".into(),
            });
        }
        for (i, tile) in roof_tiles.into_iter().enumerate() {
            let slot = 0x0a + i;
            mapped_tiles[slot] = tile;
            palettes[slot] = palette_for(slot);
        }
    }
    Ok(Graphics::new(mapped_tiles, palettes, metatiles)?)
}

fn tile_slot(i: usize, allow_256: bool) -> usize {
    if !allow_256 && i >= 0x60 {
        if i >= 0xe0 { i - 0x80 } else { i + 0x20 }
    } else {
        i
    }
}

fn graphics_path(directory: &Path, name: &str) -> PathBuf {
    first_existing(
        &["png", "2bpp", "CHR", "2bpp.lz"].map(|ext| directory.join(format!("{name}.{ext}"))),
    )
}

fn read_companion(directory: &Path, name: &str) -> Result<Vec<Tile>, ProjectError> {
    if name.is_empty() {
        return Ok(Vec::new());
    }
    validate_tileset(directory, name)?;
    let path = graphics_path(directory, name);
    if path.is_file() {
        read_tiles(&path)
    } else {
        Ok(Vec::new())
    }
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
