use bevy::{prelude::*, render::Render};
use bevy_egui::egui::{Color32, RichText};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};
use game_test::engine::entity::EEntity;
use game_test::engine::{STEP_LEN_S, STEPS_PER_SECOND};
use game_test::timestamp;

use crate::GameState;
use crate::plugins::engine::{ActiveGameEngine, ActivePlayerEntityId};

#[derive(Resource, Default)]
pub struct EngineSyncInfo {
    pub fps: f64,
    pub last_frame: f64,
    pub server_step: u64,
    pub sync_distance: i64,
}

pub struct DataHUDPlugin;

impl Plugin for DataHUDPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EngineSyncInfo>().add_systems(
            EguiContextPass,
            display_hud.run_if(in_state(crate::GameState::OnMap)),
        );
    }
}

fn display_hud(
    mut contexts: EguiContexts,
    mut hud_info: ResMut<EngineSyncInfo>,
    active_game_engine: Res<ActiveGameEngine>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
) {
    let engine = &active_game_engine.0;
    if engine.step_index % STEPS_PER_SECOND == 0 {
        hud_info.fps = (timestamp() - hud_info.last_frame) / STEP_LEN_S;
        hud_info.last_frame = timestamp();
    }

    // Method 2: Even more minimal with custom frame
    egui::Window::new("")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .frame(egui::Frame {
            fill: egui::Color32::TRANSPARENT, // Transparent background
            stroke: egui::Stroke::new(1.0, egui::Color32::GRAY), // Simple border
            ..Default::default()
        })
        .fixed_pos((0., 0.))
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                ui.label(format!("fps: {}", hud_info.fps));
                ui.label(format!("engine step: {}", engine.step_index));
                ui.label(format!("server step: {}", hud_info.server_step));
                ui.label(format!("sync distance: {}", hud_info.sync_distance));
                if let Some(player_entity_id) = active_player_entity_id.0 {
                    if let Some(entity) = engine.entities.get(&player_entity_id) {
                        ui.label(format!(
                            "player position: {} {}",
                            entity.position().x,
                            entity.position().y
                        ));
                    }
                }
            });
        });
}
