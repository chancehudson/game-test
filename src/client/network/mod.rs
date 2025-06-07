use bevy::prelude::*;

use game_test::action::Action;
use game_test::action::Response;

#[cfg(not(target_arch = "wasm32"))]
mod network_native;
#[cfg(target_arch = "wasm32")]
mod network_wasm;

#[cfg(not(target_arch = "wasm32"))]
use network_native::NetworkConnection;
#[cfg(target_arch = "wasm32")]
use network_wasm::NetworkConnection;

use crate::GameState;

// Event for incoming messages
#[derive(Event, Debug)]
pub struct NetworkMessage(pub Response);

// Event for outgoing actions
#[derive(Event)]
pub struct NetworkAction(pub Action);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NetworkMessage>()
            .add_event::<NetworkAction>()
            .add_systems(FixedUpdate, initialize_network)
            .add_systems(
                FixedUpdate,
                (send_system, receive_system).run_if(not(in_state(GameState::Disconnected))),
            );
    }
}

fn initialize_network(
    mut commands: Commands,
    query: Query<(Entity, &NetworkConnection)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok((entity, connection)) = query.get_single() {
        if connection.is_closed() {
            commands.entity(entity).despawn();
            next_state.set(GameState::Disconnected);
        }
    } else {
        let connection = NetworkConnection::attempt_connection("ws://127.0.0.1:1351".to_string());
        commands.spawn(connection);
        next_state.set(GameState::LoggedOut);
    }
}

// System to handle sending messages
fn send_system(query: Query<&NetworkConnection>, mut action_events: EventReader<NetworkAction>) {
    if let Ok(connection) = query.get_single() {
        for action in action_events.read() {
            connection.write_connection(action.0.clone());
        }
    }
}

// System to handle receiving messages
fn receive_system(
    query: Query<&NetworkConnection>,
    mut message_events: EventWriter<NetworkMessage>,
) {
    if query.is_empty() {
        println!("No network connection component in receive");
    }
    if let Ok(connection) = query.get_single() {
        let messages = connection
            .read_connection()
            .iter()
            .cloned()
            .map(|v| NetworkMessage(v))
            .collect::<Vec<_>>();
        message_events.send_batch(messages);
    }
}
