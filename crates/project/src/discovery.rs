//! Master discovery rules from config.cpp and option-dialogs.cpp at upstream.lock.toml.
//! This reads recognized metadata macros; it does not evaluate arbitrary RGBDS assembly.
use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::{ProjectError, read_bounded};

#[derive(Clone, Debug, Default, Serialize)]
pub struct MapMetadata {
    pub root: Option<PathBuf>,
    pub name: String,
    pub constant: String,
    pub dimensions: Option<(u16, u16)>,
    pub dimensions_source: Option<PathBuf>,
    pub group: usize,
    pub tileset_hint: Option<String>,
    pub tileset: Option<String>,
    pub available_tilesets: Vec<String>,
    pub header_source: Option<PathBuf>,
    pub environment: String,
    pub landmark: String,
    pub palette: String,
}

/// Locate the nearest ancestor containing Makefile or makefile, as upstream does.
pub fn find_project_root(map: &Path) -> Option<PathBuf> {
    let absolute = if map.is_absolute() {
        map.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(map)
    };
    absolute
        .ancestors()
        .skip(1)
        .find(|p| p.join("Makefile").is_file() || p.join("makefile").is_file())
        .map(Path::to_path_buf)
}

/// Inspect metadata without loading graphics or writing any project files.
/// An explicit root overrides the Makefile search. Missing metadata is optional;
/// unreadable or oversized files return an error rather than a guessed value.
pub fn discover_map(map: &Path, root: Option<&Path>) -> Result<MapMetadata, ProjectError> {
    let filename = map.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let name = filename.split('.').next().unwrap_or("").to_owned();
    let mut metadata = MapMetadata {
        root: root
            .map(Path::to_path_buf)
            .or_else(|| find_project_root(map)),
        constant: map_constant(&name),
        name,
        dimensions: filename_dimensions(map),
        tileset_hint: filename_tileset(map),
        ..Default::default()
    };
    if metadata.dimensions.is_some() {
        metadata.dimensions_source = Some(map.into());
    }
    let Some(root) = metadata.root.clone() else {
        return Ok(metadata);
    };
    if metadata.dimensions.is_none()
        && let Some(path) = first_file(
            &root,
            &[
                "constants/map_dimension_constants.asm",
                "constants/map_constants.asm",
                "constants/map_constants.inc",
            ],
        )
    {
        read_dimensions(&path, &mut metadata)?;
    }
    // Filename hints short-circuit header attributes in upstream too.
    if metadata.tileset_hint.is_none() {
        if let Some(path) = first_file(&root, &["data/maps/maps.asm", "maps/map_headers.asm"]) {
            read_headers(&path, &mut metadata, false)?;
        } else if let Some(path) = first_file(
            &root,
            &[
                &format!("data/maps/headers/{}.asm", metadata.name),
                &format!("data/mapHeaders/{}.asm", metadata.name),
            ],
        ) {
            read_headers(&path, &mut metadata, true)?;
        }
    }
    metadata.available_tilesets = list_tilesets(&root)?;
    let aliases = tileset_aliases(&root)?;
    for name in &metadata.available_tilesets {
        let base = name.split('.').next().unwrap_or(name);
        let alias = aliases
            .iter()
            .find(|(id, _)| id == base)
            .map(|(_, value)| value.as_str());
        if metadata
            .tileset_hint
            .as_deref()
            .is_some_and(|hint| hint == base || Some(hint) == alias)
        {
            metadata.tileset = Some(name.clone());
        }
    }
    Ok(metadata)
}

pub(crate) fn filename_dimensions(map: &Path) -> Option<(u16, u16)> {
    let name = map.file_name()?.to_str()?;
    let parts: Vec<_> = name.split('.').collect();
    if parts.len() < 3 || !parts.last()?.eq_ignore_ascii_case("blk") {
        return None;
    }
    // The original regex permits either Name.WxH.blk or Name.WxH.tileset.blk.
    for index in [parts.len() - 2, parts.len().saturating_sub(3)] {
        if index == 0 {
            continue;
        }
        if index + 2 < parts.len() && !valid_suffix(parts[index + 1]) {
            continue;
        }
        if let Some((w, h)) = parts[index].split_once('x')
            && !w.is_empty()
            && !h.is_empty()
            && w.bytes().all(|b| b.is_ascii_digit())
            && h.bytes().all(|b| b.is_ascii_digit())
            && let (Ok(w), Ok(h)) = (w.parse(), h.parse())
        {
            return Some((w, h));
        }
    }
    None
}

fn filename_tileset(map: &Path) -> Option<String> {
    let parts: Vec<_> = map.file_name()?.to_str()?.split('.').collect();
    if parts.len() < 3 || !parts.last()?.eq_ignore_ascii_case("blk") {
        return None;
    }
    let hint = parts[parts.len() - 2];
    valid_suffix(hint).then(|| hint.to_owned())
}

fn valid_suffix(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// Match upstream's acronym/digit boundaries, including its uppercased first character.
pub fn map_constant(name: &str) -> String {
    let name = name.split('.').next().unwrap_or("");
    let chars: Vec<_> = name.chars().collect();
    let Some(&first) = chars.first() else {
        return String::new();
    };
    let convert = |c: char| {
        if c.is_ascii_alphanumeric() {
            c.to_ascii_uppercase()
        } else {
            '_'
        }
    };
    let mut output = first.to_ascii_uppercase().to_string();
    let mut prev = convert(first);
    for (i, &c) in chars.iter().enumerate().skip(1) {
        if (prev.is_ascii_lowercase() && (c.is_ascii_uppercase() || c.is_ascii_digit()))
            || ((prev.is_ascii_uppercase() || prev.is_ascii_digit())
                && c.is_ascii_uppercase()
                && chars.get(i + 1).is_some_and(char::is_ascii_lowercase))
        {
            output.push('_');
        }
        output.push(convert(c));
        prev = c;
    }
    output
}

fn first_file(root: &Path, choices: &[&str]) -> Option<PathBuf> {
    choices.iter().map(|p| root.join(p)).find(|p| p.is_file())
}

fn read_text(path: &Path) -> Result<String, ProjectError> {
    String::from_utf8(read_bounded(path, 4 * 1024 * 1024)?).map_err(|e| ProjectError::Format {
        path: path.into(),
        message: e.to_string(),
    })
}

fn macro_args(line: &str, indented: bool) -> Option<(&str, Vec<&str>)> {
    if indented && !line.starts_with([' ', '\t']) {
        return None;
    }
    let line = line.split(';').next()?.trim();
    let end = line.find(char::is_whitespace).unwrap_or(line.len());
    let (name, rest) = line.split_at(end);
    Some((name, rest.trim().split(',').map(str::trim).collect()))
}

fn read_dimensions(path: &Path, metadata: &mut MapMetadata) -> Result<(), ProjectError> {
    for line in read_text(path)?.lines() {
        let Some((name, args)) = macro_args(line, true) else {
            continue;
        };
        if name == "newgroup" {
            metadata.group += 1;
            continue;
        }
        if !matches!(name, "map_const" | "mapgroup" | "mapconst")
            || args.first() != Some(&metadata.constant.as_str())
        {
            continue;
        }
        let parse = |i: usize| args.get(i).and_then(|s| s.parse::<u16>().ok());
        if let (Some(a), Some(b)) = (parse(1), parse(2))
            && (1..=255).contains(&a)
            && (1..=255).contains(&b)
        {
            metadata.dimensions = Some(if name == "map_const" { (a, b) } else { (b, a) });
            metadata.dimensions_source = Some(path.into());
        }
        break;
    }
    Ok(())
}

fn read_headers(
    path: &Path,
    metadata: &mut MapMetadata,
    individual: bool,
) -> Result<(), ProjectError> {
    for line in read_text(path)?.lines() {
        let Some((name, args)) = macro_args(line, false) else {
            continue;
        };
        if individual {
            let hint = match name {
                "map_header" => args.get(2).copied(),
                "db" => args.first().and_then(|s| s.split_whitespace().next()),
                _ => None,
            };
            if let Some(hint) = hint {
                metadata.tileset_hint = Some(hint.to_ascii_lowercase());
                metadata.header_source = Some(path.into());
                break;
            }
        } else if matches!(name, "map_header" | "map") {
            let Some(label) = args.first() else {
                continue;
            };
            let same = if cfg!(windows) {
                label.eq_ignore_ascii_case(&metadata.name)
            } else {
                *label == metadata.name
            };
            if !same {
                continue;
            }
            metadata.tileset_hint = args.get(1).and_then(|value| {
                if let Some(name) = value.strip_prefix("TILESET_") {
                    Some(name.to_ascii_lowercase())
                } else if let Some(hex) = value.strip_prefix('$') {
                    u32::from_str_radix(hex, 16).ok().map(|n| format!("{n:02}"))
                } else if !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) {
                    Some(format!("{value:0>2}"))
                } else {
                    None
                }
            });
            metadata.environment = args.get(2).unwrap_or(&"").to_string();
            metadata.landmark = args.get(3).unwrap_or(&"").to_ascii_lowercase();
            metadata.palette = args.get(6).unwrap_or(&"").to_string();
            metadata.header_source = Some(path.into());
            break;
        }
    }
    Ok(())
}

/// Graphics names in stable order, excluding generated colored previews.
pub fn list_tilesets(root: &Path) -> Result<Vec<String>, ProjectError> {
    let dir = if root.join("MAPDATA").is_dir() {
        root.join("MAPDATA")
    } else {
        root.join("gfx/tilesets")
    };
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| ProjectError::io(&dir, e))? {
        let entry = entry.map_err(|e| ProjectError::io(&dir, e))?;
        if !entry
            .file_type()
            .map_err(|e| ProjectError::io(&entry.path(), e))?
            .is_file()
        {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".colored.png") {
            continue;
        }
        if let Some(ext) = [".2bpp.lz", ".2bpp.unique.lz", ".2bpp", ".chr", ".png"]
            .iter()
            .find(|ext| lower.ends_with(**ext))
        {
            names.push(name[..name.len() - ext.len()].to_owned());
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}

fn tileset_aliases(root: &Path) -> Result<Vec<(String, String)>, ProjectError> {
    let Some(path) = first_file(
        root,
        &[
            "constants/tileset_constants.asm",
            "constants/tileset_constants.inc",
            "constants/tilemap_constants.asm",
        ],
    ) else {
        return Ok(Vec::new());
    };
    let mut names = Vec::new();
    for line in read_text(&path)?.lines() {
        if let Some(("const", args)) = macro_args(line, true)
            && let Some(token) = args.first().and_then(|s| s.split_whitespace().next())
        {
            names.push((
                format!("{:02}", names.len() + 1),
                token
                    .strip_prefix("TILESET_")
                    .unwrap_or(token)
                    .to_ascii_lowercase(),
            ));
        }
    }
    Ok(names)
}
