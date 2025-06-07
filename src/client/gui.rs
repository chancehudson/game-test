use bevy::prelude::*;

use crate::ActivePlayerState;

use super::network::NetworkAction;
use super::GameState;

pub struct GuiPlugin;

#[derive(Component)]
pub struct GuiWrapper;

#[derive(Component)]
pub struct UsernameLabel;

#[derive(Component)]
pub struct LogoutButton;

#[derive(Component)]
pub struct ExperienceBar;

#[derive(Component)]
pub struct HealthBar;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::OnMap), show_gui)
            .add_systems(
                Update,
                handle_logout_click.run_if(in_state(GameState::OnMap)),
            )
            .add_systems(OnExit(GameState::OnMap), remove_gui)
            .add_systems(
                Update,
                update_experience_bar.run_if(in_state(GameState::OnMap)),
            );
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
    if let Ok(interaction) = interaction_query.single() {
        if interaction == &Interaction::Pressed {
            action_events.send(NetworkAction(game_test::action::Action::LogoutPlayer));
            next_state.set(GameState::LoggedOut);
        }
    }
}

pub fn update_experience_bar(mut query: Query<&mut Node, With<ExperienceBar>>, time: Res<Time>) {
    if query.is_empty() {
        return;
    }
    let percent = time.elapsed_secs() % 100.0;
    if let Ok(mut node) = query.single_mut() {
        node.width = Val::Percent(percent);
    }
}

pub fn show_gui(
    mut commands: Commands,
    windows: Query<&Window>,
    active_player_state: Res<ActivePlayerState>,
) {
    if active_player_state.0.is_none() {
        println!("no active player state!");
        return;
    }
    let active_player_state = active_player_state.0.as_ref().unwrap();
    let window = windows.single().unwrap();
    let screen_width = window.resolution.width();
    let screen_height = window.resolution.height();
    const BAR_BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
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
                    parent.spawn((
                        Text::new(active_player_state.username.clone()),
                        UsernameLabel,
                    ));
                    parent.spawn((
                        Text::new("Level 1"),
                        TextFont {
                            font_size: 13.0,
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
                            BackgroundColor(BAR_BACKGROUND_COLOR),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                HealthBar,
                                Node {
                                    width: Val::Percent(50.0),
                                    position_type: PositionType::Absolute,
                                    top: Val::Px(0.0),
                                    bottom: Val::Px(0.0),
                                    left: Val::Px(0.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.0, 0.7, 0.0)),
                            ));
                            parent.spawn(Text::new("health"));
                        });
                    parent
                        .spawn((
                            Node {
                                height: Val::Px(20.0),
                                width: Val::Px(screen_width / 4.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(BAR_BACKGROUND_COLOR),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ExperienceBar,
                                Node {
                                    width: Val::Percent(50.0),
                                    position_type: PositionType::Absolute,
                                    top: Val::Px(0.0),
                                    bottom: Val::Px(0.0),
                                    left: Val::Px(0.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.5, 0.5, 1.0)),
                            ));
                            parent.spawn(Text::new("experience"));
                        });
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
