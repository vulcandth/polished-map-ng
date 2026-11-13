//! Main toolbar controls and command dispatch; upstream Main_Window. LGPL-3.0-only.
use super::*;

impl Editor {
    pub(super) fn toolbar(&mut self, ui: &mut egui::Ui) {
        use icons::{Button, Icon};
        ui.spacing_mut().item_spacing.x = 0.0;
        if ui
            .add(Button::new(Icon::New, "New"))
            .on_hover_text("New… (Ctrl/Cmd+N)")
            .clicked()
        {
            self.choose_new();
        }
        if ui
            .add(Button::new(Icon::Open, "Open…"))
            .on_hover_text("Open… (Ctrl/Cmd+O)")
            .clicked()
        {
            self.choose_open();
        }
        if ui
            .add_enabled(
                self.loaded.is_some(),
                Button::new(Icon::LoadEvents, "Load events"),
            )
            .on_hover_text("Load Event Script… (Ctrl/Cmd+A)")
            .clicked()
        {
            self.choose_events();
        }
        if ui
            .add_enabled(
                self.loaded.as_ref().is_some_and(|l| l.events.is_some()),
                Button::new(Icon::Reload, "Reload events"),
            )
            .on_hover_text("Reload Event Script (Ctrl/Cmd+R)")
            .clicked()
        {
            self.request_event_change(event_workflows::EventChange::Reload);
        }
        if ui
            .add_enabled(self.loaded.is_some(), Button::new(Icon::Save, "Save"))
            .on_hover_text("Save (Ctrl/Cmd+S)")
            .clicked()
        {
            self.save();
        }
        icons::separator(ui);
        if ui
            .add_enabled(
                self.loaded.is_some()
                    && self.print_worker.is_none()
                    && self.print_clipboard.is_none()
                    && self.print_request.is_none(),
                Button::new(Icon::Print, "Print…"),
            )
            .on_hover_text("Print (Ctrl/Cmd+P)")
            .clicked()
        {
            self.prepare_print();
        }
        icons::separator(ui);
        let undo = self
            .loaded
            .as_ref()
            .is_some_and(|l| l.document.map().can_undo());
        if ui
            .add_enabled(undo, Button::new(Icon::Undo, "Undo"))
            .on_hover_text("Undo (Ctrl/Cmd+Z)")
            .clicked()
            && let Some(loaded) = &mut self.loaded
        {
            loaded.document.map_mut().undo();
        }
        let redo = self
            .loaded
            .as_ref()
            .is_some_and(|l| l.document.map().can_redo());
        if ui
            .add_enabled(redo, Button::new(Icon::Redo, "Redo"))
            .on_hover_text("Redo (Ctrl/Cmd+Y)")
            .clicked()
            && let Some(loaded) = &mut self.loaded
        {
            loaded.document.map_mut().redo();
        }
        icons::separator(ui);
        for (icon, label, help, value) in [
            (Icon::Grid, "Grid", "Grid (Ctrl/Cmd+G)", &mut self.grid),
            (
                Icon::Rulers,
                "Rulers",
                "Rulers (Ctrl/Cmd+Shift+R)",
                &mut self.rulers,
            ),
            (Icon::Zoom, "Zoom", "Zoom (Ctrl/Cmd+=)", &mut self.zoom),
            (Icon::Ids, "IDs", "Block IDs (Ctrl/Cmd+I)", &mut self.ids),
            (
                Icon::Hex,
                "Hex",
                "Hexadecimal (Ctrl/Cmd+Shift+4)",
                &mut self.hex,
            ),
        ] {
            if ui
                .add(Button::new(icon, label).selected(*value))
                .on_hover_text(help)
                .clicked()
            {
                *value = !*value;
            }
        }
        if ui
            .add_enabled(
                self.allow_priority || self.profile() == documents::Profile::PlusPlus,
                Button::new(Icon::Priority, "Show Priority").selected(self.show_priority),
            )
            .on_hover_text("Show Priority (Ctrl/Cmd+Shift+P)")
            .clicked()
        {
            self.show_priority = !self.show_priority;
        }
        for (icon, label, help, value) in [
            (
                Icon::GameBoy,
                "Game Boy screen",
                "Game Boy Screen (Ctrl/Cmd+Shift+M)",
                &mut self.event_ui.game_boy,
            ),
            (
                Icon::ShowEvents,
                "Show events",
                "Show Events (Ctrl/Cmd+Shift+V)",
                &mut self.event_ui.show,
            ),
            (
                Icon::WarpIds,
                "Show warp IDs",
                "Show Warp IDs (Ctrl/Cmd+Shift+3)",
                &mut self.event_ui.warp_ids,
            ),
        ] {
            if ui
                .add(Button::new(icon, label).selected(*value))
                .on_hover_text(help)
                .clicked()
            {
                *value = !*value;
            }
        }
        icons::separator(ui);
        if ui
            .add(Button::new(Icon::Blocks, "Blocks mode").selected(!self.event_ui.mode))
            .on_hover_text("Blocks Mode (Ctrl/Cmd+Shift+B)")
            .clicked()
        {
            self.set_event_mode(false);
        }
        if ui
            .add(Button::new(Icon::Events, "Events mode").selected(self.event_ui.mode))
            .on_hover_text("Events Mode (Ctrl/Cmd+Shift+E)")
            .clicked()
        {
            self.set_event_mode(true);
        }
        icons::separator(ui);
        self.palette_set_toolbar(ui);
        if ui
            .add(Button::new(Icon::LoadPalettes, "Load palettes"))
            .on_hover_text("Load Palettes… (Ctrl/Cmd+L)")
            .clicked()
        {
            self.choose_palettes(ui.ctx());
        }
        if ui
            .add(Button::new(Icon::Palettes, "Edit current palettes"))
            .on_hover_text("Edit Current Palettes… (Ctrl/Cmd+Shift+L)")
            .clicked()
        {
            self.prepare_palettes();
        }
        icons::separator(ui);
        if ui
            .add_enabled(
                self.loaded
                    .as_ref()
                    .is_some_and(|l| l.blockset.is_some() || l.plus_assets.is_some()),
                Button::new(Icon::ResizeBlockset, "Resize blockset"),
            )
            .on_hover_text("Resize Blockset… (Ctrl/Cmd+B)")
            .clicked()
        {
            self.prepare_block_count();
        }
        if ui
            .add_enabled(
                self.loaded.is_some(),
                Button::new(Icon::Resize, "Resize map"),
            )
            .on_hover_text("Resize Map… (Ctrl/Cmd+E)")
            .clicked()
        {
            self.prepare_resize();
        }
        icons::separator(ui);
        let change = self.has_tileset_assets();
        if ui
            .add_enabled(change, Button::new(Icon::ChangeTileset, "Change tileset"))
            .on_hover_text("Change Tileset… (Ctrl/Cmd+H)")
            .clicked()
        {
            self.request_tileset_change();
        }
        if ui
            .add_enabled(self.can_edit_tiles(), Button::new(Icon::Tileset, "Tileset"))
            .on_hover_text("Edit Tileset… (Ctrl/Cmd+T)")
            .clicked()
        {
            self.prepare_tileset();
        }
        icons::separator(ui);
        if ui
            .add_enabled(
                self.can_change_roof(),
                Button::new(Icon::ChangeRoof, "Change roof"),
            )
            .on_hover_text("Change Roof… (Ctrl/Cmd+Shift+H)")
            .clicked()
        {
            self.request_roof_change();
        }
        let roof = self.can_edit_roof();
        if ui
            .add_enabled(roof, Button::new(Icon::Roof, "Roof"))
            .on_hover_text("Edit Roof… (Ctrl/Cmd+F)")
            .clicked()
        {
            self.prepare_roof();
        }
    }
}
