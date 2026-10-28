//! Master project palette-file precedence and metadata selection from config.cpp,
//! colors.cpp and Main_Window::open_map. Palette editing/writing is separate work.
use std::path::{Path, PathBuf};

use polished_map_core::{Palette, PaletteBank, PaletteSet};

use crate::{MapDocument, ProjectError, read_bounded};

#[derive(Clone, Copy, Debug, Default)]
pub enum RoofPaletteLayout {
    Custom,
    #[default]
    DayNite,
    MornDayNite,
    DayNiteCustom,
    MornDayNiteCustom,
}

#[derive(Clone, Debug)]
pub struct MasterGraphicsOptions {
    pub allow_256: bool,
    pub palette: Option<PaletteSet>,
    pub dark_theme: bool,
    pub monochrome: bool,
    pub special_palettes: bool,
    pub roof_colors: bool,
    pub roof_layout: RoofPaletteLayout,
}

impl Default for MasterGraphicsOptions {
    fn default() -> Self {
        Self {
            allow_256: false,
            palette: None,
            dark_theme: false,
            monochrome: false,
            special_palettes: true,
            roof_colors: true,
            roof_layout: RoofPaletteLayout::DayNite,
        }
    }
}

/// RGB macros store WHITE,LIGHT,DARK,BLACK. Return expanded colors in that order.
/// Reject partial triples and out-of-range five-bit channels without guessing.
pub fn parse_rgb_colors(text: &str) -> Result<Vec<[u8; 3]>, String> {
    let mut values = Vec::new();
    for (line_number, line) in text.lines().enumerate() {
        if !line.starts_with([' ', '\t']) {
            continue;
        }
        let line = line.split(';').next().unwrap_or("").trim();
        let Some(rest) = line
            .strip_prefix("RGB")
            .filter(|s| s.starts_with(char::is_whitespace))
        else {
            continue;
        };
        for token in rest.trim().split(',') {
            let value = token
                .trim()
                .parse::<u8>()
                .ok()
                .filter(|&v| v <= 31)
                .ok_or_else(|| {
                    format!(
                        "invalid five-bit RGB channel on line {}: {}",
                        line_number + 1,
                        token.trim()
                    )
                })?;
            values.push((u16::from(value) * 33 / 4) as u8);
        }
    }
    let (colors, trailing) = values.as_chunks::<3>();
    if !trailing.is_empty() {
        return Err("incomplete RGB triple".into());
    }
    Ok(colors.to_vec())
}

pub fn parse_rgb_palettes(text: &str) -> Result<Vec<Palette>, String> {
    let colors = parse_rgb_colors(text)?;
    let (palettes, trailing) = colors.as_chunks::<4>();
    if !trailing.is_empty() {
        return Err("incomplete four-color palette".into());
    }
    Ok(palettes.iter().map(|p| [p[0], p[2], p[1], p[3]]).collect())
}

fn text(path: &Path) -> Result<String, ProjectError> {
    String::from_utf8(read_bounded(path, 4 * 1024 * 1024)?).map_err(|e| ProjectError::Format {
        path: path.into(),
        message: e.to_string(),
    })
}

fn apply_file(
    bank: &mut PaletteBank,
    path: &Path,
    selected: PaletteSet,
) -> Result<PaletteSet, ProjectError> {
    let palettes = parse_rgb_palettes(&text(path)?).map_err(|message| ProjectError::Format {
        path: path.into(),
        message,
    })?;
    Ok(bank.apply(&palettes, selected))
}

pub(crate) fn map_palettes(
    doc: &MapDocument,
    tileset: &str,
    options: &MasterGraphicsOptions,
) -> Result<[Palette; 9], ProjectError> {
    use PaletteSet::*;
    let metadata = doc.metadata();
    let root = metadata
        .root
        .as_deref()
        .ok_or_else(|| ProjectError::Format {
            path: doc.path().into(),
            message: "project root is required for graphics".into(),
        })?;
    let mut bank = PaletteBank::default();
    let mut selected = Day;
    if let Some(path) = [
        root.join("gfx/tilesets/bg_tiles.pal"),
        root.join("tilesets/bg.pal"),
    ]
    .into_iter()
    .find(|p| p.is_file())
    {
        selected = apply_file(&mut bank, &path, selected)?;
    }
    if options.special_palettes {
        let gfx = if root.join("MAPDATA").is_dir() {
            root.join("MAPDATA")
        } else {
            root.join("gfx/tilesets")
        };
        let base = tileset.split('.').next().unwrap_or(tileset);
        let mut paths = Vec::new();
        if doc.has_backing_file() {
            paths.push(doc.path().with_extension("pal"));
        }
        if !metadata.landmark.is_empty() && !metadata.landmark.contains(['/', '\\']) {
            paths.push(gfx.join(format!("{}.pal", metadata.landmark)));
        }
        paths.extend([
            gfx.join(format!("{tileset}.pal")),
            gfx.join(format!("{base}.pal")),
        ]);
        if let Some(path) = paths.into_iter().find(|p| p.is_file()) {
            selected = apply_file(&mut bank, &path, selected)?;
        }
    }
    if selected != Custom {
        selected = if matches!(
            metadata.palette.as_str(),
            "PALETTE_NITE" | "PALETTE_DARK" | "2" | "4"
        ) {
            Nite
        } else if matches!(metadata.environment.as_str(), "INDOOR" | "GATE" | "3" | "6") {
            Indoor
        } else if matches!(metadata.palette.as_str(), "PALETTE_MORN" | "3") {
            Morn
        } else if options.dark_theme && !options.monochrome {
            Nite
        } else {
            Day
        };
    }
    if options.roof_colors
        && metadata.group != 0
        && !options.monochrome
        && matches!(metadata.environment.as_str(), "TOWN" | "ROUTE" | "1" | "2")
        && let Some(path) = [
            root.join("gfx/tilesets/roofs.pal"),
            root.join("tilesets/roof.pal"),
        ]
        .into_iter()
        .find(|p| p.is_file())
    {
        apply_roofs(&mut bank, &path, metadata.group, options.roof_layout)?;
    }
    Ok(*bank.colors(options.palette.unwrap_or(selected)))
}

fn apply_roofs(
    bank: &mut PaletteBank,
    path: &PathBuf,
    group: usize,
    layout: RoofPaletteLayout,
) -> Result<(), ProjectError> {
    use PaletteSet::*;
    let sets: &[PaletteSet] = match layout {
        RoofPaletteLayout::Custom => &[Custom],
        RoofPaletteLayout::DayNite => &[Day, Nite],
        RoofPaletteLayout::MornDayNite => &[Morn, Day, Nite],
        RoofPaletteLayout::DayNiteCustom => &[Day, Nite, Custom],
        RoofPaletteLayout::MornDayNiteCustom => &[Morn, Day, Nite, Custom],
    };
    let colors = parse_rgb_colors(&text(path)?).map_err(|message| ProjectError::Format {
        path: path.into(),
        message,
    })?;
    let start = group
        .checked_mul(sets.len() * 2)
        .ok_or_else(|| ProjectError::Format {
            path: path.into(),
            message: "roof palette index overflow".into(),
        })?;
    let colors = colors
        .get(start..)
        .and_then(|s| s.get(..sets.len() * 2))
        .ok_or_else(|| ProjectError::Format {
            path: path.into(),
            message: format!("no roof colors for map group {group}"),
        })?;
    for (i, &set) in sets.iter().enumerate() {
        bank.set_roof(set, colors[2 * i], colors[2 * i + 1]);
    }
    if matches!(
        layout,
        RoofPaletteLayout::DayNite | RoofPaletteLayout::DayNiteCustom
    ) {
        bank.set_roof(Morn, colors[0], colors[1]);
    }
    Ok(())
}
