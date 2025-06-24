use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use game_test::{action::Action, engine::game_event::EngineEvent};

use crate::{
    GameState,
    network::NetworkAction,
    plugins::engine::{ActiveGameEngine, ActivePlayerEntityId},
};

pub struct ChatPlugin;

#[derive(Resource, Default)]
pub struct ChatBarState {
    showing: bool,
    current_msg: String,
    // username, content
    history: Vec<(String, String)>,
}

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatBarState>().add_systems(
            Update,
            (chat_window, chat_key_listener).run_if(in_state(GameState::OnMap)),
        );
    }
}

fn chat_key_listener(mut chat_state: ResMut<ChatBarState>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Enter) {
        chat_state.showing = !chat_state.showing;
    }
}

fn chat_window(
    mut active_player_entity_id: ResMut<ActivePlayerEntityId>,
    mut chat_state: ResMut<ChatBarState>,
    mut contexts: EguiContexts,
    mut active_engine: ResMut<ActiveGameEngine>,
    mut action_events: EventWriter<NetworkAction>,
) {
    if !chat_state.showing || active_player_entity_id.0.is_none() {
        return;
    }
    let player_entity_id = active_player_entity_id.0.unwrap();
    let engine = &mut active_engine.0;
    egui::Window::new("local chat")
        .default_size([200.0, 150.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.set_max_height(400.0);
            // Chat history area (takes most of the space)
            egui::ScrollArea::vertical()
                .auto_shrink([true, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (username, content) in &chat_state.history {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}:", username))
                                    .color(egui::Color32::from_rgb(100, 149, 237))
                                    .strong(),
                            );
                            ui.label(content);
                        });
                        ui.separator();
                    }
                });

            ui.separator();

            // Input area at bottom
            ui.horizontal(|ui| {
                ui.allocate_ui(egui::Vec2::new(ui.available_width() - 60.0, 20.0), |ui| {
                    let text_edit = ui.text_edit_singleline(&mut chat_state.current_msg);
                    // Handle Enter key here
                    if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let chat_event = EngineEvent::ChatMessage {
                            id: rand::random(),
                            text: chat_state.current_msg.to_string(),
                            entity_id: player_entity_id,
                            universal: true,
                        };
                        engine.register_event(None, chat_event.clone());
                        // send the new input to the server
                        action_events.write(NetworkAction(Action::RemoteEngineEvent(
                            engine.id,
                            chat_event,
                            engine.step_index,
                        )));
                        chat_state.current_msg = "".to_string();
                    }
                });
                // Send button
                if ui.button("Send").clicked() {
                    let chat_event = EngineEvent::ChatMessage {
                        id: rand::random(),
                        text: chat_state.current_msg.to_string(),
                        entity_id: player_entity_id,
                        universal: true,
                    };
                    engine.register_event(None, chat_event.clone());
                    // send the new input to the server
                    action_events.write(NetworkAction(Action::RemoteEngineEvent(
                        engine.id,
                        chat_event,
                        engine.step_index,
                    )));
                    chat_state.current_msg = "".to_string();
                }
            });
        });
}
