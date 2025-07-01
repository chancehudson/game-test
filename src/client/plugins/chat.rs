use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use bevy_lunex::prelude::*;
use bevy_lunex::{UiFetchFromCamera, UiLayout, UiLayoutRoot};
use bevy_simple_text_input::{TextInput, TextInputSubmitEvent};
use game_test::{action::Action, engine::game_event::EngineEvent};

use crate::InputFocus;
use crate::{
    network::NetworkAction,
    plugins::engine::{ActiveGameEngine, ActivePlayerEntityId},
};

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_enter.run_if(in_state(InputFocus::Chat)))
            .add_systems(OnEnter(InputFocus::Chat), input_field)
            .add_systems(OnExit(InputFocus::Chat), remove_input_field);
    }
}

fn handle_enter(
    mut enter_events: EventReader<TextInputSubmitEvent>,
    mut action_events: EventWriter<NetworkAction>,
    mut active_engine: ResMut<ActiveGameEngine>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
    mut next_state: ResMut<NextState<InputFocus>>,
) {
    if active_player_entity_id.0.is_none() {
        return;
    }
    let engine = &mut active_engine.0;
    let player_entity_id = active_player_entity_id.0.unwrap();
    for event in enter_events.read() {
        let chat_event = EngineEvent::ChatMessage {
            id: rand::random(),
            text: event.value.clone(),
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
        next_state.set(InputFocus::Game);
    }
}

fn remove_input_field(mut commands: Commands, query: Query<(Entity, &TextInput)>) {
    for (entity, _) in query {
        commands.entity(entity).despawn();
    }
}

fn input_field(mut commands: Commands) {
    commands.spawn((
        Node {
            top: Val::Percent(50.0),
            ..default()
        },
        BackgroundColor(Color::linear_rgba(0., 0., 0., 0.4)),
        TextInput,
    ));
}
