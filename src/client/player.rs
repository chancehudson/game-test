use bevy::prelude::*;

use game_test::action::PlayerAction;
use game_test::action::PlayerBody;
use game_test::action::Response;
use game_test::actor::move_x;
use game_test::actor::move_y;

use crate::animated_sprite::AnimatedSprite;

use super::map::ActiveMap;
use super::map::MapEntity;
use super::network::NetworkAction;
use super::network::NetworkMessage;
use super::GameState;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct ActivePlayer;

#[derive(Component)]
pub struct Player {
    pub id: String,
    pub username: String,
    pub current_map: String,
    pub body: PlayerBody, // data necessary to render the player
}

impl Player {
    pub fn default_sprite(
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) -> (AnimatedSprite, Sprite) {
        let texture = asset_server.load("banana.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::splat(52), 2, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        (
            AnimatedSprite {
                fps: 2,
                frame_count: 2,
                time: 0.0,
            },
            Sprite {
                image: texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                }),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
        )
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_player_state)
            .add_systems(Update, flip_player)
            .add_systems(
                Update,
                (input_system, step_physics, step_movement)
                    .chain()
                    .run_if(in_state(GameState::OnMap)),
            );
    }
}

fn handle_player_state(
    mut action_events: EventReader<NetworkMessage>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player, &mut Transform)>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for event in action_events.read() {
        if let Response::PlayerRemoved(id) = &event.0 {
            for (entity, player, _) in player_query.iter_mut() {
                if &player.id == id {
                    commands.entity(entity).despawn();
                    return;
                }
            }
            println!("Received remove for unknown player: {}", id);
        }
        if let Response::PlayerChange(body) = &event.0 {
            for (_entity, mut existing_player, mut transform) in player_query.iter_mut() {
                if &existing_player.id != &body.id {
                    continue;
                }
                existing_player.body = body.clone();
                transform.translation.x = body.position.x;
                transform.translation.y = body.position.y;
                return;
            }
            println!("Received update for unknown player: {}", body.id);
        }
        if let Response::PlayerData(state, body) = &event.0 {
            for (_entity, mut existing_player, mut transform) in player_query.iter_mut() {
                if &existing_player.id != &state.id {
                    continue;
                }
                existing_player.body = body.clone();
                transform.translation.x = body.position.x;
                transform.translation.y = body.position.y;
                return;
            }
            commands.spawn((
                MapEntity,
                Player {
                    id: state.id.clone(),
                    username: state.username.clone(),
                    current_map: state.current_map.clone(),
                    body: body.clone(),
                },
                Transform::from_translation(Vec3::new(body.position.x, body.position.y, 0.0)),
                Player::default_sprite(&asset_server, &mut texture_atlas_layouts),
            ));
        }
    }
}

fn flip_player(mut query: Query<(&Player, &mut Sprite)>) {
    for (player, mut sprite) in query.iter_mut() {
        if player.body.velocity.x > 0.0 {
            sprite.flip_x = true;
        } else if player.body.velocity.x < 0.0 {
            sprite.flip_x = false;
        }
    }
}

fn input_system(
    mut query: Query<(&mut Player, &Transform), With<ActivePlayer>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action_events: EventWriter<NetworkAction>,
) {
    if query.is_empty() {
        return;
    }
    let (mut active_player, transform) = query.single_mut();
    let player_action = PlayerAction {
        move_left: keyboard.pressed(KeyCode::ArrowLeft),
        move_right: keyboard.pressed(KeyCode::ArrowRight),
        jump: keyboard.just_pressed(KeyCode::Space) && !keyboard.pressed(KeyCode::ArrowDown),
        downward_jump: keyboard.pressed(KeyCode::ArrowDown)
            && keyboard.just_pressed(KeyCode::Space),
        enter_portal: keyboard.just_pressed(KeyCode::ArrowUp),
        ..default()
    };
    if active_player.body.action.as_ref() != Some(&player_action) {
        action_events.send(NetworkAction(game_test::action::Action::SetPlayerAction(
            player_action.clone(),
            Vec2::new(transform.translation.x, transform.translation.y),
            active_player.body.velocity,
        )));
    }
    active_player.body.action = Some(player_action);
}

fn step_physics(mut query: Query<(&mut Player, &mut Transform)>, time: Res<Time>) {
    let delta = time.delta_secs();
    for (mut player, mut transform) in query.iter_mut() {
        let action = player.body.action.clone();
        if action.is_none() {
            continue;
        }
        let action = action.unwrap();
        let (new_position, velocity, out_action) = action.step_action_raw(
            Vec2::new(transform.translation.x, transform.translation.y),
            player.body.velocity,
            delta,
        );
        player.body.action = Some(out_action);
        player.body.velocity = Vec2::new(velocity.x, velocity.y);
        player.body.velocity.y += -game_test::actor::GRAVITY_ACCEL * delta;
        transform.translation.x = new_position.x;
        transform.translation.y = new_position.y;
    }
}

fn step_movement(
    mut players: Query<(&mut Player, &mut Transform)>,
    time: Res<Time>,
    active_map: Res<ActiveMap>,
) {
    let delta = time.delta_secs();
    let map_data = active_map.data.as_ref().unwrap();
    for (mut player, mut transform) in players.iter_mut() {
        let (new_x, new_vel_x) = move_x(
            Rect::new(
                transform.translation.x,
                transform.translation.y,
                transform.translation.x + player.body.size.x,
                transform.translation.y + player.body.size.y,
            ),
            player.body.velocity,
            player.body.velocity.x * delta,
            map_data,
        );
        let (new_y, new_vel_y) = move_y(
            Rect::new(
                new_x,
                transform.translation.y,
                new_x + player.body.size.x,
                transform.translation.y + player.body.size.y,
            ),
            Vec2::new(new_vel_x, player.body.velocity.y),
            player.body.velocity.y * delta,
            map_data,
        );
        transform.translation.x = new_x;
        transform.translation.y = new_y;
        player.body.velocity.x = new_vel_x;
        player.body.velocity.y = new_vel_y;
    }
}
