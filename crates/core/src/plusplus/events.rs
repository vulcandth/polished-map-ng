//! Pinned plusplus map-events.cpp `events_meta` catalog. LGPL-3.0-only.
//! Coordinates, edits, warp numbering and serialization use the shared model.
use crate::EventKind;

pub const EVENT_KINDS: [EventKind; 26] = [
    EventKind::WarpEvent,
    EventKind::CoordEvent,
    EventKind::BgEvent,
    EventKind::ObjectEvent,
    EventKind::WarpDef,
    EventKind::XyTrigger,
    EventKind::Signpost,
    EventKind::PersonEvent,
    EventKind::DummyWarp,
    EventKind::ItemballEvent,
    EventKind::KeyitemballEvent,
    EventKind::TmhmballEvent,
    EventKind::CuttreeEvent,
    EventKind::FruittreeEvent,
    EventKind::StrengthboulderEvent,
    EventKind::SmashrockEvent,
    EventKind::PokemonEvent,
    EventKind::PcNurseEvent,
    EventKind::MartClerkEvent,
    EventKind::PcChanseyEvent,
    EventKind::PcBlisseyEvent,
    EventKind::CavebaseEvent,
    EventKind::TreebaseLeftEvent1,
    EventKind::TreebaseLeftEvent2,
    EventKind::TreebaseRightEvent1,
    EventKind::TreebaseRightEvent2,
];

pub fn event_kind_from_macro(name: &str) -> Option<EventKind> {
    EVENT_KINDS
        .into_iter()
        .find(|kind| kind.macro_name() == name)
}
