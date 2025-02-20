use bevy::prelude::*;

use websocket::client::ClientBuilder;
use websocket::ws::dataframe::DataFrame;
use websocket::OwnedMessage;

use game_test::action::Action;
use game_test::action::Response;

#[derive(Resource)]
pub struct NetworkConnection {
    sender: Option<websocket::sender::Writer<std::net::TcpStream>>,
    receiver: Option<flume::Receiver<OwnedMessage>>,
    _read_thread: Option<std::thread::JoinHandle<()>>,
}

// Resource for connection status
#[derive(Resource)]
pub struct ConnectionStatus {
    is_connected: bool,
    last_error: Option<String>,
}

// Event for incoming messages
#[derive(Event)]
pub struct NetworkMessage(pub Response);

// Event for outgoing actions
#[derive(Event)]
pub struct NetworkAction(pub Action);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkConnection>()
            .init_resource::<ConnectionStatus>()
            .add_event::<NetworkMessage>()
            .add_event::<NetworkAction>()
            .add_systems(Update, (connect_system, send_system, receive_system));
    }
}

impl Default for NetworkConnection {
    fn default() -> Self {
        Self {
            sender: None,
            receiver: None,
            _read_thread: None,
        }
    }
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self {
            is_connected: false,
            last_error: None,
        }
    }
}

// System to establish connection
fn connect_system(mut net_conn: ResMut<NetworkConnection>, mut status: ResMut<ConnectionStatus>) {
    if !status.is_connected && net_conn.sender.is_none() {
        match establish_connection("ws://127.0.0.1:1351") {
            Ok((sender, receiver, thread)) => {
                println!("Connected!");
                net_conn.sender = Some(sender);
                net_conn.receiver = Some(receiver);
                net_conn._read_thread = Some(thread);
                status.is_connected = true;
                status.last_error = None;
            }
            Err(e) => {
                println!("Error establishing connection! {}", e);
                status.last_error = Some(e.to_string());
            }
        }
    }
}

// System to handle sending messages
fn send_system(
    mut net_conn: ResMut<NetworkConnection>,
    mut status: ResMut<ConnectionStatus>,
    mut action_events: EventReader<NetworkAction>,
) {
    if let Some(sender) = &mut net_conn.sender {
        for action in action_events.read() {
            match bincode::serialize(&action.0) {
                Ok(data) => {
                    if let Err(e) = sender.send_message(&OwnedMessage::Binary(data)) {
                        status.is_connected = false;
                        status.last_error = Some(e.to_string());
                        break;
                    }
                }
                Err(e) => {
                    status.last_error = Some(e.to_string());
                }
            }
        }
    }
}

// System to handle receiving messages
fn receive_system(
    net_conn: Res<NetworkConnection>,
    mut status: ResMut<ConnectionStatus>,
    mut message_events: EventWriter<NetworkMessage>,
) {
    if let Some(receiver) = &net_conn.receiver {
        match receiver.try_recv() {
            Ok(msg) => {
                if msg.is_data() {
                    match bincode::deserialize::<Response>(&msg.take_payload()) {
                        Ok(response) => {
                            message_events.send(NetworkMessage(response));
                        }
                        Err(e) => {
                            status.last_error = Some(e.to_string());
                        }
                    }
                }
            }
            Err(e) => match e {
                flume::TryRecvError::Disconnected => {
                    status.is_connected = false;
                    status.last_error = Some("Connection lost".to_string());
                }
                flume::TryRecvError::Empty => {}
            },
        }
    }
}

// Helper function to establish connection
fn establish_connection(
    url: &str,
) -> anyhow::Result<(
    websocket::sender::Writer<std::net::TcpStream>,
    flume::Receiver<OwnedMessage>,
    std::thread::JoinHandle<()>,
)> {
    let client = ClientBuilder::new(url)?.connect_insecure()?;
    let (mut receiver, sender) = client.split()?;
    let (msg_tx, msg_rx) = flume::unbounded();

    let read_thread = std::thread::spawn(move || {
        for message in receiver.incoming_messages() {
            match message {
                Ok(msg) => {
                    if msg_tx.send(msg).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok((sender, msg_rx, read_thread))
}
