//! Composition follows master Config::tileset_{before,after}_path and
//! Tileset::{read_graphics,read_roof_graphics}; immutable source in upstream.lock.toml.
use std::path::{Path, PathBuf};

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

pub(crate) fn unified_graphics_path(directory: &Path, name: &str) -> Result<PathBuf, ProjectError> {
    validate_tileset(directory, name)?;
    let candidates = ["png", "2bpp", "CHR", "2bpp.lz"]
        .map(|extension| directory.join(format!("{name}.{extension}")));
    for path in &candidates {
        if crate::asset_paths::is_file(path)? {
            return Ok(path.clone());
        }
    }
    Ok(candidates[3].clone())
}

/// Resolve the selected asset, not merely the presence of a MAPDATA directory.
/// Preserve each model's preferred location, then permit the other source layout.
pub(crate) fn unified_graphics_directory(
    root: &Path,
    name: &str,
    format: crate::ProjectFormat,
) -> Result<PathBuf, ProjectError> {
    let directories = match format {
        crate::ProjectFormat::Master => [root.join("MAPDATA"), root.join("gfx/tilesets")],
        crate::ProjectFormat::PlusPlus => [root.join("gfx/tilesets"), root.join("MAPDATA")],
    };
    for directory in &directories {
        if crate::asset_paths::is_file(&unified_graphics_path(directory, name)?)? {
            return Ok(directory.clone());
        }
    }
    Ok(format.graphics_directory(root))
}

pub(crate) fn palette_map_path(root: &Path, tileset: &str) -> Result<PathBuf, ProjectError> {
    crate::asset_paths::AssetKind::PaletteMap.path(root, tileset)
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

pub fn read_tiles(
    path: &Path,
    png_platform: crate::PngPlatform,
) -> Result<Vec<Tile>, ProjectError> {
    read_tiles_for_storage(path, png_platform, crate::ProjectFormat::Master, true)
}

/// Unified graphics syntax with the detected model's physical tile capacity.
/// Accepts master's CHR import without reducing attributed data to 256 tiles.
pub fn read_unified_tiles(
    path: &Path,
    png_platform: crate::PngPlatform,
    format: crate::ProjectFormat,
) -> Result<Vec<Tile>, ProjectError> {
    read_tiles_for_storage(path, png_platform, format, true)
}

pub(crate) fn read_tiles_for_storage(
    path: &Path,
    png_platform: crate::PngPlatform,
    format: crate::ProjectFormat,
    allow_asm: bool,
) -> Result<Vec<Tile>, ProjectError> {
    let bytes = read_bounded(path, 8 * 1024 * 1024)?;
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let decode = match format {
        crate::ProjectFormat::Master => codecs::decode_2bpp,
        crate::ProjectFormat::PlusPlus => codecs::decode_plusplus_2bpp,
    };
    match extension.as_str() {
        "2bpp" => decode(&bytes).map_err(|e| ProjectError::codec(path, e)),
        "lz" => match format {
            crate::ProjectFormat::Master => codecs::decode_lz_tiles(&bytes),
            crate::ProjectFormat::PlusPlus => codecs::decode_plusplus_lz_tiles(&bytes),
        }
        .map_err(|e| ProjectError::codec(path, e)),
        "chr" if allow_asm => decode(
            &codecs::parse_asm(&text(path, &bytes)?).map_err(|e| ProjectError::codec(path, e))?,
        )
        .map_err(|e| ProjectError::codec(path, e)),
        "png" => match format {
            crate::ProjectFormat::Master => {
                crate::png_import::read_png_tiles(path, &bytes, png_platform)
            }
            crate::ProjectFormat::PlusPlus => {
                crate::png_import::read_png_tiles_bounded(path, &bytes, png_platform, 512)
            }
        },
        _ => Err(ProjectError::Format {
            path: path.into(),
            message: if allow_asm {
                "unsupported graphics extension"
            } else {
                "plusplus graphics require PNG, 2bpp or LZ"
            }
            .into(),
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
    write_observed_png(path, None, raster)
}

/// Replace an observed screenshot only after the caller has obtained consent.
/// Recheck its content before and after encoding. Existing-file replacement
/// still has the same narrow check/rename race as other document saves.
pub fn write_png_export(
    target: &crate::PngExportTarget,
    raster: &RgbImage,
) -> Result<(), ProjectError> {
    write_observed_png(target.path(), target.observed.hash, raster)
}

fn write_observed_png(
    path: &Path,
    expected_hash: Option<[u8; 32]>,
    raster: &RgbImage,
) -> Result<(), ProjectError> {
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
    if crate::png_export::png_hash(path)? != expected_hash {
        return Err(ProjectError::Conflict(path.into()));
    }
    let mut staged =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| ProjectError::io(path, e))?;
    if expected_hash.is_some() {
        staged
            .as_file()
            .set_permissions(fs_permissions(path)?)
            .map_err(|e| ProjectError::io(path, e))?;
    }
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
    if crate::png_export::png_hash(path)? != expected_hash {
        return Err(ProjectError::Conflict(path.into()));
    }
    let result = if expected_hash.is_some() {
        staged.persist(path)
    } else {
        staged.persist_noclobber(path)
    };
    result.map_err(|e| {
        if e.error.kind() == std::io::ErrorKind::AlreadyExists {
            ProjectError::Conflict(path.into())
        } else {
            ProjectError::io(path, e.error)
        }
    })?;
    Ok(())
}

fn fs_permissions(path: &Path) -> Result<std::fs::Permissions, ProjectError> {
    std::fs::metadata(path)
        .map(|m| m.permissions())
        .map_err(|e| ProjectError::io(path, e))
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
