//! Composition follows master Config::tileset_{before,after}_path and
//! Tileset::{read_graphics,read_roof_graphics}; immutable source in upstream.lock.toml.
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use polished_map_core::{Graphics, RgbImage, Tile};

use crate::{MapDocument, MasterGraphicsOptions, ProjectError, codecs, read_bounded};

/// Load graphics with per-project palettes, map metadata and roof colors.
pub fn load_master_map_graphics(
    doc: &MapDocument,
    tileset: &str,
    options: &MasterGraphicsOptions,
) -> Result<Graphics, ProjectError> {
    let source = crate::MasterTilesetDocument::open_for_map(doc, tileset, options)?;
    let root = doc
        .metadata()
        .root
        .as_deref()
        .ok_or_else(|| ProjectError::Format {
            path: doc.path().into(),
            message: "project root is required for graphics".into(),
        })?;
    let blocks = crate::BlocksetDocument::open_in_project(root, tileset)?;
    source.graphics(blocks.blockset())
}

/// Load raw master tilesets with upstream DAY defaults, without map metadata.
pub fn load_master_graphics(
    root: &Path,
    tileset: &str,
    allow_256: bool,
) -> Result<Graphics, ProjectError> {
    let source = crate::MasterTilesetDocument::open_in_project(
        root,
        tileset,
        &MasterGraphicsOptions {
            allow_256,
            ..Default::default()
        },
    )?;
    let blocks = crate::BlocksetDocument::open_in_project(root, tileset)?;
    source.graphics(blocks.blockset())
}

pub(crate) fn validate_tileset(root: &Path, tileset: &str) -> Result<(), ProjectError> {
    if tileset.is_empty()
        || tileset.contains(['/', '\\', ':', '\0'])
        || matches!(tileset, "." | "..")
    {
        return Err(ProjectError::Format {
            path: root.into(),
            message: "tileset must be a project-local name".into(),
        });
    }
    Ok(())
}

pub(crate) fn tile_slot(i: usize, allow_256: bool) -> usize {
    if !allow_256 && i >= 0x60 {
        if i >= 0xe0 { i - 0x80 } else { i + 0x20 }
    } else {
        i
    }
}

pub(crate) fn graphics_path(directory: &Path, name: &str) -> PathBuf {
    first_existing(
        &["png", "2bpp", "CHR", "2bpp.lz"].map(|ext| directory.join(format!("{name}.{ext}"))),
    )
}

pub(crate) fn palette_map_path(root: &Path, tileset: &str) -> PathBuf {
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

/// Stage a complete PNG and install it only if the destination is still absent.
/// This does not authorize replacing an existing file. A failed encode never
/// leaves a partial output; a concurrent creator is reported as a conflict.
pub fn write_new_png(path: &Path, raster: &RgbImage) -> Result<(), ProjectError> {
    use image::ImageEncoder;
    let expected = usize::try_from(raster.width)
        .ok()
        .and_then(|w| {
            usize::try_from(raster.height)
                .ok()
                .and_then(|h| w.checked_mul(h))
        })
        .and_then(|pixels| pixels.checked_mul(3));
    if raster.width == 0
        || raster.height == 0
        || raster.width > 8160
        || raster.height > 8160
        || expected != Some(raster.pixels.len())
    {
        return Err(ProjectError::Format {
            path: path.into(),
            message: "invalid RGB image dimensions or byte count".into(),
        });
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut staged =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| ProjectError::io(path, e))?;
    image::codecs::png::PngEncoder::new(&mut staged)
        .write_image(
            &raster.pixels,
            raster.width,
            raster.height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| ProjectError::Format {
            path: path.into(),
            message: e.to_string(),
        })?;
    staged
        .as_file()
        .sync_all()
        .map_err(|e| ProjectError::io(path, e))?;
    staged.persist_noclobber(path).map_err(|e| {
        if e.error.kind() == std::io::ErrorKind::AlreadyExists {
            ProjectError::Conflict(path.into())
        } else {
            ProjectError::io(path, e.error)
        }
    })?;
    Ok(())
}

/// Same two-bit grayscale format as master Image::write_image.
pub(crate) fn encode_grayscale_png(
    path: &Path,
    raster: &RgbImage,
) -> Result<Vec<u8>, ProjectError> {
    // Image::write_image packs four grayscale samples, most significant
    // first. Keep the original two-bit grayscale PNG format; compressed
    // bytes may differ between libpng/zlib and the Rust encoder.
    let packed: Vec<u8> = raster
        .pixels
        .as_chunks::<12>()
        .0
        .iter()
        .map(|pixels| {
            pixels
                .as_chunks::<3>()
                .0
                .iter()
                .fold(0, |byte, pixel| (byte << 2) | (pixel[0] / 64))
        })
        .collect();
    let mut bytes = Vec::new();
    let encoded = (|| -> Result<(), png::EncodingError> {
        let mut encoder = png::Encoder::new(&mut bytes, raster.width, raster.height);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Two);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&packed)?;
        writer.finish()
    })();
    encoded.map_err(|e| ProjectError::Format {
        path: path.into(),
        message: e.to_string(),
    })?;
    Ok(bytes)
}
