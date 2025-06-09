use std::collections::BTreeMap;

use bevy::dev_tools::fps_overlay::FpsOverlayConfig;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use bevy::text::FontSmoothing;

pub use game_test::action::Action;
use game_test::action::PlayerState;
pub use game_test::action::Response;
use game_test::engine::entity::EngineEntity;
use game_test::engine::GameEngine;
use game_test::engine::STEP_LEN_S;
use game_test::mob::SPRITE_MANIFEST;
use game_test::timestamp;
pub use game_test::MapData;

mod animated_sprite;
mod gui;
mod loading_screen;
mod login;
mod map;
mod map_data_loader;
mod mob;
mod sprite_data_loader;
// mod mob_health_bar;
mod network;
mod player;
mod smooth_camera;

use network::NetworkMessage;

use crate::map::MapEntity;
use crate::mob::MobComponent;
use crate::player::PlayerComponent;
use crate::sprite_data_loader::SpriteDataAsset;
use crate::sprite_data_loader::SpriteManager;

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    Disconnected,
    LoggedOut,
    LoadingMap,
    OnMap,
}

#[derive(Resource, Default)]
pub struct ActiveGameEngine(pub GameEngine);

#[derive(Component, Default)]
pub struct GameEntityComponent {
    entity_id: u128,
}

#[derive(Resource, Default)]
pub struct ActivePlayerEntityId(pub Option<u128>);

#[derive(Resource, Default)]
pub struct ActivePlayerState(pub Option<PlayerState>);

// Event for incoming messages
#[derive(Event, Debug)]
pub struct LoadSpriteRequest(pub u64);

fn main() {
    let mut app = App::new();
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(bevy_web_asset::WebAssetPlugin::default());
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: 12.0,
                    font: default(),
                    font_smoothing: FontSmoothing::default(),
                    ..default()
                },
                // We can also change color of the overlay
                text_color: Color::WHITE,
                enabled: true,
                ..default()
            },
        },
    ))
    .init_state::<GameState>()
    .init_resource::<SpriteManager>()
    .init_resource::<ActiveGameEngine>()
    .init_resource::<ActivePlayerEntityId>()
    .init_resource::<ActivePlayerState>()
    .add_event::<LoadSpriteRequest>()
    .add_plugins(loading_screen::LoadingScreenPlugin)
    .add_plugins(smooth_camera::SmoothCameraPlugin)
    .add_plugins(animated_sprite::AnimatedSpritePlugin)
    .add_plugins(map::MapPlugin)
    .add_plugins(map_data_loader::MapDataLoaderPlugin)
    .add_plugins(sprite_data_loader::SpriteDataLoaderPlugin)
    .add_plugins(login::LoginPlugin)
    .add_plugins(gui::GuiPlugin)
    .add_plugins(network::NetworkPlugin)
    .add_plugins(mob::MobPlugin)
    // .add_plugins(mob_health_bar::MobHealthBarPlugin)
    .add_systems(
        FixedUpdate,
        (handle_login, handle_player_entity_id, handle_engine_state),
    )
    .add_systems(
        Update,
        (
            load_sprite_manager,
            step_game_engine,
            sync_engine_components,
        )
            .run_if(in_state(GameState::OnMap)),
    )
    .add_plugins(player::PlayerPlugin);
    app.run();
}

fn load_sprite_manager(
    mut sprite_manager: ResMut<SpriteManager>,
    asset_server: Res<AssetServer>,
    sprite_data: Res<Assets<SpriteDataAsset>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    sprite_manager.continue_loading(&asset_server, &sprite_data, &mut texture_atlas_layouts);
}

fn step_game_engine(mut active_game_engine: ResMut<ActiveGameEngine>) {
    active_game_engine.0.step();
}

fn handle_player_entity_id(
    mut action_events: EventReader<NetworkMessage>,
    mut active_player_entity_id: ResMut<ActivePlayerEntityId>,
) {
    for event in action_events.read() {
        if let Response::PlayerEntityId(id) = &event.0 {
            println!("entity: {id}");
            active_player_entity_id.0 = Some(*id);
        }
    }
}

fn handle_engine_state(
    mut action_events: EventReader<NetworkMessage>,
    mut active_engine_state: ResMut<ActiveGameEngine>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
) {
    for event in action_events.read() {
        if let Response::EngineState(engine, server_step_index) = &event.0 {
            if active_player_entity_id.0.is_none() {
                println!("WARNING: received map state without an active entity id");
                return;
            }
            let player_entity_id = active_player_entity_id.0.unwrap();
            if !engine.entities.contains_key(&player_entity_id) {
                println!("WARNING: player is not in engine");
                return;
            }
            let mut engine = engine.clone();
            // println!("{} {}", active_engine_state.0.step_index, engine.step_index);
            // compute a local start timestamp
            engine.start_timestamp = timestamp() - STEP_LEN_S * (engine.step_index as f64);
            // treat engine like it's the latest point in time
            // copy our local player entity into the engine
            let local_inputs = active_engine_state
                .0
                .inputs
                .get(&player_entity_id)
                .cloned()
                .unwrap_or_else(|| BTreeMap::new());
            if let Some(local_entity) = active_engine_state.0.entities.get(&player_entity_id) {
                engine
                    .entities
                    .insert(player_entity_id, local_entity.clone());
            }
            engine.inputs.insert(player_entity_id, local_inputs.clone());
            active_engine_state.0 = engine;
        }
    }
}

fn sync_engine_components(
    mut commands: Commands,
    active_engine_state: Res<ActiveGameEngine>,
    mut entity_query: Query<(Entity, &GameEntityComponent, &mut Transform, &mut Sprite)>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut sprite_manager: ResMut<SpriteManager>,
    sprite_data: Res<Assets<SpriteDataAsset>>,
) {
    // TODO:::::::::::::
    use game_test::engine::entity::Entity;
    let mut entity_ids = active_engine_state.0.entities.clone();
    for (entity, entity_component, mut transform, mut sprite) in entity_query.iter_mut() {
        if let Some(game_entity) = active_engine_state
            .0
            .entities
            .get(&entity_component.entity_id)
        {
            transform.translation = game_entity.position().extend(0.0);
            if let Some(latest_input) = active_engine_state
                .0
                .latest_input(&entity_component.entity_id)
            {
                if latest_input.move_left {
                    sprite.flip_x = false;
                } else if latest_input.move_right {
                    sprite.flip_x = true;
                }
            }
            entity_ids.remove(&game_entity.id());
        } else {
            commands.entity(entity).despawn();
        }
    }
    // we're left with game entities we need to spawn
    for (id, game_entity) in entity_ids {
        match game_entity {
            EngineEntity::Player(p) => {
                if !sprite_manager.is_loaded(&0, &sprite_data) {
                    sprite_manager.load(0, &asset_server);
                    continue;
                }
                commands.spawn((
                    GameEntityComponent { entity_id: id },
                    Transform::from_translation(p.position().extend(10.0)),
                    PlayerComponent::default_sprite(sprite_manager.as_ref()),
                    MapEntity,
                ));
            }
            EngineEntity::MobSpawner(p) => {}
            EngineEntity::Mob(p) => {
                if !sprite_manager.is_loaded(&p.mob_type, &sprite_data) {
                    sprite_manager.load(p.mob_type, &asset_server);
                    continue;
                }
                commands.spawn((
                    GameEntityComponent { entity_id: id },
                    Transform::from_translation(p.position().extend(0.0)),
                    MobComponent::new(p, &sprite_data, sprite_manager.as_ref()),
                    MapEntity,
                ));
            }
            EngineEntity::Platform(p) => {
                println!("spawning platform");
                commands.spawn((
                    GameEntityComponent { entity_id: id },
                    Transform::from_translation(p.position().extend(0.0)),
                    MapEntity,
                    Sprite {
                        color: Color::srgb(0.0, 0.0, 1.0),
                        custom_size: Some(Vec2::new(p.size.x, p.size.y)),
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        ..default()
                    },
                ));
            }
        }
    }
}

fn handle_login(
    mut action_events: EventReader<NetworkMessage>,
    mut next_state: ResMut<NextState<GameState>>,
    mut active_player_state: ResMut<ActivePlayerState>,
) {
    for event in action_events.read() {
        if let Response::PlayerLoggedIn(state) = &event.0 {
            active_player_state.0 = Some(state.clone());
            next_state.set(GameState::LoadingMap);
        }
    }
}
