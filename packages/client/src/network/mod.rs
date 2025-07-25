use bevy::prelude::*;

use game_common::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
mod network_native;
#[cfg(target_arch = "wasm32")]
mod network_wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use network_native::NetworkConnection;
#[cfg(target_arch = "wasm32")]
pub use network_wasm::NetworkConnection;

use crate::GameState;

// Event for incoming messages
#[derive(Event, Debug)]
pub struct NetworkMessage(pub Response);

// Event for outgoing actions
#[derive(Event)]
pub struct NetworkAction(pub Action);

#[derive(Resource, Default)]
pub struct NetworkConnectionMaybe(pub Option<NetworkConnection>);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkConnectionMaybe>()
            .add_event::<NetworkMessage>()
            .add_event::<NetworkAction>()
            .add_systems(
                FixedUpdate,
                (send_system, receive_system).run_if(not(in_state(GameState::Disconnected))),
            );
    }
}

// System to handle sending messages
fn send_system(
    mut connection_maybe: ResMut<NetworkConnectionMaybe>,
    mut action_events: EventReader<NetworkAction>,
    mut next_state: ResMut<NextState<GameState>>,
    game_state: Res<State<GameState>>,
) {
    if game_state.get() == &GameState::Disconnected {
        return;
    }
    if let Some(connection) = &connection_maybe.0 {
        for action in action_events.read() {
            if connection.is_closed() {
                println!("Connection detected closed in send system");
                connection_maybe.0 = None;
                next_state.set(GameState::Disconnected);
                return;
            }
            connection.write_connection(action.0.clone());
        }
    } else {
        println!("WARNING: attempting to send network event without connection");
        next_state.set(GameState::Disconnected);
    }
}

// System to handle receiving messages
fn receive_system(
    mut connection_maybe: ResMut<NetworkConnectionMaybe>,
    mut message_events: EventWriter<NetworkMessage>,
    mut next_state: ResMut<NextState<GameState>>,
    game_state: Res<State<GameState>>,
) {
    if game_state.get() == &GameState::Disconnected {
        return;
    }
    if let Some(connection) = &connection_maybe.0 {
        if connection.is_closed() {
            println!("Connection detected closed in send system");
            connection_maybe.0 = None;
            next_state.set(GameState::Disconnected);
            return;
        }
        let messages = connection
            .read_connection()
            .iter()
            .cloned()
            .map(|v| NetworkMessage(v))
            .collect::<Vec<_>>();
        message_events.write_batch(messages);
    } else {
        println!("WARNING: attempting to receive network event without connection");
        next_state.set(GameState::Disconnected);
    }
}
