//! Indented assembly data from pinned upstream parse-asm.cpp.
//! LGPL-3.0-only; revisions are recorded in upstream.lock.toml.
use crate::codecs::CodecError;

/// The original data parser recognizes only indented db/dw statements.
pub fn parse_asm(text: &str) -> Result<Vec<u8>, CodecError> {
    let mut bytes = Vec::new();
    for (line, raw) in text.lines().enumerate() {
        let Some((op, arguments)) = indented_macro(raw) else {
            continue;
        };
        let words = op.eq_ignore_ascii_case("dw");
        if !words && !op.eq_ignore_ascii_case("db") {
            continue;
        }
        // getline accepts a final comma, but interior empty operands are not
        // numbers. Reject them instead of silently shifting subsequent bytes.
        for value in arguments.split_terminator(',') {
            let value = value.trim();
            let (digits, base) = if let Some(s) = value.strip_prefix('$') {
                (s, 16)
            } else if let Some(s) = value.strip_prefix('%') {
                (s, 2)
            } else if let Some(s) = value.strip_suffix(['h', 'H']) {
                (s, 16)
            } else {
                (value, 10)
            };
            let parsed = u16::from_str_radix(digits, base)
                .ok()
                .filter(|&n| words || n <= 255)
                .ok_or_else(|| CodecError::Assembly {
                    line: line + 1,
                    value: value.into(),
                })?;
            bytes.push(parsed as u8);
            if words {
                bytes.push((parsed >> 8) as u8);
            }
        }
    }
    Ok(bytes)
}

pub(crate) fn indented_macro(line: &str) -> Option<(&str, &str)> {
    if !line.starts_with([' ', '\t']) {
        return None;
    }
    let statement = line.split(';').next()?.trim();
    let offset = statement.find(char::is_whitespace)?;
    Some((&statement[..offset], statement[offset..].trim()))
}
