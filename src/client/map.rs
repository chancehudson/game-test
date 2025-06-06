use bevy::asset::LoadState;
use bevy::prelude::*;

use game_test::MapData;

// use crate::mob::MobRegistry;
use crate::smooth_camera::CAMERA_Y_PADDING;
use crate::ActivePlayerState;

use super::map_data_loader::MapDataAsset;
use super::GameState;

pub struct MapPlugin;

#[derive(Event)]
pub struct MapLoadComplete {
    pub map_data: MapData,
}

#[derive(Event)]
pub struct MapLoadFailed {
    pub error: String,
}

#[derive(Default)]
pub enum MapLoadingState {
    #[default]
    Idle,
    LoadingMapData(Handle<MapDataAsset>),
    LoadingAssets {
        map_data: MapData,
        pending: Vec<Handle<Image>>,
        loaded: usize,
    },
    Failed(String),
}

#[derive(Resource, Default)]
pub struct MapLoader {
    pub state: MapLoadingState,
    pub target_map: String,
}

#[derive(Component)]
pub struct MapEntity;

#[derive(Resource, Default)]
pub struct ActiveMap {
    pub name: String,
    pub solids: Vec<Rect>,
    pub size: Vec2,
    pub data: Option<MapData>,
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveMap>()
            .init_resource::<MapLoader>()
            .add_event::<MapLoadComplete>()
            .add_event::<MapLoadFailed>()
            .add_systems(OnEnter(GameState::LoadingMap), begin_load_map)
            .add_systems(OnExit(GameState::OnMap), exit_map)
            // .add_systems(OnExit(GameState::LoadingMap), end_load_map)
            .add_systems(
                Update,
                (
                    update_map_loading,
                    handle_map_load_complete,
                    // spawn_map_entities,
                )
                    .run_if(in_state(GameState::LoadingMap)),
            );
    }
}

fn handle_map_load_complete(
    mut events: EventReader<MapLoadComplete>,
    mut next_state: ResMut<NextState<GameState>>,
    mut active_map: ResMut<ActiveMap>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        active_map.name = event.map_data.name.clone();
        active_map.size = event.map_data.size;
        active_map.data = Some(event.map_data.clone());
        spawn_background(&mut commands, &asset_server, &event.map_data);
        spawn_platforms(&mut commands, &asset_server, &event.map_data);
        spawn_portals(&mut commands, &asset_server, &event.map_data);
        spawn_npcs(&mut commands, &asset_server, &event.map_data);
        next_state.set(GameState::OnMap);
    }
}

fn update_map_loading(
    mut loader: ResMut<MapLoader>,
    asset_server: Res<AssetServer>,
    map_assets: Res<Assets<MapDataAsset>>,
    mut load_complete: EventWriter<MapLoadComplete>,
    mut load_failed: EventWriter<MapLoadFailed>,
) {
    match &mut loader.state {
        MapLoadingState::LoadingMapData(handle) => {
            match asset_server.get_load_state(handle.id()) {
                Some(LoadState::Loaded) => {
                    if let Some(asset) = map_assets.get(handle) {
                        let map_data = asset.data.clone();
                        let mut pending = vec![];
                        pending.push(asset_server.load(&map_data.background));
                        for npc in &map_data.npc {
                            pending.push(asset_server.load(&npc.asset));
                        }
                        loader.state = MapLoadingState::LoadingAssets {
                            map_data,
                            pending,
                            loaded: 0,
                        };
                    } else {
                        loader.state = MapLoadingState::Failed("Map asset not found".to_string());
                    }
                }
                Some(LoadState::Failed(err)) => {
                    loader.state =
                        MapLoadingState::Failed(format!("Failed to load map: {:?}", err));
                }
                _ => {} // Still loading
            }
        }
        MapLoadingState::LoadingAssets {
            map_data,
            pending,
            loaded,
        } => {
            let mut new_loaded = 0;
            for handle in pending.iter() {
                if asset_server.is_loaded(handle.id()) {
                    new_loaded += 1;
                }
            }
            *loaded = new_loaded;

            if *loaded == pending.len() {
                load_complete.send(MapLoadComplete {
                    map_data: map_data.clone(),
                });
                loader.state = MapLoadingState::Idle;
            }
        }
        MapLoadingState::Failed(error) => {
            load_failed.send(MapLoadFailed {
                error: error.clone(),
            });
            loader.state = MapLoadingState::Idle;
        }
        _ => {}
    }
}

fn exit_map(
    // mut mob_registry: ResMut<MobRegistry>,
    mut commands: Commands,
    old_map_query: Query<Entity, With<MapEntity>>,
) {
    for v in &old_map_query {
        commands.entity(v).despawn_recursive();
    }
    // mob_registry.mobs.clear();
}

fn begin_load_map(
    asset_server: Res<AssetServer>,
    mut loader: ResMut<MapLoader>,
    active_player_state: Res<ActivePlayerState>,
) {
    // Clean up any previous loading state
    if !matches!(loader.state, MapLoadingState::Idle) {
        warn!("Starting new map load while previous load was in progress");
    }

    // Validate we have a map to load
    if active_player_state.0.is_none() {
        error!("Cannot start map load: no map name specified");
        loader.state = MapLoadingState::Failed("No map name specified".to_string());
        return;
    }
    let active_player_state = active_player_state.0.as_ref().unwrap();

    // Start loading the map data
    let map_path = format!("maps/{}.json5", active_player_state.current_map);
    let handle: Handle<MapDataAsset> = asset_server.load(&map_path);

    info!("Starting to load map: {}", active_player_state.current_map);
    loader.state = MapLoadingState::LoadingMapData(handle);
    loader.target_map = active_player_state.current_map.clone();
}

pub fn spawn_background(commands: &mut Commands, asset_server: &AssetServer, map_data: &MapData) {
    commands.spawn((
        MapEntity,
        Sprite {
            anchor: bevy::sprite::Anchor::BottomLeft,
            image: asset_server.load(&map_data.background),
            custom_size: Some(Vec2::new(
                map_data.size.x,
                map_data.size.y + CAMERA_Y_PADDING,
            )),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -CAMERA_Y_PADDING, -10.0)),
    ));
}

pub fn spawn_platforms(commands: &mut Commands, asset_server: &AssetServer, map_data: &MapData) {
    for platform in &map_data.platforms {
        commands.spawn((
            MapEntity,
            Transform::from_translation(Vec3::new(platform.position.x, platform.position.y, -1.0)),
            Sprite {
                color: Color::srgb(0.0, 0.0, 1.0),
                custom_size: Some(Vec2::new(platform.size.x, platform.size.y)),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
        ));
    }
}

pub fn spawn_portals(commands: &mut Commands, asset_server: &AssetServer, map_data: &MapData) {
    for portal in &map_data.portals {
        commands.spawn((
            MapEntity,
            Transform::from_translation(Vec3::new(portal.position.x, portal.position.y, -1.0)),
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(portal.rect().size()),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
        ));
    }
}

pub fn spawn_npcs(commands: &mut Commands, asset_server: &AssetServer, map_data: &MapData) {
    for npc in &map_data.npc {
        commands.spawn((
            MapEntity,
            Transform::from_translation(Vec3::new(npc.position.x, npc.position.y, 0.0)),
            Sprite {
                image: asset_server.load(&npc.asset),
                custom_size: Some(Vec2::new(npc.size.x, npc.size.y)),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
        ));
    }
}
