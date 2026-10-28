//! master map-events.cpp/Event::parse and Config::event_script_path adaptations.
//! Untouched assembly bytes are retained, including CRLF and the final newline.
//! LGPL-3.0-only; see upstream.lock.toml and NOTICE.md.
use crate::{
    ProjectError, SaveTarget, TilesetOutput,
    document::{optional_hash, persist_bytes, prepare_target},
    read_bounded,
};
use polished_map_core::{EventCommand, EventDefinition, EventError, EventKind, Events};
use sha2::{Digest, Sha256};
use std::{
    ops::Range,
    path::{Path, PathBuf},
};

const LIMIT: usize = 4 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum EventScriptError {
    #[error("event script exceeds four MiB")]
    Limit,
    #[error("event script contains NUL")]
    Nul,
    #[error("line {line}: {message}")]
    Line { line: usize, message: String },
    #[error(transparent)]
    Model(#[from] EventError),
}

/// Upstream search order. A missing script returns the final fallback path.
pub fn event_script_path(root: &Path, map_name: &str) -> Result<PathBuf, ProjectError> {
    if map_name.is_empty()
        || !map_name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err(ProjectError::Format {
            path: root.into(),
            message: "invalid map name for event discovery".into(),
        });
    }
    let candidates = ["maps", "SXY", "data/maps/objects", "data/mapObjects"];
    Ok(candidates
        .iter()
        .map(|dir| root.join(dir).join(format!("{map_name}.asm")))
        .find(|path| path.is_file())
        .unwrap_or_else(|| root.join(candidates[3]).join(format!("{map_name}.asm"))))
}

#[derive(Clone, Debug)]
pub struct EventScript {
    source: String,
    spans: Vec<Range<usize>>,
    events: Events,
}
impl EventScript {
    pub fn parse(source: &str) -> Result<Self, EventScriptError> {
        if source.len() > LIMIT {
            return Err(EventScriptError::Limit);
        }
        if source.contains('\0') {
            return Err(EventScriptError::Nul);
        }
        let mut definitions = Vec::new();
        let mut spans = Vec::new();
        let mut offset = 0;
        for (line_index, line) in source.split_inclusive('\n').enumerate() {
            let body = line.strip_suffix('\n').unwrap_or(line);
            let body = body.strip_suffix('\r').unwrap_or(body);
            let start = offset;
            offset += line.len();
            if !body.starts_with([' ', '\t']) {
                continue;
            }
            let content = body.trim_start_matches(char::is_whitespace);
            let macro_end = content.find(char::is_whitespace).unwrap_or(content.len());
            let Some(kind) = EventKind::from_macro(&content[..macro_end]) else {
                continue;
            };
            let args = content[macro_end..].trim_start_matches(char::is_whitespace);
            if kind == EventKind::Warp
                && args
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .bytes()
                    .filter(|c| *c == b',')
                    .count()
                    != 3
            {
                continue;
            }
            let parse = || -> Result<EventDefinition, String> {
                let (prefix, args) = if kind.has_prefix() {
                    let pos = args.find(',').ok_or("missing prefix argument")?;
                    (Some(args[..=pos].into()), &args[pos + 1..])
                } else {
                    (None, args)
                };
                let (a, tail) = args.split_once(',').ok_or("missing coordinate separator")?;
                let end = tail.find([',', ';']).unwrap_or(tail.len());
                let b = &tail[..end];
                let suffix = (end < tail.len()).then(|| tail[end..].to_owned());
                let hex_coordinates = a.trim().starts_with('$') || b.trim().starts_with('$');
                let (a, b) = (coordinate(a)?, coordinate(b)?);
                let (x, y) = if kind.y_first() { (b, a) } else { (a, b) };
                Ok(EventDefinition {
                    kind,
                    x,
                    y,
                    hex_coordinates,
                    prefix,
                    suffix,
                    line: line_index + 1,
                })
            };
            let definition = parse().map_err(|message| EventScriptError::Line {
                line: line_index + 1,
                message,
            })?;
            // Keep indentation and line ending outside the owned replacement span.
            spans.push(start + body.len() - content.len()..start + body.len());
            definitions.push(definition);
        }
        Ok(Self {
            source: source.into(),
            spans,
            events: Events::new(definitions)?,
        })
    }
    pub fn events(&self) -> &Events {
        &self.events
    }
    pub fn apply(&mut self, commands: &[EventCommand]) -> Result<(), ProjectError> {
        let mut draft = self.events.clone();
        draft.apply_batch(commands)?;
        let mut length = self.source.len();
        for (span, event) in self.spans.iter().zip(draft.entries()) {
            if event.is_rewritten() {
                length = length - span.len() + event.text().len();
                if length > LIMIT {
                    return Err(EventError::Syntax("edited event script exceeds four MiB").into());
                }
            }
        }
        self.events = draft;
        Ok(())
    }
    pub fn render(&self) -> String {
        let mut output = String::with_capacity(self.source.len());
        let mut offset = 0;
        for (span, event) in self.spans.iter().zip(self.events.entries()) {
            output.push_str(&self.source[offset..span.start]);
            if event.is_rewritten() {
                output.push_str(&event.text());
            } else {
                output.push_str(&self.source[span.clone()]);
            }
            offset = span.end;
        }
        output.push_str(&self.source[offset..]);
        output
    }
}

fn coordinate(token: &str) -> Result<i16, String> {
    let token = token.trim();
    let (base, digits) = if let Some(s) = token.strip_prefix('$') {
        (16, s)
    } else if let Some(s) = token.strip_prefix('%') {
        (2, s)
    } else {
        (10, token)
    };
    let number = i64::from_str_radix(digits, base)
        .map_err(|_| format!("unsupported coordinate {token:?}; expected an integer literal"))?;
    if !(i64::from(i32::MIN)..=i64::from(u32::MAX)).contains(&number) {
        return Err("coordinate literal exceeds 32 bits".into());
    }
    // C++ narrows strtol to int, then uses signed remainder (not Euclidean modulo).
    let number = (number as i32)
        .checked_add(6)
        .ok_or("coordinate overflow")?
        % 256
        - 6;
    if !(-6..=249).contains(&number) {
        return Err("coordinate wraps outside -6..=249".into());
    }
    Ok(number as i16)
}

#[derive(Clone, Debug)]
pub struct EventDocument {
    source: SaveTarget,
    script: EventScript,
}
impl EventDocument {
    pub fn open(path: &Path) -> Result<Self, ProjectError> {
        let source = prepare_target(path)?;
        let bytes = read_bounded(source.path(), LIMIT)?;
        let text = std::str::from_utf8(&bytes).map_err(|e| ProjectError::Format {
            path: source.path.clone(),
            message: e.to_string(),
        })?;
        let script = EventScript::parse(text).map_err(|error| ProjectError::Format {
            path: source.path.clone(),
            message: error.to_string(),
        })?;
        if Some(Sha256::digest(&bytes).into()) != source.hash {
            return Err(ProjectError::Conflict(source.path));
        }
        Ok(Self { source, script })
    }
    pub fn source(&self) -> &SaveTarget {
        &self.source
    }
    pub fn script(&self) -> &EventScript {
        &self.script
    }
    pub fn apply(&mut self, commands: &[EventCommand]) -> Result<(), ProjectError> {
        self.script.apply(commands)
    }
    pub fn preview_save(&self) -> Result<Option<TilesetOutput>, ProjectError> {
        if optional_hash(self.source.path())? != self.source.hash {
            return Err(ProjectError::Conflict(self.source.path.clone()));
        }
        if !self.script.events.is_modified() {
            return Ok(None);
        }
        let bytes = self.script.render();
        Ok(Some(TilesetOutput {
            path: self.source.path.clone(),
            previous_hash: self.source.content_hash(),
            content_hash: Sha256::digest(bytes.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
            bytes: bytes.len(),
        }))
    }
    pub fn save(&mut self) -> Result<(), ProjectError> {
        if self.preview_save()?.is_none() {
            return Ok(());
        }
        let text = self.script.render();
        persist_bytes(self.source.path(), self.source.hash, text.as_bytes())?;
        self.source.hash = Some(Sha256::digest(text.as_bytes()).into());
        self.script.events.mark_saved();
        Ok(())
    }
}
