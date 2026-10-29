//! File-list ordering from FLTK 1.4.5 numericsort.c, used by master selectors.
//! FLTK copyright/license are retained in NOTICE.md and LICENSES/FLTK.txt.
use std::{cmp::Ordering, fs, path::Path};

use crate::ProjectError;

/// Case-sensitive UTF-8 branch of FLTK's numeric comparison. The characters
/// consumed just after numeric runs are intentionally not compared again.
pub(crate) fn numeric_order(a: &str, b: &str) -> Ordering {
    let mut a = a.chars().peekable();
    let mut b = b.chars().peekable();
    loop {
        let mut ca = a.next().unwrap_or('\0');
        let mut cb = b.next().unwrap_or('\0');
        if ca.is_ascii_digit() && cb.is_ascii_digit() {
            while ca == '0' {
                ca = a.next().unwrap_or('\0');
            }
            while cb == '0' {
                cb = b.next().unwrap_or('\0');
            }
            while ca.is_ascii_digit() && ca == cb {
                ca = a.next().unwrap_or('\0');
                cb = b.next().unwrap_or('\0');
            }
            let difference = if ca.is_ascii_digit() && cb.is_ascii_digit() {
                ca.cmp(&cb)
            } else {
                Ordering::Equal
            };
            let mut a_digits = 0;
            let mut b_digits = 0;
            while ca.is_ascii_digit() {
                a_digits += 1;
                ca = a.next().unwrap_or('\0');
            }
            while cb.is_ascii_digit() {
                b_digits += 1;
                cb = b.next().unwrap_or('\0');
            }
            let order = a_digits.cmp(&b_digits).then(difference);
            if order != Ordering::Equal {
                return order;
            }
        } else {
            let order = ca.cmp(&cb);
            if order != Ordering::Equal || a.peek().is_none() {
                return order;
            }
        }
    }
}

pub(crate) fn graphics_stems(
    directory: &Path,
    extensions: &[&str],
    ignore_colored: bool,
) -> Result<Vec<String>, ProjectError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(ProjectError::io(directory, error)),
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| ProjectError::io(directory, e))?;
        if !entry
            .file_type()
            .map_err(|e| ProjectError::io(&entry.path(), e))?
            .is_file()
        {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_owned());
        }
    }
    // FLTK leaves equivalent numeric names tied. Resolve only those ties by
    // filename so the project API does not depend on directory enumeration order.
    names.sort_by(|a, b| numeric_order(a, b).then_with(|| a.cmp(b)));
    Ok(names
        .into_iter()
        .filter_map(|name| {
            let lower = name.to_ascii_lowercase();
            if ignore_colored && lower.ends_with(".colored.png") {
                return None;
            }
            extensions
                .iter()
                .find(|ext| lower.ends_with(**ext))
                .map(|ext| name[..name.len() - ext.len()].to_owned())
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires POLISHED_MAP_ASSET_SORT_ORACLE; run in reference CI"]
    fn reference_asset_numeric_order_matches_fltk() {
        let oracle = std::env::var_os("POLISHED_MAP_ASSET_SORT_ORACLE")
            .expect("set POLISHED_MAP_ASSET_SORT_ORACLE");
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("names.txt");
        let output = dir.path().join("comparisons.bin");
        let mut names = Vec::new();
        for prefix in ["", "map", "MAP", "猫", "é", "２"] {
            for number in [
                "0",
                "00",
                "1",
                "01",
                "2",
                "10",
                "100",
                "9999999999999999999999999",
            ] {
                for suffix in [".png", "a.2bpp", "b.2bpp", ".before,after.chr"] {
                    names.push(format!("{prefix}{number}{suffix}"));
                }
            }
        }
        names.extend(
            [
                "demo.png",
                "demo.2bpp",
                "demo.before,after.png",
                "é.png",
                "猫.png",
            ]
            .map(str::to_owned),
        );
        fs::write(&input, names.join("\n") + "\n").unwrap();
        assert!(
            std::process::Command::new(oracle)
                .args([&input, &output])
                .status()
                .unwrap()
                .success()
        );
        let bytes = fs::read(output).unwrap();
        assert_eq!(bytes.len(), names.len() * names.len());
        for (i, a) in names.iter().enumerate() {
            for (j, b) in names.iter().enumerate() {
                let expected = match bytes[i * names.len() + j] {
                    255 => Ordering::Less,
                    0 => Ordering::Equal,
                    1 => Ordering::Greater,
                    other => panic!("unexpected result {other}"),
                };
                assert_eq!(numeric_order(a, b), expected, "{a:?} vs {b:?}");
            }
        }
    }
}
