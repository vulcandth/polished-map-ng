//! Preserve unrelated palette-map assembly when applying tile attribute changes.
use crate::{ProjectError, codecs};
use std::path::Path;

fn row(line: &str) -> Option<(&str, &str, &str)> {
    if !line.starts_with([' ', '\t']) {
        return None;
    }
    let trimmed = line.trim_start_matches([' ', '\t']);
    let rest = trimmed
        .strip_prefix("tilepal")?
        .strip_prefix(char::is_whitespace)?;
    let comma = rest.find(',')?;
    let values = &rest[comma + 1..];
    let start = line.len() - values.len();
    let code = values.split(';').next()?.trim_end();
    Some((&line[..start], code, &values[code.len()..]))
}

fn gap(lines: &[&str], i: usize) -> bool {
    i + 2 < lines.len()
        && matches!(lines[i].trim(), "rept 16" | "rept 32")
        && lines[i + 1].trim() == "db $ff"
        && lines[i + 2].trim() == "endr"
}

pub(crate) fn encode(
    path: &Path,
    original: &str,
    canonical: &str,
) -> Result<Vec<u8>, ProjectError> {
    let old = codecs::parse_palette_map(original).map_err(|e| ProjectError::codec(path, e))?;
    let new = codecs::parse_palette_map(canonical).map_err(|e| ProjectError::codec(path, e))?;
    let newline = if original.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let names: Vec<_> = canonical
        .lines()
        .filter_map(row)
        .flat_map(|(_, values, _)| values.split(',').map(str::trim))
        .collect();
    let lines: Vec<_> = original.split_inclusive('\n').collect();
    let first = lines
        .iter()
        .position(|line| row(line).is_some())
        .ok_or_else(|| ProjectError::Format {
            path: path.into(),
            message: "palette table has no tilepal rows".into(),
        })?;
    let last = lines.iter().rposition(|line| row(line).is_some()).unwrap();
    let mut end = last + 1;
    let mut after = end;
    while after < lines.len() && lines[after].trim().is_empty() {
        after += 1;
    }
    if gap(&lines, after) {
        end = after + 3;
    }
    let old_gaps = (first..end).filter(|&i| gap(&lines, i)).count();
    let new_gaps = canonical
        .lines()
        .filter(|line| matches!(line.trim(), "rept 16" | "rept 32"))
        .count();
    if old.len() == new.len() && old_gaps == new_gaps {
        let mut out = String::new();
        let mut index = 0;
        for (line_index, line) in lines.iter().enumerate() {
            if let Some((prefix, values, suffix)) = row(line) {
                let count = values.split(',').count();
                if old[index..index + count] == new[index..index + count] {
                    out.push_str(line);
                } else {
                    out.push_str(prefix);
                    out.push(' ');
                    out.push_str(&names[index..index + count].join(", "));
                    out.push_str(suffix);
                }
                index += count;
            } else if (first..end).contains(&line_index) && gap(&lines, line_index) {
                let desired = if canonical.contains("rept 32\n") {
                    "32"
                } else {
                    "16"
                };
                let old_count = if line.contains("32") { "32" } else { "16" };
                out.push_str(&line.replacen(old_count, desired, 1));
            } else {
                out.push_str(line);
            }
        }
        return Ok(out.into_bytes());
    }
    // Resizing changes the bank/gap structure. Only rewrite a simple palette
    // table; refuse interspersed labels/macros whose ownership is ambiguous.
    let mut comments = Vec::new();
    let mut i = first;
    while i < end {
        let line = lines[i];
        if row(line).is_some() {
            if let Some((_, comment)) = line.split_once(';') {
                comments.push(format!(";{}", comment.trim_end_matches(['\r', '\n'])));
            }
            i += 1;
        } else if line.trim().is_empty() || line.trim_start().starts_with(';') {
            if !line.trim().is_empty() {
                comments.push(line.trim_end_matches(['\r', '\n']).into());
            }
            i += 1;
        } else if gap(&lines, i) {
            i += 3;
        } else {
            return Err(ProjectError::Format { path: path.into(), message: "cannot resize a palette table containing unrelated labels or macros; separate the table first".into() });
        }
    }
    let mut out = lines[..first].concat();
    for comment in comments {
        out.push_str(&comment);
        out.push_str(newline);
    }
    out.push_str(&canonical.replace('\n', newline));
    if !out.ends_with('\n') && end < lines.len() {
        out.push_str(newline);
    }
    out.push_str(&lines[end..].concat());
    Ok(out.into_bytes())
}
