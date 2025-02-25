use bevy::prelude::*;

use super::network::NetworkAction;
use super::GameState;
use super::Player;

pub struct GuiPlugin;

#[derive(Component)]
pub struct GuiWrapper;

#[derive(Component)]
pub struct UsernameLabel;

#[derive(Component)]
pub struct LogoutButton;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::OnMap), show_gui)
            .add_systems(
                Update,
                handle_logout_click.run_if(in_state(GameState::OnMap)),
            )
            .add_systems(OnExit(GameState::OnMap), remove_gui);
    }
}

pub fn handle_logout_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LogoutButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut action_events: EventWriter<NetworkAction>,
) {
    if interaction_query.is_empty() {
        return;
    }
    let interaction = interaction_query.single();
    if interaction == &Interaction::Pressed {
        action_events.send(NetworkAction(game_test::action::Action::LogoutPlayer));
        next_state.set(GameState::LoggedOut);
    }
}

pub fn show_gui(mut commands: Commands, windows: Query<&Window>, player_query: Query<&Player>) {
    if player_query.is_empty() {
        println!("No player!");
        return;
    }
    let player = player_query.single();
    let window = windows.single();
    let screen_width = window.resolution.width();
    let screen_height = window.resolution.height();
    commands
        .spawn((
            GuiWrapper,
            BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
            Node {
                top: Val::Px(screen_height - 60.0),
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::SpaceAround,
                padding: UiRect {
                    left: Val::Px(4.0),
                    right: Val::Px(4.0),
                    top: Val::Px(4.0),
                    bottom: Val::Px(4.0),
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // username and level column display
            parent
                .spawn(Node {
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceAround,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((Text::new(player.username.clone()), UsernameLabel));
                    parent.spawn((
                        Text::new("Level 1"),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                    ));
                });
            // health and experience display
            parent
                .spawn(Node {
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceAround,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                height: Val::Px(20.0),
                                width: Val::Px(screen_width / 4.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.0, 0.7, 0.0)),
                        ))
                        .with_child(Text::new("health"));
                    parent
                        .spawn((
                            Node {
                                height: Val::Px(20.0),
                                width: Val::Px(screen_width / 4.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.5, 0.5, 1.0)),
                        ))
                        .with_child(Text::new("experience"));
                });
            // buttons
            parent
                .spawn(Node {
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceAround,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Button,
                            LogoutButton,
                            Node {
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(8.0)),
                                ..default()
                            },
                            BorderColor(Color::WHITE),
                            BorderRadius::MAX,
                        ))
                        .with_child(Text::new("Logout"));
                });
        });
}

pub fn remove_gui(mut commands: Commands, query: Query<Entity, With<GuiWrapper>>) {
    for wrapper in query.iter() {
        commands.entity(wrapper).despawn_recursive();
    }
}
