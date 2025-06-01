use bevy::dev_tools::fps_overlay::FpsOverlayConfig;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy::utils::HashMap;

pub use game_test::action::Action;
pub use game_test::action::PlayerAction;
use game_test::action::PlayerState;
pub use game_test::action::Response;
pub use game_test::actor::move_x;
pub use game_test::actor::move_y;
pub use game_test::Actor;
pub use game_test::MapData;

mod animated_sprite;
mod gui;
mod loading_screen;
mod login;
mod map;
mod map_data_loader;
mod mob;
mod mob_health_bar;
mod network;
mod player;
mod smooth_camera;

use map::ActiveMap;
use map::MapEntity;
use mob::MobEntity;
use mob::MobRegistry;
use mob_health_bar::MobHealthBar;
use network::NetworkMessage;
use network::NetworkPlugin;
use player::ActivePlayer;
use player::Player;

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    LoggedOut,
    LoadingMap,
    OnMap,
}

#[derive(Resource, Default)]
pub struct ActivePlayerState(pub PlayerState);

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: 12.0,
                    font: default(),
                    font_smoothing: FontSmoothing::default(),
                },
                // We can also change color of the overlay
                text_color: Color::WHITE,
                enabled: true,
            },
        },
    ))
    .init_state::<GameState>()
    .init_resource::<ActivePlayerState>()
    .add_plugins(loading_screen::LoadingScreenPlugin)
    .add_plugins(smooth_camera::SmoothCameraPlugin)
    .add_plugins(animated_sprite::AnimatedSpritePlugin)
    .add_plugins(map::MapPlugin)
    .add_plugins(map_data_loader::MapDataLoaderPlugin)
    .add_plugins(login::LoginPlugin)
    .add_plugins(gui::GuiPlugin)
    .add_plugins(NetworkPlugin)
    .add_plugins(player::PlayerPlugin)
    .add_plugins(mob::MobPlugin)
    .add_plugins(mob_health_bar::MobHealthBarPlugin)
    .add_systems(FixedUpdate, response_handler_system)
    .add_systems(FixedUpdate, handle_login)
    .add_systems(FixedUpdate, handle_map_state)
    .add_systems(Update, step_mobs.run_if(in_state(GameState::OnMap)));
    app.run();
}

fn step_mobs(
    mut mobs: Query<(&mut MobEntity, &mut Transform)>,
    // players: Query<(&Player, &Transform), Without<MobEntity>>,
    time: Res<Time>,
    active_map: Res<ActiveMap>,
) {
    let delta = time.delta_secs();
    let map_data = active_map.data.as_ref().unwrap();
    for (mut mob, mut transform) in &mut mobs {
        mob.step(delta, map_data);
        transform.translation.x = mob.mob.position.x;
        transform.translation.y = mob.mob.position.y;
    }
}

fn response_handler_system(
    mut action_events: EventReader<NetworkMessage>,
    mut active_map: ResMut<ActiveMap>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in action_events.read() {
        match &event.0 {
            Response::ChangeMap(new_map) => {
                active_map.name = new_map.clone();
                next_state.set(GameState::LoadingMap);
            }
            Response::LoginError(err) => {
                println!("Error logging in: {err}");
            }
            Response::Log(msg) => {
                println!("Server message: {msg}");
            }
            _ => {}
        }
    }
}

fn handle_map_state(
    mut mob_registry: ResMut<MobRegistry>,
    mut action_events: EventReader<NetworkMessage>,
    mut commands: Commands,
    // mut mob_query: Query<(Entity, &mut MobEntity, &mut Transform)>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for event in action_events.read() {
        if let Response::MapState(mobs) = &event.0 {
            for mob in mobs {
                if mob_registry.mobs.get(&mob.id).is_some() {
                    // update the mob state if needed
                    // currently do nothing
                } else {
                    // insert the mob
                    println!("spawned entity {} with health {}", mob.id, mob.health);
                    let mut entity = commands.spawn((
                        MapEntity,
                        Transform::from_translation(Vec3::new(mob.position.x, mob.position.y, 1.0)),
                        MobEntity::new(mob.clone(), &asset_server, &mut texture_atlas_layouts),
                    ));
                    entity.with_child(MobHealthBar::new(mob.clone()));
                    mob_registry.mobs.insert(mob.id, entity.id());
                }
            }
        }
    }
}

fn handle_login(
    mut commands: Commands,
    mut action_events: EventReader<NetworkMessage>,
    mut active_player: Query<(&mut Player, &mut Transform), With<ActivePlayer>>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    mut active_map: ResMut<ActiveMap>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut active_player_state: ResMut<ActivePlayerState>,
) {
    for event in action_events.read() {
        if let Response::PlayerLoggedIn(state, body) = &event.0 {
            active_map.name = state.current_map.clone();
            next_state.set(GameState::LoadingMap);
            active_player_state.0 = state.clone();

            if active_player.is_empty() {
                commands.spawn((
                    ActivePlayer,
                    Player {
                        id: state.id.clone(),
                        username: state.username.clone(),
                        current_map: state.current_map.clone(),
                        body: body.clone(),
                    },
                    Transform::from_translation(Vec3::new(body.position.x, body.position.y, 1.0)),
                    Player::default_sprite(&asset_server, &mut texture_atlas_layouts),
                ));
            } else {
                let (mut player, mut transform) = active_player.single_mut();
                player.id = state.id.clone();
                player.username = state.username.clone();
                player.current_map = state.current_map.clone();
                player.body = body.clone();
                transform.translation.x = body.position.x;
                transform.translation.y = body.position.y;
            }
        }
    }
}
