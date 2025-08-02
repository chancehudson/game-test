use std::time::Instant;

use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_egui::egui::RichText;
use bevy_egui::{EguiContextPass, EguiContexts, egui};

use game_common::prelude::*;
use keind::prelude::*;
use keind_time::GameEngineTime;

use crate::plugins::engine::ActiveGameEngine;
use crate::plugins::engine::ActivePlayerEntityId;

#[derive(Resource, Default)]
pub struct EngineSyncInfo {
    pub fps: f32,
    pub server_step_timestamp: Option<Instant>,
    pub server_step: u64,
    pub sync_distance: i64,
    pub requested_resync: bool,
    pub last_fps_timestamp: Option<Instant>,
    pub last_fps_step_index: u64,
    pub engine_time: GameEngineTime,
}

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FpsCounter {
    #[default]
    Shown,
    Hidden,
}

pub struct DataHUDPlugin;

impl Plugin for DataHUDPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<FpsCounter>()
            .init_resource::<EngineSyncInfo>()
            .add_systems(
                EguiContextPass,
                (
                    display_hud.run_if(in_state(crate::GameState::OnMap)),
                    display_fps.run_if(in_state(FpsCounter::Shown)),
                ),
            );
    }
}

fn display_fps(mut hud_info: ResMut<EngineSyncInfo>, active_game_engine: Res<ActiveGameEngine>) {
    let engine = &active_game_engine.0;

    const FPS_SAMPLE_LEN_S: f32 = 2.0;
    let now = Instant::now();
    if let Some(last_instant) = hud_info.last_fps_timestamp
        && let elapsed_secs = now.duration_since(last_instant).as_secs_f32()
        && elapsed_secs >= FPS_SAMPLE_LEN_S
    {
        assert!(
            engine.step_index() < &hud_info.last_fps_step_index,
            "fps counter time ran backward"
        );
        let elapsed_steps = (engine.step_index() - hud_info.last_fps_step_index) as f32;
        hud_info.fps = (elapsed_secs / elapsed_steps).round();
        hud_info.last_fps_timestamp = Some(now);
        hud_info.last_fps_step_index = *engine.step_index();
    } else {
        hud_info.last_fps_timestamp = Some(Instant::now());
    }
}

fn display_hud(
    mut contexts: EguiContexts,
    hud_info: ResMut<EngineSyncInfo>,
    active_game_engine: Res<ActiveGameEngine>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
) {
    let engine = &active_game_engine.0;

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
                ui.label(format!("engine step: {}", engine.step_index()));
                ui.label(format!("server step: {}", hud_info.server_step));
                ui.label(format!("sync distance: {}", hud_info.sync_distance));
                ui.label(format!("entity count: {}", engine.entity_count()));
                if hud_info.requested_resync {
                    ui.label(RichText::new("requested resync!").color(Color32::RED));
                }
                if let Some(player_entity_id) = active_player_entity_id.0 {
                    if let Some(entity) = engine
                        .entities_at_step(engine.step_index())
                        .get(&player_entity_id)
                    {
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
