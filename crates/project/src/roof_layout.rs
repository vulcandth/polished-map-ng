//! Roof-table context for unified Open/New. The pinned colors.cpp readers use
//! two colors per palette set and include unused group zero. LGPL-3.0-only.
use std::path::Path;

use crate::{ProjectError, RoofPaletteLayout, parse_rgb_colors, read_bounded, roof_palette_path};

/// Infer a roof layout only when the table and map-group count agree.
/// Three sets are ambiguous without a recognized palette-order comment.
/// Missing or unsupported metadata returns `None`; malformed RGB reports an
/// error. This inspection never changes files or selects a fallback layout.
pub fn infer_roof_palette_layout(root: &Path) -> Result<Option<RoofPaletteLayout>, ProjectError> {
    let path = roof_palette_path(root);
    if !path.is_file() {
        return Ok(None);
    }
    let read = |path: &Path| -> Result<String, ProjectError> {
        String::from_utf8(read_bounded(path, 4 * 1024 * 1024)?).map_err(|e| ProjectError::Format {
            path: path.into(),
            message: e.to_string(),
        })
    };
    let Some(constants) = [
        "constants/map_dimension_constants.asm",
        "constants/map_constants.asm",
        "constants/map_constants.inc",
    ]
    .iter()
    .map(|p| root.join(p))
    .find(|p| p.is_file()) else {
        return Ok(None);
    };
    let constants = read(&constants)?;
    let groups = constants
        .lines()
        .filter(|line| {
            line.starts_with([' ', '\t'])
                && line
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    == Some("newgroup")
        })
        .count();
    if groups == 0 || groups > 255 {
        return Ok(None);
    }
    let table = read(&path)?;
    let colors =
        parse_rgb_colors(&table).map_err(|message| ProjectError::Format { path, message })?;
    let per_set = (groups + 1) * 2;
    if colors.len() % per_set != 0 {
        return Ok(None);
    }
    Ok(match colors.len() / per_set {
        1 => Some(RoofPaletteLayout::Custom),
        2 => Some(RoofPaletteLayout::DayNite),
        3 => three_sets(&table),
        4 => Some(RoofPaletteLayout::MornDayNiteCustom),
        _ => None,
    })
}

fn three_sets(table: &str) -> Option<RoofPaletteLayout> {
    // Recognize ordering, not the project name. Both three-set layouts have
    // exactly the same color count. Conflicting comments stay ambiguous.
    let mut found = None;
    for comment in table.lines().filter_map(|l| l.trim().strip_prefix(';')) {
        let words: Vec<_> = comment.trim().split(',').map(str::trim).collect();
        let layout = match words.as_slice() {
            ["morn/day", "nite", "eve" | "custom"] => RoofPaletteLayout::DayNiteCustom,
            ["morn", "day", "nite"] => RoofPaletteLayout::MornDayNite,
            _ => continue,
        };
        if found.is_some_and(|old| old != layout) {
            return None;
        }
        found = Some(layout);
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    #[ignore = "requires pinned POLISHED_MAP_POKECRYSTAL and POLISHED_CRYSTAL_FIXTURE"]
    fn pinned_roof_tables_have_distinct_detected_layouts() {
        for (variable, expected) in [
            ("POLISHED_MAP_POKECRYSTAL", RoofPaletteLayout::DayNite),
            ("POLISHED_CRYSTAL_FIXTURE", RoofPaletteLayout::DayNiteCustom),
        ] {
            let root = std::env::var_os(variable).expect(variable);
            assert_eq!(
                infer_roof_palette_layout(Path::new(&root)).unwrap(),
                Some(expected),
                "{variable}"
            );
        }
    }

    #[test]
    fn counts_include_group_zero_and_ambiguous_tables_are_not_guessed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("constants")).unwrap();
        fs::create_dir_all(root.join("tilesets")).unwrap();
        fs::write(
            root.join("constants/map_constants.asm"),
            "\tnewgroup A\n\tnewgroup B\n",
        )
        .unwrap();
        for (sets, header, expected) in [
            (1, "", Some(RoofPaletteLayout::Custom)),
            (2, "", Some(RoofPaletteLayout::DayNite)),
            (3, "", None),
            (
                3,
                "; morn, day, nite\n",
                Some(RoofPaletteLayout::MornDayNite),
            ),
            (
                3,
                "; morn/day, nite, eve\n",
                Some(RoofPaletteLayout::DayNiteCustom),
            ),
            (4, "", Some(RoofPaletteLayout::MornDayNiteCustom)),
            (5, "", None),
        ] {
            fs::write(
                root.join("tilesets/roof.pal"),
                format!("{header}{}", "\tRGB 1,2,3, 4,5,6\n".repeat(3 * sets)),
            )
            .unwrap();
            assert_eq!(infer_roof_palette_layout(root).unwrap(), expected);
        }
        fs::write(root.join("tilesets/roof.pal"), "\tRGB 32,0,0\n").unwrap();
        assert!(infer_roof_palette_layout(root).is_err());
        fs::write(root.join("tilesets/roof.pal"), "\tRGB 1,2,3\n").unwrap();
        assert_eq!(infer_roof_palette_layout(root).unwrap(), None);
    }
}
