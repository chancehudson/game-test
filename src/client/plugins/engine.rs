use bevy::prelude::*;

use crate::Action;
use crate::GameEngine;
use crate::GameState;
use crate::NetworkMessage;
use crate::PlayerState;
use crate::Response;
use crate::SpriteDataAsset;
use crate::SpriteManager;
use crate::map::MapEntity;
use crate::map_data_loader::MapDataAsset;
use crate::mob::MobComponent;
use crate::network::NetworkAction;
use crate::player::PlayerComponent;
use crate::plugins::engine_sync::EngineSyncInfo;
use crate::plugins::smooth_camera::CameraMovement;

use game_test::engine::STEP_LEN_S;
use game_test::engine::entity::EEntity;
use game_test::engine::entity::EngineEntity;
use game_test::engine::entity::player::PlayerEntity;
use game_test::engine::game_event::GameEvent;
use game_test::timestamp;

/// Engine tracking resources/components
///
#[derive(Resource, Default)]
pub struct ActiveGameEngine(pub GameEngine, pub u64);

#[derive(Component, Default)]
pub struct GameEntityComponent {
    pub entity_id: u128,
}

#[derive(Resource, Default)]
pub struct ActivePlayerEntityId(pub Option<u128>);

#[derive(Resource, Default)]
pub struct LoggedInAt(pub f64);

#[derive(Resource, Default)]
pub struct ActivePlayerState(pub Option<PlayerState>);

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        println!("building this piece of shit");
        app.init_resource::<ActiveGameEngine>()
            .init_resource::<ActivePlayerEntityId>()
            .init_resource::<ActivePlayerState>()
            .init_resource::<LoggedInAt>()
            .add_systems(
                Update,
                (
                    handle_engine_event,
                    step_game_engine,
                    sync_engine_components,
                )
                    .chain()
                    .run_if(in_state(crate::GameState::OnMap)),
            )
            .add_systems(
                FixedUpdate,
                (
                    handle_login,
                    handle_exit_map,
                    handle_engine_state,
                    handle_player_state,
                    handle_engine_stats,
                ),
            );
    }
}

fn handle_login(
    mut action_events: EventReader<NetworkMessage>,
    mut active_player_entity_id: ResMut<ActivePlayerEntityId>,
    mut logged_in_at: ResMut<LoggedInAt>,
) {
    for event in action_events.read() {
        if let Response::PlayerLoggedIn(_state) = &event.0 {
            active_player_entity_id.0 = None;
            logged_in_at.0 = timestamp();
        }
    }
}

fn handle_player_state(
    mut action_events: EventReader<NetworkMessage>,
    mut active_player_state: ResMut<ActivePlayerState>,
) {
    for event in action_events.read() {
        if let Response::PlayerState(state) = &event.0 {
            active_player_state.0 = Some(state.clone());
        }
    }
}

fn step_game_engine(mut active_game_engine: ResMut<ActiveGameEngine>) {
    let engine = &mut active_game_engine.0;
    let expected = engine.expected_step_index();
    if expected <= engine.step_index {
        engine.step();
    } else {
        let step_count = expected - engine.step_index;
        for _ in 0..step_count {
            engine.step();
        }
    }
}

fn handle_exit_map(
    mut action_events: EventReader<NetworkMessage>,
    query: Query<Entity, With<MapEntity>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in action_events.read() {
        if let Response::PlayerExitMap(_from_map) = &event.0 {
            // despawn everything??
            // TODO: check that from_map is the current map
            for entity in query {
                commands.entity(entity).despawn();
            }
            next_state.set(GameState::Waiting);
        }
    }
}

fn handle_engine_event(
    mut action_events: EventReader<NetworkMessage>,
    mut active_engine_state: ResMut<ActiveGameEngine>,
) {
    for event in action_events.read() {
        match &event.0 {
            Response::EngineEvents(engine_id, events) => {
                let engine = &mut active_engine_state.0;
                if engine.id != *engine_id {
                    continue;
                }
                engine.integrate_events(events.clone());
            }
            _ => {}
        }
    }
}

fn handle_engine_stats(
    mut action_events: EventReader<NetworkMessage>,
    mut engine_sync: ResMut<EngineSyncInfo>,
    active_engine_state: Res<ActiveGameEngine>,
) {
    let engine = &active_engine_state.0;
    for event in action_events.read() {
        if let Response::EngineStats(step_index) = &event.0 {
            engine_sync.server_step = *step_index;
            engine_sync.sync_distance = (engine.step_index as i64) - (*step_index as i64);
        }
    }
}

fn handle_engine_state(
    mut action_events: EventReader<NetworkMessage>,
    mut action_events_write: EventWriter<NetworkAction>,
    mut active_engine_state: ResMut<ActiveGameEngine>,
    mut next_state: ResMut<NextState<GameState>>,
    mut active_player_entity_id: ResMut<ActivePlayerEntityId>,
    active_player_state: Res<ActivePlayerState>,
    mut camera_query: Query<(&mut Transform, &mut CameraMovement), With<Camera2d>>,
    map_loader: Res<crate::map::MapLoader>,
    map_assets: Res<Assets<MapDataAsset>>,
    windows: Query<&Window>,
) {
    for event in action_events.read() {
        if let Response::EngineState(engine) = &event.0 {
            println!("INFO: Received engine with id: {}", engine.id);
            // TODO: figure out how to get rid of this clone
            let is_map_change = active_engine_state.0.map.name != engine.map.name;
            active_engine_state.0 = engine.clone();
            let engine = &mut active_engine_state.0;
            // approximate locally
            engine.start_timestamp = timestamp() - ((engine.step_index as f64) * STEP_LEN_S);

            if is_map_change {
                if let Some(active_entity_id) = &mut active_player_entity_id.0 {
                    if let Some(player_state) = &active_player_state.0 {
                        let mut entity =
                            PlayerEntity::new_with_ids(rand::random(), player_state.id.clone());
                        entity.is_active = true;
                        entity.position = engine.map.spawn_location;
                        if let Ok((mut camera_transform, _)) = camera_query.single_mut() {
                            camera_transform.translation = entity.position_f32().extend(0.0);
                        }
                        // TODO: move this to camera
                        crate::plugins::smooth_camera::snap_to_position(
                            &mut camera_query,
                            &map_loader,
                            &map_assets,
                            windows,
                            true,
                        );
                        *active_entity_id = entity.id;
                        let spawn_event = GameEvent::SpawnEntity {
                            id: rand::random(),
                            entity: EngineEntity::Player(entity),
                            universal: true,
                        };
                        // register the event locally
                        engine.register_event(None, spawn_event.clone());
                        // send the new input to the server
                        action_events_write.write(NetworkAction(Action::EngineEvent(
                            engine.id,
                            spawn_event,
                            engine.step_index,
                        )));
                    }
                }
            }
            next_state.set(GameState::LoadingMap);
        }
    }
}

fn sync_engine_components(
    mut commands: Commands,
    active_engine_state: Res<ActiveGameEngine>,
    mut entity_query: Query<(Entity, &GameEntityComponent, &mut Transform, &mut Sprite)>,
    asset_server: Res<AssetServer>,
    mut sprite_manager: ResMut<SpriteManager>,
    sprite_data: Res<Assets<SpriteDataAsset>>,
) {
    let engine = &active_engine_state.0;
    let mut to_spawn = engine.entities.clone();
    for (entity, entity_component, mut transform, mut sprite) in entity_query.iter_mut() {
        if let Some(game_entity) = engine.entities.get(&entity_component.entity_id) {
            transform.translation = game_entity.position_f32().extend(transform.translation.z);
            match game_entity {
                EngineEntity::Player(p) => {
                    sprite.flip_x = !p.facing_left;
                }
                EngineEntity::Mob(p) => {
                    if p.velocity.x < 0 {
                        sprite.flip_x = false;
                    }
                    if p.velocity.x > 0 {
                        sprite.flip_x = true;
                    }
                }
                _ => {}
            }
            to_spawn.remove(&game_entity.id());
        } else {
            commands.entity(entity).despawn();
        }
    }
    // we're left with game entities we need to spawn
    for (id, game_entity) in to_spawn {
        match game_entity {
            EngineEntity::Player(p) => {
                if !sprite_manager.is_loaded(&0, &sprite_data) {
                    sprite_manager.load(0, &asset_server);
                    continue;
                }
                commands.spawn((
                    GameEntityComponent { entity_id: id },
                    Transform::from_translation(p.position_f32().extend(100.0)),
                    PlayerComponent::default_sprite(sprite_manager.as_ref()),
                    MapEntity,
                ));
            }
            EngineEntity::MobSpawner(_) => {}
            EngineEntity::Mob(p) => {
                if !sprite_manager.is_loaded(&p.mob_type, &sprite_data) {
                    sprite_manager.load(p.mob_type, &asset_server);
                    continue;
                }
                commands.spawn((
                    GameEntityComponent { entity_id: id },
                    Transform::from_translation(p.position_f32().extend(1.0)),
                    Text2d(p.id.to_string().split_off(15)),
                    TextFont {
                        font_size: 8.0,
                        ..default()
                    },
                    MobComponent::new(p, &sprite_data, sprite_manager.as_ref()),
                    MapEntity,
                ));
            }
            EngineEntity::Platform(p) => {
                commands.spawn((
                    GameEntityComponent { entity_id: id },
                    Transform::from_translation(p.position_f32().extend(0.0)),
                    MapEntity,
                    Sprite {
                        color: Color::srgb(0.0, 0.0, 1.0),
                        custom_size: Some(p.size_f32()),
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        ..default()
                    },
                ));
            }
            EngineEntity::Portal(p) => {
                commands.spawn((
                    GameEntityComponent { entity_id: id },
                    Transform::from_translation(p.position_f32().extend(0.0)),
                    MapEntity,
                    Sprite {
                        color: Color::srgb(0.0, 1.0, 0.0),
                        custom_size: Some(p.size_f32()),
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        ..default()
                    },
                ));
            }
            EngineEntity::Rect(p) => {
                commands.spawn((
                    GameEntityComponent { entity_id: id },
                    Transform::from_translation(p.position_f32().extend(0.0)),
                    MapEntity,
                    Sprite {
                        color: Color::srgb(p.color.x, p.color.y, p.color.z),
                        custom_size: Some(p.size_f32()),
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        ..default()
                    },
                ));
            }
            EngineEntity::Emoji(p) => {
                let asset_name = "reactions/eqib.jpg".to_string();
                if !sprite_manager.is_image_loaded(&asset_name, &asset_server) {
                    sprite_manager.load_image(asset_name.clone(), &asset_server);
                    continue;
                }
                commands.spawn((
                    GameEntityComponent { entity_id: id },
                    Transform::from_translation(p.position_f32().extend(20.0)),
                    MapEntity,
                    Sprite {
                        image: sprite_manager.image_handle(&asset_name),
                        custom_size: Some(p.size_f32()),
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        ..default()
                    },
                ));
            }
            EngineEntity::Text(p) => {
                commands.spawn((
                    GameEntityComponent { entity_id: id },
                    Transform::from_translation(p.position_f32().extend(20.0)),
                    MapEntity,
                    Text2d(p.text),
                    TextFont {
                        font_size: p.font_size,
                        ..default()
                    },
                    TextColor(Color::srgb(p.color.x, p.color.y, p.color.z)),
                ));
            }
        }
    }
}
