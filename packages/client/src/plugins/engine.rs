use std::collections::BTreeMap;
use std::collections::HashMap;

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use bevy::text::TextLayoutInfo;

use db::PlayerRecord;
use game_common::prelude::*;
use keind::prelude::*;

use crate::GameState;
use crate::NetworkMessage;
use crate::SpriteManager;
use crate::components::mob::MobComponent;
use crate::components::player::PlayerComponent;
use crate::interpolation::InterpolatingEntities;
use crate::interpolation::Interpolation;
use crate::interpolation::interpolate_mobs;
use crate::map::MapEntity;
use crate::network::NetworkAction;
use crate::plugins::animated_sprite::AnimatedSprite;
use crate::plugins::engine_sync::EngineSyncInfo;
use crate::plugins::game_data_loader::GameDataResource;
use crate::plugins::info_text::InfoMessage;

/// Engine tracking resources/components
///
#[derive(Resource, Default)]
pub struct ActiveGameEngine(pub GameEngine<KeindGameLogic>);

#[derive(Component, Default)]
pub struct GameEntityComponent {
    pub entity_id: u128,
}

#[derive(Resource, Default)]
pub struct ActivePlayerEntityId(pub Option<u128>);

#[derive(Resource, Default)]
pub struct LoggedInAt(pub f64);

#[derive(Resource, Default)]
pub struct ActivePlayerState(pub Option<PlayerRecord>);

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveGameEngine>()
            .init_resource::<ActivePlayerEntityId>()
            .init_resource::<ActivePlayerState>()
            .init_resource::<LoggedInAt>()
            .init_resource::<InterpolatingEntities>()
            .add_systems(
                Update,
                (
                    handle_engine_state,
                    handle_engine_stats,
                    handle_engine_event,
                    step_game_engine,
                    sync_engine_components,
                    add_simple_bubble_background,
                )
                    .chain()
                    .run_if(
                        in_state(GameState::OnMap)
                            .or(in_state(GameState::LoadingMap))
                            .or(in_state(GameState::Waiting)),
                    ),
            )
            .add_systems(
                FixedUpdate,
                (handle_login, handle_exit_map, handle_player_state),
            );
    }
}

fn handle_login(
    mut action_events: EventReader<NetworkMessage>,
    mut active_player_entity_id: ResMut<ActivePlayerEntityId>,
    mut logged_in_at: ResMut<LoggedInAt>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in action_events.read() {
        if let Response::PlayerLoggedIn(_state) = &event.0 {
            active_player_entity_id.0 = None;
            logged_in_at.0 = timestamp();
            next_state.set(GameState::Waiting);
        }
    }
}

fn handle_player_state(
    mut action_events: EventReader<NetworkMessage>,
    mut active_player_state: ResMut<ActivePlayerState>,
) {
    for event in action_events.read() {
        if let Response::PlayerState(state) = &event.0 {
            println!("Player state received");
            active_player_state.0 = Some(state.clone());
        }
    }
}

fn step_game_engine(
    mut active_game_engine: ResMut<ActiveGameEngine>,
    sync_info: Res<EngineSyncInfo>,
    mut info_event_writer: EventWriter<InfoMessage>,
    game_data: Res<GameDataResource>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
) {
    let game_data = &game_data.0;
    let engine = &mut active_game_engine.0;
    let target_step = sync_info.server_step
        + (((timestamp() - sync_info.server_step_timestamp) / STEP_LEN_S).ceil() as u64);
    let game_events = if target_step > engine.step_index {
        let steps = target_step - engine.step_index;
        if steps >= 30 {
            println!("skipping forward {} steps", steps / 2);
            engine.step_to(&(engine.step_index + (steps / 2)))
        } else if steps >= 10 {
            let mut out = engine.step();
            out.append(&mut engine.step());
            out
        } else {
            engine.step()
        }
    } else {
        println!("skipped step");
        vec![]
        // local engine is ahead of server, skip a step
    };
    for event in game_events {
        match event {
            GameEvent::Message(_, _) => {
                // spawn a message in bevy
            }
            GameEvent::PlayerPickUp(_, item_type, count) => {
                if let Some(item) = game_data.items.get(&item_type) {
                    info_event_writer.write(InfoMessage(format!("+ {count} {}", item.name)));
                }
                // TODO: optimistically make update
            }
            GameEvent::PlayerAbilityExp(entity_id, ability, amount) => {
                if let Some(player_entity_id) = active_player_entity_id.0
                    && player_entity_id == entity_id
                {
                    let ability_str: &'static str = ability.into();
                    info_event_writer
                        .write(InfoMessage(format!("+ {} {} exp", amount, ability_str)));
                }
            }
            _ => {}
        }
    }
}

fn handle_exit_map(
    mut action_events: EventReader<NetworkMessage>,
    query: Query<Entity, With<MapEntity>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut active_player_entity_id: ResMut<ActivePlayerEntityId>,
    mut active_game_engine: ResMut<ActiveGameEngine>,
) {
    for event in action_events.read() {
        if let Response::PlayerExitMap(_from_map) = &event.0 {
            println!("Player exit map received");
            for entity in query {
                commands.entity(entity).despawn();
            }
            active_player_entity_id.0 = None;
            active_game_engine.0 = RewindableGameEngine::default();
            next_state.set(GameState::Waiting);
        }
    }
}

fn handle_engine_event(
    mut action_events: EventReader<NetworkMessage>,
    mut active_engine_state: ResMut<ActiveGameEngine>,
    mut engine_sync: ResMut<EngineSyncInfo>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
    mut interpolating_entities: ResMut<InterpolatingEntities>,
) {
    for event in action_events.read() {
        match &event.0 {
            Response::RemoteEngineEvents(engine_id, events, server_step_index) => {
                let engine = &mut active_engine_state.0;
                if engine.id != *engine_id || events.is_empty() {
                    continue;
                }
                let player_entity_id = active_player_entity_id.0.unwrap_or_default();
                engine_sync.server_step = *server_step_index;
                engine_sync.server_step_timestamp = timestamp();
                // these are the mobs we were seeing before
                let last_mobs = engine
                    .entities_by_type::<MobEntity>()
                    .cloned()
                    .collect::<Vec<_>>();
                engine.integrate_events(events.clone());
                interpolate_mobs(
                    last_mobs,
                    engine,
                    player_entity_id,
                    &mut interpolating_entities,
                );
            }
            _ => {}
        }
    }
}

fn handle_engine_stats(
    mut action_events: EventReader<NetworkMessage>,
    mut action_events_writer: EventWriter<NetworkAction>,
    mut engine_sync: ResMut<EngineSyncInfo>,
    active_engine_state: Res<ActiveGameEngine>,
) {
    let engine = &active_engine_state.0;
    for event in action_events.read() {
        if let Response::EngineStats(
            engine_id,
            step_index,
            (hash_step_index, server_engine_hash),
            entities_maybe,
        ) = &event.0
        {
            if engine_id != &active_engine_state.0.id {
                println!(
                    "WARNING: received engine stats for inactive engine, discarding  server: {} local: {}",
                    engine_id, active_engine_state.0.id
                );
                return;
            }
            engine_sync.server_step = *step_index;
            engine_sync.server_step_timestamp = timestamp();
            engine_sync.sync_distance = (engine.step_index as i64) - (*step_index as i64);
            if !engine_sync.requested_resync {
                if let Ok(local_engine_hash) = engine.step_hash(&hash_step_index) {
                    if local_engine_hash != *server_engine_hash {
                        println!("WARNING: desync detected");
                        println!(
                            "local engine state: {:?}",
                            active_engine_state.0.entities_at_step(*hash_step_index)
                        );
                        action_events_writer.write(NetworkAction(Action::RequestEngineReload(
                            engine.id,
                            *hash_step_index,
                        )));
                        engine_sync.requested_resync = true;
                        // trigger resync
                        // debug if needed
                        if let Some(server_entities) = entities_maybe {
                            let local_entities = engine.entities_at_step(*hash_step_index);
                            let server_json =
                                serde_json::to_string_pretty(&server_entities).unwrap();
                            let local_json = serde_json::to_string_pretty(&local_entities).unwrap();
                            let diff = similar::TextDiff::from_lines(&local_json, &server_json);
                            for change in diff.iter_all_changes() {
                                let sign = match change.tag() {
                                    similar::ChangeTag::Delete => "-",
                                    similar::ChangeTag::Insert => "+",
                                    similar::ChangeTag::Equal => " ",
                                };
                                if change.tag() != similar::ChangeTag::Equal {
                                    println!("diff {}{}", sign, change);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn handle_engine_state(
    mut action_events: EventReader<NetworkMessage>,
    mut active_engine_state: ResMut<ActiveGameEngine>,
    mut next_state: ResMut<NextState<GameState>>,
    mut engine_sync: ResMut<EngineSyncInfo>,
    mut active_player_entity_id: ResMut<ActivePlayerEntityId>,
) {
    for event in action_events.read() {
        if let Response::EngineState(engine, player_entity_id_maybe, server_step) = &event.0 {
            active_player_entity_id.0 = Some(*player_entity_id_maybe);
            *engine_sync = EngineSyncInfo::default();
            engine_sync.server_step = *server_step;
            engine_sync.server_step_timestamp = timestamp();
            active_engine_state.0 = engine.clone();
            let engine = &mut active_engine_state.0;
            if server_step > &engine.step_index {
                engine.step_to(&server_step);
            }
            println!("INFO: Received engine with id: {}", engine.id);
            // TODO: figure out how to get rid of this clone
            next_state.set(GameState::LoadingMap);
        }
    }
}

/// Call S the current step number
/// By default we'll show all components with player_creator_id != self at their positions S - STEP_DELAY
/// any entity with player_creator_id == self will be shown at S
/// if player_creator_id == None we show at S because it's assume deterministic
///
/// This logical split happens at the Bevy/plugin level, not the engine level. So the engine
/// remains deterministic and checksum verifiable
pub fn sync_engine_components(
    mut commands: Commands,
    active_engine_state: Res<ActiveGameEngine>,
    mut entity_query: Query<(Entity, &GameEntityComponent, &mut Transform)>,
    asset_server: Res<AssetServer>,
    mut sprite_manager: ResMut<SpriteManager>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
    mut interpolating_entities: ResMut<InterpolatingEntities>,
    game_data: Res<GameDataResource>,
) {
    let player_entity_id = active_player_entity_id.0.unwrap_or_default();
    let engine = &active_engine_state.0;
    // this is the entities in relative positions we want to render
    let mut aggrod_mobs = HashMap::new();
    let mut current_entities = engine
        .entities_at_step(engine.step_index)
        .iter()
        .filter(|(_id, entity)| {
            match entity {
                EngineEntity::Mob(p) => {
                    if let Some(aggro_to) = p.aggro_to {
                        if aggro_to.0 != player_entity_id {
                            aggrod_mobs.insert(p.id, true);
                            return false;
                        }
                    }
                }
                _ => {}
            }
            if let Some(player_creator_id) = entity.player_creator_id() {
                player_creator_id == player_entity_id
            } else {
                true
            }
        })
        .collect::<BTreeMap<_, _>>();
    if engine.step_index >= STEP_DELAY {
        let past_step_index = engine.step_index - STEP_DELAY;
        let past_entities = engine.entities_at_step(past_step_index);
        for (entity_id, entity) in past_entities.iter().filter(|(id, entity)| {
            if aggrod_mobs.contains_key(id) {
                return true;
            }
            if let Some(player_creator_id) = entity.player_creator_id() {
                player_creator_id != player_entity_id
            } else {
                false
            }
        }) {
            if let Some(_) = current_entities.insert(entity_id, entity) {
                println!("WARNING: entity filtered to both present and past");
            }
        }
    }
    let mut position_overrides = HashMap::new();
    for (entity_id, interpolation) in &interpolating_entities.0 {
        if engine.step_index < interpolation.to_step {
            let pos = interpolation.start_position
                + IVec2::splat((engine.step_index - interpolation.from_step) as i32)
                    * interpolation.diff_position;
            position_overrides.insert(*entity_id, pos);
        }
    }
    interpolating_entities
        .0
        .retain(|_, Interpolation { to_step, .. }| engine.step_index < *to_step);

    for (entity, entity_component, mut transform) in entity_query.iter_mut() {
        if let Some(game_entity) = current_entities.get(&entity_component.entity_id) {
            if let Some(position_override) = position_overrides.get(&game_entity.id()) {
                transform.translation = Vec3::new(
                    position_override.x as f32,
                    position_override.y as f32,
                    transform.translation.z,
                );
            } else {
                transform.translation = game_entity.position_f32().extend(transform.translation.z);
            }
            current_entities.remove(&game_entity.id());
        } else {
            commands.entity(entity).despawn();
        }
    }
    // we're left with game entities we need to spawn
    for (_id, engine_entity) in current_entities {
        spawn_bevy_entity(
            &game_data,
            engine_entity,
            &mut commands,
            &asset_server,
            &mut sprite_manager,
        );
    }
}

#[derive(Component)]
struct NeedsSpriteBackground;

fn add_simple_bubble_background(
    mut commands: Commands,
    query: Query<(Entity, &TextLayoutInfo, &GameEntityComponent), With<NeedsSpriteBackground>>,
    active_engine: Res<ActiveGameEngine>,
) {
    let engine = &active_engine.0;
    for (entity, info, game_entity) in query.iter() {
        let text_size = info.size;
        if text_size == Vec2::ZERO {
            continue;
        }

        const MARGIN: f32 = 4.0;
        if let Some(p) = engine
            .entities_at_step(engine.step_index)
            .get(&game_entity.entity_id)
        {
            let bubble_size =
                Vec2::new(p.size().x as f32 + MARGIN * 2.0, text_size.y + MARGIN * 2.0);

            // Create a nice speech bubble using multiple rounded rectangles
            let bubble_id = commands
                .spawn((
                    Sprite {
                        color: Color::srgba(0.95, 0.95, 0.95, 0.95), // Light background
                        custom_size: Some(bubble_size),
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(-MARGIN, -MARGIN, -0.1)),
                ))
                .with_children(|parent| {
                    // Add border effect with slightly larger darker sprite behind
                    parent.spawn((
                        Sprite {
                            color: Color::srgba(0.6, 0.6, 0.6, 0.8), // Border color
                            custom_size: Some(bubble_size + Vec2::splat(2.0)),
                            anchor: Anchor::BottomLeft,
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(-1.0, -1.0, -0.1)),
                    ));
                })
                .id();

            commands
                .entity(entity)
                .add_child(bubble_id)
                .remove::<NeedsSpriteBackground>();
        }
    }
}
pub fn spawn_bevy_entity(
    game_data: &Res<GameDataResource>,
    engine_entity: &EngineEntity,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    sprite_manager: &mut ResMut<SpriteManager>,
) {
    match engine_entity {
        EngineEntity::Npc(p) => {
            let standing_animation = &p.data.standing_animation;
            if !sprite_manager.is_animation_loaded(standing_animation, asset_server) {
                sprite_manager.load_animation(standing_animation);
                return;
            }
            let (handle, atlas) = sprite_manager
                .atlas(&standing_animation.sprite_sheet)
                .unwrap();
            commands.spawn((
                GameEntityComponent {
                    entity_id: engine_entity.id(),
                },
                MapEntity,
                Transform::from_translation(p.position_f32().extend(10.0)),
                AnimatedSprite {
                    frame_count: standing_animation.frame_count as u8,
                    fps: standing_animation.fps as u8,
                    time: 0.0,
                },
                Sprite {
                    image: handle.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas.clone(),
                        index: 0,
                    }),
                    custom_size: Some(p.size_f32()),
                    anchor: bevy::sprite::Anchor::BottomLeft,
                    ..default()
                },
            ));
        }
        EngineEntity::Message(p) => {
            commands.spawn((
                GameEntityComponent {
                    entity_id: engine_entity.id(),
                },
                Transform::from_translation(p.position_f32().extend(100.0)),
                MapEntity,
                NeedsSpriteBackground,
                Text2d::new(&p.text),
                TextColor(Color::BLACK.lighter(0.05)),
                Anchor::BottomLeft,
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    linebreak: LineBreak::WordOrCharacter,
                },
                TextBounds {
                    width: Some(p.size.x as f32),
                    ..default()
                },
            ));
        }
        EngineEntity::Player(p) => {
            let default_animation = PlayerComponent::default_animation();
            if !sprite_manager.is_animation_loaded(&default_animation, asset_server) {
                sprite_manager.load_animation(&default_animation);
                return;
            }
            commands
                .spawn((
                    GameEntityComponent {
                        entity_id: engine_entity.id(),
                    },
                    Transform::from_translation(p.position_f32().extend(100.0)),
                    PlayerComponent::default_sprite(sprite_manager.as_ref()),
                    MapEntity,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Transform::from_translation(Vec3::new(p.size.x as f32 / 2., -10., 100.)),
                        Text2d::new(p.record.username.clone()),
                        TextFont::from_font_size(10.0),
                    ));
                });
        }
        EngineEntity::MobSpawner(_) => {}
        EngineEntity::Mob(p) => {
            let mob_data = if let Some(mob_data) = game_data.0.mobs.get(&p.mob_type) {
                mob_data
            } else {
                println!(
                    "WARNING: unable to find mob data for mob type: {}",
                    p.mob_type
                );
                return;
            };
            for animation_data in vec![&mob_data.walking_animation, &mob_data.standing_animation] {
                if !sprite_manager.is_animation_loaded(&animation_data, asset_server) {
                    sprite_manager.load_animation(&animation_data);
                    return;
                }
            }
            commands.spawn((
                GameEntityComponent {
                    entity_id: engine_entity.id(),
                },
                Transform::from_translation(p.position_f32().extend(1.0)),
                // Text2d(p.id.to_string().split_off(15)),
                // TextFont {
                //     font_size: 8.0,
                //     ..default()
                // },
                MobComponent::new(p.clone(), sprite_manager, game_data),
                MapEntity,
            ));
        }
        EngineEntity::Platform(p) => {
            commands.spawn((
                GameEntityComponent {
                    entity_id: engine_entity.id(),
                },
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
                GameEntityComponent {
                    entity_id: engine_entity.id(),
                },
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
                GameEntityComponent {
                    entity_id: engine_entity.id(),
                },
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
            let animation =
                AnimationData::static_data("reactions/eqib.jpg", UVec2 { x: 25, y: 25 });
            if !sprite_manager.is_animation_loaded(&animation, &asset_server) {
                sprite_manager.load_animation(&animation);
                return;
            }
            commands.spawn((
                GameEntityComponent {
                    entity_id: engine_entity.id(),
                },
                Transform::from_translation(p.position_f32().extend(20.0)),
                MapEntity,
                Sprite {
                    image: sprite_manager
                        .atlas(&animation.sprite_sheet)
                        .unwrap()
                        .0
                        .clone(),
                    custom_size: Some(p.size_f32()),
                    anchor: bevy::sprite::Anchor::BottomLeft,
                    ..default()
                },
            ));
        }
        EngineEntity::Text(p) => {
            commands.spawn((
                GameEntityComponent {
                    entity_id: engine_entity.id(),
                },
                Transform::from_translation(p.position_f32().extend(20.0)),
                MapEntity,
                Text2d(p.text.clone()),
                TextFont {
                    font_size: p.font_size,
                    ..default()
                },
                TextColor(Color::srgb(p.color.x, p.color.y, p.color.z)),
            ));
        }
        EngineEntity::Item(p) => {
            let item_data = if let Some(item_data) = game_data.0.items.get(&p.item_type) {
                item_data
            } else {
                println!(
                    "WARNING: unable to find item data for item type: {}",
                    p.item_type
                );
                return;
            };
            if !sprite_manager.is_animation_loaded(&item_data.icon_animation, &asset_server) {
                sprite_manager.load_animation(&item_data.icon_animation);
                return;
            }
            let (handle, atlas) = sprite_manager
                .atlas(&item_data.icon_animation.sprite_sheet)
                .unwrap();

            commands.spawn((
                GameEntityComponent {
                    entity_id: engine_entity.id(),
                },
                Transform::from_translation(p.position_f32().extend(20.0)),
                MapEntity,
                AnimatedSprite {
                    frame_count: item_data.icon_animation.frame_count as u8,
                    fps: item_data.icon_animation.fps as u8,
                    time: 0.0,
                },
                Sprite {
                    image: handle.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas.clone(),
                        index: 0,
                    }),
                    custom_size: Some(p.size_f32()),
                    anchor: bevy::sprite::Anchor::BottomLeft,
                    // color: Color::srgb(1.0, 0.0, 0.0),
                    ..default()
                },
            ));
        }
        EngineEntity::MobDamage(_) => {}
    }
}
