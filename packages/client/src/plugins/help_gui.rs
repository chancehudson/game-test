use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui;

use crate::GameState;

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
            ui.label("Move with arrow keys");
            ui.label("Move through portals with up arrow key");
            ui.label("Jump with space");
            ui.label("Attack with a");
            ui.label("Respawn with r");
            ui.label("Resync engine with p");
            ui.label("Pick up item with z");
        });
}
