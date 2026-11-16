//! Shared event semantics adapted from pinned master/plusplus event.cpp, map-events.cpp and
//! Main_Window::resize_map/change_event_cb; pinned in upstream.lock.toml.
//! Copyright Rangi42/Polished Map contributors and 2026 Polished Map NG contributors.
//! LGPL-3.0-only.
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Recognized assembly event macros, including both legacy and extended catalogs.
///
/// Use [`crate::event_catalog`] for context-specific parsing; [`Self::ALL`] is
/// the original classic catalog, not the union of all variants.
pub enum EventKind {
    /// The `warp` assembly macro.
    Warp,
    /// The `sign` assembly macro.
    Sign,
    /// The `object` assembly macro.
    Object,
    /// The `warp_to` assembly macro.
    WarpTo,
    /// The `warp_event` assembly macro.
    WarpEvent,
    /// The `coord_event` assembly macro.
    CoordEvent,
    /// The `bg_event` assembly macro.
    BgEvent,
    /// The `object_event` assembly macro.
    ObjectEvent,
    /// The `warp_def` assembly macro.
    WarpDef,
    /// The `xy_trigger` assembly macro.
    XyTrigger,
    /// The `signpost` assembly macro.
    Signpost,
    /// The `person_event` assembly macro.
    PersonEvent,
    /// The `dummy_warp` assembly macro.
    DummyWarp,
    /// The `itemball_event` assembly macro.
    ItemballEvent,
    /// The `keyitemball_event` assembly macro.
    KeyitemballEvent,
    /// The `tmhmball_event` assembly macro.
    TmhmballEvent,
    /// The `cuttree_event` assembly macro.
    CuttreeEvent,
    /// The `fruittree_event` assembly macro.
    FruittreeEvent,
    /// The `strengthboulder_event` assembly macro.
    StrengthboulderEvent,
    /// The `smashrock_event` assembly macro.
    SmashrockEvent,
    /// The `pc_nurse_event` assembly macro.
    PcNurseEvent,
    /// The `mart_clerk_event` assembly macro.
    MartClerkEvent,
    /// The `pokemon_event` assembly macro.
    PokemonEvent,
    /// The `pc_chansey_event` assembly macro.
    PcChanseyEvent,
    /// The `pc_blissey_event` assembly macro.
    PcBlisseyEvent,
    /// The `cavebase_event` assembly macro.
    CavebaseEvent,
    #[serde(rename = "treebase_left_event_1")]
    /// The `treebase_left_event_1` assembly macro.
    TreebaseLeftEvent1,
    #[serde(rename = "treebase_left_event_2")]
    /// The `treebase_left_event_2` assembly macro.
    TreebaseLeftEvent2,
    #[serde(rename = "treebase_right_event_1")]
    /// The `treebase_right_event_1` assembly macro.
    TreebaseRightEvent1,
    #[serde(rename = "treebase_right_event_2")]
    /// The `treebase_right_event_2` assembly macro.
    TreebaseRightEvent2,
}

impl EventKind {
    /// Interior and one-pixel-border RGB in both branches' event textures (alpha 128).
    pub fn marker_rgb(self) -> ([u8; 3], [u8; 3]) {
        use EventKind::*;
        match self {
            Warp | WarpEvent | WarpDef | DummyWarp => ([160, 0, 160], [110, 0, 110]),
            Sign | BgEvent | Signpost => ([255, 0, 0], [205, 0, 0]),
            Object | ObjectEvent | PersonEvent => ([48, 48, 255], [0, 0, 205]),
            CoordEvent | XyTrigger => ([0, 160, 0], [0, 110, 0]),
            WarpTo | PcChanseyEvent | PcBlisseyEvent | CavebaseEvent | TreebaseLeftEvent1
            | TreebaseLeftEvent2 | TreebaseRightEvent1 | TreebaseRightEvent2 => {
                ([255, 128, 0], [205, 78, 0])
            }
            _ => ([0, 128, 255], [0, 78, 205]),
        }
    }

    /// Master's catalog; plusplus has an explicit catalog in `event_catalog::EVENT_KINDS`.
    pub const ALL: [Self; 22] = [
        Self::Warp,
        Self::Sign,
        Self::Object,
        Self::WarpTo,
        Self::WarpEvent,
        Self::CoordEvent,
        Self::BgEvent,
        Self::ObjectEvent,
        Self::WarpDef,
        Self::XyTrigger,
        Self::Signpost,
        Self::PersonEvent,
        Self::DummyWarp,
        Self::ItemballEvent,
        Self::KeyitemballEvent,
        Self::TmhmballEvent,
        Self::CuttreeEvent,
        Self::FruittreeEvent,
        Self::StrengthboulderEvent,
        Self::SmashrockEvent,
        Self::PcNurseEvent,
        Self::MartClerkEvent,
    ];
    /// Exact assembly macro spelling for this event kind.
    pub fn macro_name(self) -> &'static str {
        match self {
            Self::Warp => "warp",
            Self::Sign => "sign",
            Self::Object => "object",
            Self::WarpTo => "warp_to",
            Self::WarpEvent => "warp_event",
            Self::CoordEvent => "coord_event",
            Self::BgEvent => "bg_event",
            Self::ObjectEvent => "object_event",
            Self::WarpDef => "warp_def",
            Self::XyTrigger => "xy_trigger",
            Self::Signpost => "signpost",
            Self::PersonEvent => "person_event",
            Self::DummyWarp => "dummy_warp",
            Self::ItemballEvent => "itemball_event",
            Self::KeyitemballEvent => "keyitemball_event",
            Self::TmhmballEvent => "tmhmball_event",
            Self::CuttreeEvent => "cuttree_event",
            Self::FruittreeEvent => "fruittree_event",
            Self::StrengthboulderEvent => "strengthboulder_event",
            Self::SmashrockEvent => "smashrock_event",
            Self::PcNurseEvent => "pc_nurse_event",
            Self::MartClerkEvent => "mart_clerk_event",
            Self::PokemonEvent => "pokemon_event",
            Self::PcChanseyEvent => "pc_chansey_event",
            Self::PcBlisseyEvent => "pc_blissey_event",
            Self::CavebaseEvent => "cavebase_event",
            Self::TreebaseLeftEvent1 => "treebase_left_event_1",
            Self::TreebaseLeftEvent2 => "treebase_left_event_2",
            Self::TreebaseRightEvent1 => "treebase_right_event_1",
            Self::TreebaseRightEvent2 => "treebase_right_event_2",
        }
    }
    /// Look up a macro in the classic catalog; extended-only names return `None`.
    pub fn from_macro(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.macro_name() == name)
    }
    /// Whether the macro has an argument preceding its coordinates.
    pub fn has_prefix(self) -> bool {
        matches!(self, Self::Object | Self::XyTrigger | Self::PersonEvent)
    }
    /// Whether assembly syntax stores the vertical coordinate before horizontal.
    pub fn y_first(self) -> bool {
        matches!(
            self,
            Self::WarpDef | Self::XyTrigger | Self::Signpost | Self::PersonEvent | Self::DummyWarp
        )
    }
    /// Whether this kind receives a one-based warp ID and has a destination.
    pub fn is_warp(self) -> bool {
        matches!(
            self,
            Self::Warp | Self::WarpEvent | Self::WarpDef | Self::DummyWarp
        )
    }
    /// Whether a warp stores destination ID before destination map name.
    pub fn id_before_map(self) -> bool {
        matches!(self, Self::Warp | Self::WarpDef)
    }
    /// Single-character marker used by the canvas and print renderer.
    pub fn symbol(self) -> char {
        match self {
            Self::Warp | Self::WarpEvent | Self::WarpDef => 'W',
            Self::Sign | Self::Signpost => 'S',
            Self::Object | Self::ObjectEvent => 'O',
            Self::WarpTo | Self::TmhmballEvent => 'T',
            Self::CoordEvent | Self::CuttreeEvent | Self::PcChanseyEvent => 'C',
            Self::BgEvent | Self::StrengthboulderEvent | Self::PcBlisseyEvent => 'B',
            Self::XyTrigger => 'X',
            Self::PersonEvent | Self::PokemonEvent => 'P',
            Self::DummyWarp => 'D',
            Self::ItemballEvent => 'I',
            Self::KeyitemballEvent => 'K',
            Self::FruittreeEvent => 'F',
            Self::SmashrockEvent => 'R',
            Self::PcNurseEvent => 'N',
            Self::MartClerkEvent => 'M',
            Self::CavebaseEvent => 'V',
            Self::TreebaseLeftEvent1 => '¿',
            Self::TreebaseLeftEvent2 => '¡',
            Self::TreebaseRightEvent1 => '?',
            Self::TreebaseRightEvent2 => '!',
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
/// Parsed event source fields. [`Events::new`] validates coordinates and syntax.
pub struct EventDefinition {
    /// Assembly macro kind.
    pub kind: EventKind,
    /// Horizontal coordinate in 16-pixel event cells, -6..=249.
    pub x: i16,
    /// Vertical coordinate in 16-pixel event cells, -6..=249.
    pub y: i16,
    /// Serialize coordinates as hexadecimal instead of decimal.
    pub hex_coordinates: bool,
    /// Includes the trailing comma, when the macro has a leading argument.
    pub prefix: Option<String>,
    /// Includes its initial comma or semicolon.
    pub suffix: Option<String>,
    /// One-based source line number.
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
/// A validated event, its assigned warp ID, and source-rewrite state.
pub struct Event {
    definition: EventDefinition,
    warp_id: Option<usize>,
    rewritten: bool,
}
impl Event {
    /// Validated coordinates, macro kind and preserved source fields.
    pub fn definition(&self) -> &EventDefinition {
        &self.definition
    }
    /// One-based ordinal among warp events, or `None` for other event kinds.
    pub fn warp_id(&self) -> Option<usize> {
        self.warp_id
    }
    /// Whether serialization must replace this event's original source line.
    pub fn is_rewritten(&self) -> bool {
        self.rewritten
    }
    /// Serialize the macro and arguments using the stored coordinate base and order.
    pub fn text(&self) -> String {
        let d = &self.definition;
        let (a, b) = if d.kind.y_first() {
            (d.y, d.x)
        } else {
            (d.x, d.y)
        };
        let number = |n: i16| {
            if d.hex_coordinates {
                format!("${:x}", i32::from(n) as u32)
            } else {
                format!("{n:2}")
            }
        };
        format!(
            "{} {}{}, {}{}",
            d.kind.macro_name(),
            d.prefix
                .as_ref()
                .map(|p| format!("{p} "))
                .unwrap_or_default(),
            number(a),
            number(b),
            d.suffix.as_deref().unwrap_or_default()
        )
    }
    /// Extract the normalized destination map name and signed warp ID.
    ///
    /// Returns `None` for non-warps or absent arguments. Symbolic/non-numeric
    /// IDs resolve to zero, following the original numeric-prefix parser.
    pub fn warp_destination(&self) -> Option<(String, i32)> {
        if !self.definition.kind.is_warp() {
            return None;
        }
        let mut args = self.definition.suffix.as_deref()?.split(',');
        args.next()?;
        let (mut map, mut id) = (args.next()?, args.next()?);
        if self.definition.kind.id_before_map() {
            std::mem::swap(&mut map, &mut id);
        }
        let mut name = String::new();
        let (mut letter, mut digit, mut digit_underscore) = (false, false, false);
        for mut c in map.split(';').next()?.trim().chars() {
            if c != '_' {
                if digit_underscore && c.is_ascii_digit() {
                    name.push('_');
                }
                if letter {
                    c = c.to_ascii_lowercase();
                }
                name.push(c);
                digit_underscore = false;
            } else if digit {
                digit_underscore = true;
            }
            letter = c.is_ascii_alphabetic();
            digit = c.is_ascii_digit();
        }
        // Upstream strtol reads a numeric prefix and treats a symbolic ID as zero.
        id = id.trim();
        let (base, digits) = if let Some(v) = id.strip_prefix('$') {
            (16, v)
        } else if let Some(v) = id.strip_prefix('%') {
            (2, v)
        } else {
            (10, id)
        };
        let digits = digits.trim_start();
        let end = digits
            .char_indices()
            .take_while(|(i, c)| c.is_digit(base) || (*i == 0 && (*c == '-' || *c == '+')))
            .last()
            .map_or(0, |(i, c)| i + c.len_utf8());
        let id = i64::from_str_radix(&digits[..end], base)
            .ok()
            .filter(|n| (i64::from(i32::MIN)..=i64::from(u32::MAX)).contains(n))
            .map_or(0, |n| n as i32);
        Some((name, id))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
/// Position and field edits for existing events; no add/remove operations.
pub enum EventCommand {
    /// Move one event without changing its source arguments.
    Move {
        /// Zero-based index in [`Events::entries`].
        index: usize,
        /// Horizontal coordinate in 16-pixel cells, -6..=249.
        x: i16,
        /// Vertical coordinate in 16-pixel cells, -6..=249.
        y: i16,
    },
    /// Move an event and optionally replace its existing prefix/suffix fields.
    Edit {
        /// Zero-based index in [`Events::entries`].
        index: usize,
        /// Horizontal coordinate in 16-pixel cells, -6..=249.
        x: i16,
        /// Vertical coordinate in 16-pixel cells, -6..=249.
        y: i16,
        /// Replacement leading argument; `None` preserves it.
        prefix: Option<String>,
        /// Replacement trailing arguments; `None` preserves them.
        suffix: Option<String>,
    },
    /// Map-resize anchor displacement, in 32px blocks (events use 16px cells).
    Translate {
        /// Horizontal displacement in 32-pixel map blocks.
        block_dx: i16,
        /// Vertical displacement in 32-pixel map blocks.
        block_dy: i16,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
/// Rejected event coordinates, index, or source syntax.
pub enum EventError {
    #[error("event coordinate must be in -6..=249")]
    /// A coordinate is outside -6..=249 event cells.
    Coordinate,
    #[error("event index {0} is out of range")]
    /// The zero-based event index is absent.
    Index(usize),
    #[error("invalid event syntax: {0}")]
    /// A source argument or line number violates an event syntax constraint.
    Syntax(&'static str),
}

fn coordinates(x: i16, y: i16) -> Result<(), EventError> {
    if (-6..=249).contains(&x) && (-6..=249).contains(&y) {
        Ok(())
    } else {
        Err(EventError::Coordinate)
    }
}
fn text_valid(s: &str) -> bool {
    s.len() <= 65536 && !s.contains(['\r', '\n', '\0'])
}

fn validate_definition(d: &EventDefinition) -> Result<(), EventError> {
    coordinates(d.x, d.y)?;
    if d.line == 0
        || d.kind.has_prefix() != d.prefix.is_some()
        || d.prefix.as_ref().is_some_and(|p| {
            !text_valid(p) || p.strip_suffix(',').is_none_or(|p| p.contains([',', ';']))
        })
        || d.suffix
            .as_ref()
            .is_some_and(|s| !text_valid(s) || !s.trim_start().starts_with([',', ';']))
    {
        return Err(EventError::Syntax("source prefix, suffix or line number"));
    }
    if d.kind == EventKind::Warp
        && d.suffix
            .as_deref()
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("")
            .bytes()
            .filter(|c| *c == b',')
            .count()
            != 2
    {
        return Err(EventError::Syntax(
            "warp event requires destination ID and map",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Validated event sequence with one-based warp IDs and sticky dirty state.
pub struct Events {
    entries: Vec<Event>,
    modified: bool,
}
impl Events {
    /// Validate source definitions and assign warp IDs in input order.
    ///
    /// # Errors
    /// Rejects coordinates outside -6..=249, zero line numbers, mismatched
    /// prefix fields, malformed suffixes, or invalid warp arguments.
    pub fn new(definitions: Vec<EventDefinition>) -> Result<Self, EventError> {
        let mut warp_id = 0;
        let mut entries = Vec::with_capacity(definitions.len());
        for d in definitions {
            validate_definition(&d)?;
            let id = if d.kind.is_warp() {
                warp_id += 1;
                Some(warp_id)
            } else {
                None
            };
            entries.push(Event {
                definition: d,
                warp_id: id,
                rewritten: false,
            });
        }
        Ok(Self {
            entries,
            modified: false,
        })
    }
    /// Events in source order; command indices refer to this slice.
    pub fn entries(&self) -> &[Event] {
        &self.entries
    }
    /// Whether an edit has succeeded since construction or the last save.
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    /// Clear dirty state; retain rewrite flags for future source serialization.
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
    /// Find a warp’s destination index within the named map.
    ///
    /// Returns `None` for missing events, other maps, symbolic IDs, or absent targets.
    pub fn local_warp_target(&self, index: usize, map_name: &str) -> Option<usize> {
        let (name, id) = self.entries.get(index)?.warp_destination()?;
        if name != map_name || id <= 0 {
            return None;
        }
        self.entries
            .iter()
            .position(|e| e.warp_id == Some(id as usize))
    }
    /// Match the original drag bounds; event options use the full coordinate range.
    pub fn drag_max(block_dimension: u8) -> i16 {
        (i16::from(block_dimension) * 2 + 5).min(249)
    }
    /// Apply a complete batch on a draft and commit only if every command succeeds.
    ///
    /// # Errors
    /// Any invalid index, coordinate, or field syntax leaves all events unchanged.
    pub fn apply_batch(&mut self, commands: &[EventCommand]) -> Result<(), EventError> {
        let mut draft = self.clone();
        for command in commands {
            draft.apply(command)?;
        }
        *self = draft;
        Ok(())
    }
    /// Validate and apply one edit; translations clamp coordinates to -6..=249.
    ///
    /// Field edits flatten line feeds and normalize separators. `None` retains
    /// the existing field; it does not remove it. Successful edits mark source
    /// lines for rewriting. A zero translation leaves state unchanged.
    ///
    /// # Errors
    /// Invalid indices, coordinates, or field syntax leave the sequence unchanged.
    pub fn apply(&mut self, command: &EventCommand) -> Result<(), EventError> {
        match command {
            EventCommand::Move { index, x, y } | EventCommand::Edit { index, x, y, .. } => {
                coordinates(*x, *y)?;
                let event = self
                    .entries
                    .get_mut(*index)
                    .ok_or(EventError::Index(*index))?;
                let mut draft = event.definition.clone();
                if let EventCommand::Edit { prefix, suffix, .. } = command {
                    for (value, field, is_prefix) in [
                        (prefix, &mut draft.prefix, true),
                        (suffix, &mut draft.suffix, false),
                    ] {
                        if let Some(value) = value {
                            if field.is_none() {
                                return Err(EventError::Syntax("this event has no such field"));
                            }
                            let mut value = value.trim().replace('\n', " ");
                            if !text_valid(&value) {
                                return Err(EventError::Syntax(
                                    "field must be bounded single-line text",
                                ));
                            }
                            if is_prefix && !value.ends_with(',') {
                                value.push(',');
                            }
                            if !is_prefix {
                                if value.starts_with(';') {
                                    value.insert(0, ' ');
                                } else if !value.starts_with(',') {
                                    value.insert_str(0, ", ");
                                }
                            }
                            *field = Some(value);
                        }
                    }
                }
                draft.x = *x;
                draft.y = *y;
                validate_definition(&draft)?;
                event.definition = draft;
                event.rewritten = true;
                self.modified = true;
            }
            EventCommand::Translate { block_dx, block_dy } => {
                if *block_dx == 0 && *block_dy == 0 {
                    return Ok(());
                }
                for event in &mut self.entries {
                    let d = &mut event.definition;
                    d.x = (i32::from(d.x) + i32::from(*block_dx) * 2).clamp(-6, 249) as i16;
                    d.y = (i32::from(d.y) + i32::from(*block_dy) * 2).clamp(-6, 249) as i16;
                    event.rewritten = true;
                    self.modified = true;
                }
            }
        }
        Ok(())
    }
}
