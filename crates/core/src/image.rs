//! Image::write_image counterparts: PNG encoding and guarded publication.
//! Adapted from pinned image.cpp; LGPL-3.0-only.
use crate::ProjectError;
use crate::RgbImage;
use std::path::Path;

/// Write RGB8 pixels directly to a PNG, replacing any existing destination.
///
/// Prefer [`write_new_png`] or [`write_png_export`] for guarded publication.
/// This compatibility helper validates the raster before opening the destination,
/// but an encoding/I/O failure can still leave partial output.
///
/// # Errors
/// Rejects zero dimensions, a mismatched RGB byte count, and encoding/I/O errors.
pub fn write_png(path: &Path, raster: &RgbImage) -> Result<(), ProjectError> {
    validate_rgb(path, raster)?;
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
    validate_rgb(path, raster)?;
    if raster.width > 8160 || raster.height > 8160 {
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

fn validate_rgb(path: &Path, raster: &RgbImage) -> Result<(), ProjectError> {
    let expected = usize::try_from(raster.width)
        .ok()
        .and_then(|w| {
            usize::try_from(raster.height)
                .ok()
                .and_then(|h| w.checked_mul(h))
        })
        .and_then(|pixels| pixels.checked_mul(3));
    if raster.width == 0 || raster.height == 0 || expected != Some(raster.pixels.len()) {
        return Err(ProjectError::Format {
            path: path.into(),
            message: "invalid RGB image dimensions or byte count".into(),
        });
    }
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
