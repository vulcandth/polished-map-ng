//! Exact-modifier actions from pinned master Main_Window::handle/initialize.
//! Native input may deliver press and release in the same frame. LGPL-3.0-only.
use crate::egui;

/// Accept native clipboard events as well as raw keys from test/other backends.
/// Call only after the active editor/dialog has claimed keyboard input.
pub(super) fn consume_copy(input: &mut egui::InputState) -> bool {
    let mut consumed = input.consume_key(egui::Modifiers::COMMAND, egui::Key::C);
    input.events.retain(|event| {
        let copy = matches!(event, egui::Event::Copy);
        consumed |= copy;
        !copy
    });
    consumed
}

pub(super) fn consume_exact(
    input: &mut egui::InputState,
    required: egui::Modifiers,
    wanted: egui::Key,
) -> bool {
    let mut consumed = false;
    input.events.retain(|event| {
        let matches = matches!(event, egui::Event::Key {
            key, pressed: true, modifiers, ..
        } if *key == wanted && modifiers.matches_exact(required));
        consumed |= matches;
        !matches
    });
    consumed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_consumes_native_and_raw_forms_once_without_removing_other_input() {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            events: vec![
                egui::Event::Copy,
                egui::Event::Key {
                    key: egui::Key::C,
                    physical_key: Some(egui::Key::C),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                },
                egui::Event::Cut,
                egui::Event::Text("retained".into()),
            ],
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            ui.input_mut(|input| {
                assert!(consume_copy(input));
                assert!(!consume_copy(input));
                assert_eq!(
                    input.events,
                    [egui::Event::Cut, egui::Event::Text("retained".into())]
                );
            });
        });
        output.textures_delta.clear();
    }

    #[test]
    fn released_native_chords_use_event_modifiers_and_keep_unrelated_events() {
        for key in [egui::Key::Slash, egui::Key::P, egui::Key::F10] {
            let required = if key == egui::Key::F10 {
                egui::Modifiers::NONE
            } else {
                egui::Modifiers::COMMAND
            };
            for extra in [
                egui::Modifiers::NONE,
                egui::Modifiers::SHIFT,
                egui::Modifiers::ALT,
            ] {
                let ctx = egui::Context::default();
                let modifiers = required | extra;
                let mut input = egui::RawInput::default();
                for pressed in [true, false] {
                    input.events.push(egui::Event::Key {
                        key,
                        physical_key: Some(key),
                        pressed,
                        repeat: false,
                        modifiers,
                    });
                }
                input
                    .events
                    .push(egui::Event::ModifiersChanged(egui::Modifiers::NONE));
                let mut output =
                    ctx.run_ui(input, |ui| {
                        ui.input_mut(|input| {
                            assert_eq!(consume_exact(input, required, key), extra.is_none());
                            assert!(input.events.iter().any(|event| matches!(
                                event,
                                egui::Event::Key { pressed: false, .. }
                            )));
                            assert!(!consume_exact(input, required, key));
                        });
                    });
                output.textures_delta.clear();
            }
        }
    }
}
