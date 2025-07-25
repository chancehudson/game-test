use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui;

use crate::GameState;
use crate::ui::draw_key_binding_inline;

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum HelpGuiState {
    #[default]
    Hidden,
    Visible,
}

pub struct HelpGuiPlugin;

impl Plugin for HelpGuiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<HelpGuiState>().add_systems(
            Update,
            show_help_gui.run_if(in_state(GameState::OnMap).and(in_state(HelpGuiState::Visible))),
        );
    }
}

fn show_help_gui(mut contexts: EguiContexts) {
    egui::Window::new("Help")
        .default_height(300.)
        .min_width(150.)
        .max_width(150.)
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Controls");
            ui.horizontal(|ui| {
                draw_key_binding_inline(ui, "← →");
                ui.label("Move");
            });
            ui.horizontal(|ui| {
                draw_key_binding_inline(ui, "↑");
                ui.label("Enter portal");
            });
            ui.horizontal(|ui| {
                draw_key_binding_inline(ui, "space");
                ui.label("Jump");
            });
            ui.horizontal(|ui| {
                draw_key_binding_inline(ui, "↓ + space");
                ui.label("Jump down");
            });
            ui.horizontal(|ui| {
                draw_key_binding_inline(ui, "a");
                ui.label("Attack");
            });
            ui.horizontal(|ui| {
                draw_key_binding_inline(ui, "r");
                ui.label("Respawn");
            });
            ui.horizontal(|ui| {
                draw_key_binding_inline(ui, "p");
                ui.label("Request full resync");
            });
            ui.horizontal(|ui| {
                draw_key_binding_inline(ui, "z");
                ui.label("Pick up item");
            });
        });
}
