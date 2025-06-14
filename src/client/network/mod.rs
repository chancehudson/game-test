use bevy::prelude::*;

use game_test::action::Action;
use game_test::action::Response;

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
    connection_maybe: Res<NetworkConnectionMaybe>,
    mut action_events: EventReader<NetworkAction>,
) {
    if let Some(connection) = &connection_maybe.0 {
        for action in action_events.read() {
            connection.write_connection(action.0.clone());
        }
    } else {
        println!("WARNING: attempting to send network event without connection");
    }
}

// System to handle receiving messages
fn receive_system(
    connection_maybe: Res<NetworkConnectionMaybe>,
    mut message_events: EventWriter<NetworkMessage>,
) {
    if let Some(connection) = &connection_maybe.0 {
        let messages = connection
            .read_connection()
            .iter()
            .cloned()
            .map(|v| NetworkMessage(v))
            .collect::<Vec<_>>();
        message_events.write_batch(messages);
    } else {
        println!("WARNING: attempting to receive network event without connection");
    }
}
