use bevy::prelude::*;
use bevy_egui::egui::{Color32, RichText};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};
use game_test::action::{Action, Response};
use game_test::timestamp;
use web_time::Duration;
use web_time::Instant;

use crate::network::{NetworkConnectionMaybe, NetworkMessage};
use crate::{ActivePlayerEntityId, LoggedInAt};

use super::GameState;
use super::network::NetworkConnection;

pub struct GuiPlugin;

// A player connects to a server
#[derive(Resource)]
pub struct ConnectViewState {
    server_url: String,
    attempting_connection: bool,
    error: Option<String>,
    requested_initial_focus: bool,
    began_connecting: Option<Instant>,
}

// A player authenticates with a server
#[derive(Resource, Default)]
pub struct LoginViewState {
    username: String,
    requested_initial_focus: bool,
    error: Option<String>,
}

impl Default for ConnectViewState {
    fn default() -> Self {
        Self {
            server_url: "ws://127.0.0.1:1351".to_string(),
            attempting_connection: false,
            error: None,
            requested_initial_focus: false,
            began_connecting: None,
        }
    }
}

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .init_resource::<ConnectViewState>()
        .init_resource::<LoginViewState>()
        .add_systems(FixedUpdate, (handle_login_error,))
        .add_systems(
            EguiContextPass,
            (
                connect_view,
                login_view,
                spawn_howto_view,
                playtest_info_view,
            ),
        );
    }
}

fn spawn_howto_view(
    mut contexts: EguiContexts,
    game_state: Res<State<GameState>>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
    logged_in_at: Res<LoggedInAt>,
) {
    if game_state.get() != &GameState::OnMap || active_player_entity_id.0.is_some() {
        return;
    }
    // spawn 6 seconds after logging in if no entity is spawned
    if timestamp() - logged_in_at.0 > 6. {
        egui::Window::new("Spawn Howto!")
            .resizable(false)
            .collapsible(false)
            .show(contexts.ctx_mut(), |ui| {
                ui.label("Press j to spawn");
                ui.add_space(10.);
                ui.label("lbh qhzo vqvbg");
            });
    }
}

fn playtest_info_view(mut contexts: EguiContexts, game_state: Res<State<GameState>>) {
    if game_state.get() != &GameState::Disconnected {
        return;
    }
    egui::Window::new("Playtest info!")
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label("hello if you're here for the playtest press this button");
            ui.add_enabled_ui(false, |ui| {
                if ui.button("connect").clicked() {
                    // connect_view_state.server_url = "wss://playtest.keccak-doomsday.com/".to_string();
                }
            });
            ui.label("(the playtest server isn't running yet)");
        });
}

fn connect_view(
    mut contexts: EguiContexts,
    game_state: Res<State<GameState>>,
    mut connect_view_state: ResMut<ConnectViewState>,
    mut connection_maybe: ResMut<NetworkConnectionMaybe>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if game_state.get() != &GameState::Disconnected {
        return;
    }
    egui::Window::new("Welcome!")
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

            ui.label("Hello! Welcome to the first public test of codename keccak_doomsday.");
            ui.add_space(10.);
            ui.label("keccak_doomsday is a seeded deterministic 2d game engine synchronized across many players in realtime.");
            ui.add_space(10.);

            ui.heading("Connect to server");
            ui.add_space(10.);
            let url_edit = ui.text_edit_singleline(&mut connect_view_state.server_url);
            if !connect_view_state.requested_initial_focus {
                connect_view_state.requested_initial_focus = true;
                url_edit.request_focus();
            }
            ui.add_space(10.);
            // render connect text field/button
            if connect_view_state.attempting_connection {
                ui.label("Connecting...");
                if let Some(connection) = &connection_maybe.0 {
                    if let Err(e) = connection.is_open() {
                        connect_view_state.error = Some(e.to_string());
                        connect_view_state.attempting_connection = false;
                        connection_maybe.0 = None;
                    } else {
                        // successful connection, render loop ends
                        *connect_view_state = ConnectViewState::default();
                        next_state.set(GameState::LoggedOut);
                    }
                } else {
                    connect_view_state.attempting_connection = false;
                    connection_maybe.0 = None;
                }
            } else {
                if ui.button("Connect!").clicked() || enter_pressed{
                    // handle join click
                    connect_view_state.attempting_connection = true;
                    connect_view_state.began_connecting = Some(Instant::now());
                    let connection = NetworkConnection::attempt_connection(connect_view_state.server_url.clone());
                    connection_maybe.0 = Some(connection);
                }
            }
            // render error message
            if let Some(msg) = connect_view_state.error.clone() {
                ui.add_space(10.);
                let error_label = RichText::new(msg)
                .color(Color32::RED);
                ui.label(error_label);
            }
        });
}

fn handle_login_error(
    mut action_events: EventReader<NetworkMessage>,
    mut login_state: ResMut<LoginViewState>,
) {
    for event in action_events.read() {
        if let Response::LoginError(e) = &event.0 {
            login_state.error = Some(e.clone());
        }
    }
}

fn login_view(
    mut contexts: EguiContexts,
    game_state: Res<State<GameState>>,
    mut login_state: ResMut<LoginViewState>,
    connection_maybe: Res<NetworkConnectionMaybe>,
) {
    if game_state.get() != &GameState::LoggedOut {
        return;
    }

    egui::Window::new("Connected!")
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            if connection_maybe.0.is_none() {
                ui.add_space(10.);
                ui.label(
                    RichText::new("connection_maybe is None!".to_string()).color(Color32::RED),
                );
                return;
            }
            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let connection = connection_maybe.0.as_ref().unwrap();
            ui.heading("Congrats, you connected to a server!");
            ui.add_space(10.);
            // TODO: custom server welcome messages

            // username selection
            ui.label("Type a name for your character in the engine");
            ui.add_space(10.);
            let username_edit = ui.text_edit_singleline(&mut login_state.username);
            if !login_state.requested_initial_focus {
                login_state.requested_initial_focus = true;
                username_edit.request_focus();
            }
            if ui.button("Login").clicked() || enter_pressed {
                connection.write_connection(Action::LoginPlayer(login_state.username.clone()));
            }
            // render error message
            if let Some(msg) = login_state.error.clone() {
                ui.add_space(10.);
                let error_label = RichText::new(msg).color(Color32::RED);
                ui.label(error_label);
            }
            /*
            ui.heading("Character info:");
            ui.label("Accuracy: 10");
            ui.label("x velocity limit: -250, 250");
            ui.label("y velocity limit: -350, 700");
            ui.label("Jump weightless frames: 4");
            ui.label("Can jumpdash_basic: false");
            ui.label("Can shoot: true");
            ui.label("Can spawn entities: false");
            ui.label("Can spawn portals: false");
            ui.label("Can spawn self: true");
            ui.label("Can despawn self: true");
            ui.label("Can rewind universal engine: false");
             */
        });
}
