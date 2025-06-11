use bevy::prelude::*;
use bevy_simple_text_input::TextInput;
use bevy_simple_text_input::TextInputPlugin;
use bevy_simple_text_input::TextInputSubmitEvent;
use bevy_simple_text_input::TextInputValue;

use super::network::NetworkAction;
use crate::GameState;

pub struct LoginPlugin;

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LoggedOut), setup_login_screen)
            .add_systems(OnExit(GameState::LoggedOut), cleanup_login_screen)
            .add_plugins(TextInputPlugin)
            .add_systems(
                Update,
                handle_login_click.run_if(in_state(GameState::LoggedOut)),
            )
            .add_systems(
                Update,
                handle_signup_click.run_if(in_state(GameState::LoggedOut)),
            )
            .add_systems(Update, handle_enter.run_if(in_state(GameState::LoggedOut)));
    }
}

#[derive(Component)]
struct UsernameInput;

#[derive(Component)]
struct LoginButton;

#[derive(Component)]
struct SignupButton;

// Component to mark login UI entities
#[derive(Component)]
struct LoginUI;

fn handle_login_click(
    text_query: Query<&TextInputValue, With<UsernameInput>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoginButton>)>,
    mut action_events: EventWriter<NetworkAction>,
) {
    if interaction_query.is_empty() {
        return;
    }
    if let Ok(button) = interaction_query.single() {
        if let Ok(text) = text_query.single() {
            if button == &Interaction::Pressed {
                action_events.write(NetworkAction(game_test::action::Action::LoginPlayer(
                    text.0.clone(),
                )));
            }
        }
    }
}

fn handle_signup_click(
    text_query: Query<&TextInputValue, With<UsernameInput>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SignupButton>)>,
    mut action_events: EventWriter<NetworkAction>,
) {
    if interaction_query.is_empty() {
        return;
    }
    if let Ok(button) = interaction_query.single() {
        if let Ok(text) = text_query.single() {
            if button == &Interaction::Pressed {
                action_events.write(NetworkAction(game_test::action::Action::CreatePlayer(
                    text.0.clone(),
                )));
            }
        }
    }
}

fn handle_enter(
    mut enter_events: EventReader<TextInputSubmitEvent>,
    mut action_events: EventWriter<NetworkAction>,
) {
    for event in enter_events.read() {
        action_events.write(NetworkAction(game_test::action::Action::LoginPlayer(
            event.value.clone(),
        )));
    }
}

fn setup_login_screen(mut commands: Commands) {
    commands
        .spawn((
            LoginUI,
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(Text::new("username:"));
                    parent.spawn(Node {
                        width: Val::Px(8.),
                        ..default()
                    });
                    parent.spawn((
                        UsernameInput,
                        TextInput,
                        Text::new(""),
                        Node {
                            height: Val::Px(64.),
                            width: Val::Percent(80.),
                            border: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        BorderColor(Color::WHITE),
                        BorderRadius::MAX,
                    ));
                });
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Button,
                            LoginButton,
                            Node {
                                border: UiRect::all(Val::Px(2.0)),
                                // horizontally center child text
                                justify_content: JustifyContent::Center,
                                // vertically center child text
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(8.0)),
                                ..default()
                            },
                            BorderColor(Color::WHITE),
                            BorderRadius::MAX,
                        ))
                        .with_child((Text::new("Login"),));
                    parent
                        .spawn((
                            Button,
                            SignupButton,
                            Node {
                                border: UiRect::all(Val::Px(2.0)),
                                // horizontally center child text
                                justify_content: JustifyContent::Center,
                                // vertically center child text
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(8.0)),
                                ..default()
                            },
                            BorderColor(Color::WHITE),
                            BorderRadius::MAX,
                        ))
                        .with_child((Text::new("Signup"),));
                });
        });
}

fn cleanup_login_screen(mut commands: Commands, login_ui: Query<Entity, With<LoginUI>>) {
    // Remove all entities with LoginUI component
    for entity in &login_ui {
        commands.entity(entity).despawn();
    }
}
