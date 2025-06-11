use bevy::asset::LoadedUntypedAsset;
use bevy::prelude::*;

use game_test::MapData;

// use crate::mob::MobRegistry;
use crate::smooth_camera::CAMERA_Y_PADDING;
use crate::ActivePlayerState;

use super::map_data_loader::MapDataAsset;
use super::GameState;

pub struct MapPlugin;

#[derive(Resource, Default)]
pub struct MapLoader {
    pub target_map: Option<String>,
    pub map_data_handle: Option<Handle<MapDataAsset>>,
    pub pending_assets: Option<Vec<Handle<LoadedUntypedAsset>>>,
    loading_complete: bool,
}

impl MapLoader {
    /// TODO: don't clone in the return, this is potentially extremely inefficient
    pub fn map_data(&self, map_assets: &Res<Assets<MapDataAsset>>) -> Option<MapData> {
        if let Some(map_data_handle) = &self.map_data_handle {
            if let Some(asset) = map_assets.get(map_data_handle.id()) {
                return Some(asset.data.clone());
            }
        }
        None
    }

    pub fn is_loaded(&self) -> bool {
        self.loading_complete
    }

    pub fn begin_loading(
        &mut self,
        name: String,
        data_path: String,
        asset_server: Res<AssetServer>,
    ) {
        self.reset();
        self.target_map = Some(name);
        self.map_data_handle = Some(asset_server.load(&data_path));
    }

    pub fn reset(&mut self) {
        self.pending_assets = None;
        self.target_map = None;
        self.map_data_handle = None;
        self.loading_complete = false;
    }

    pub fn continue_loading(
        &mut self,
        asset_server: &Res<AssetServer>,
        map_assets: &Res<Assets<MapDataAsset>>,
    ) {
        if let Some(pending_handles) = &self.pending_assets {
            for pending_handle in pending_handles {
                if !asset_server.is_loaded(pending_handle.id()) {
                    return;
                }
            }
            self.loading_complete = true;
        } else {
            let map_data_handle = self.map_data_handle.as_ref().unwrap();
            if !asset_server.is_loaded(map_data_handle.id()) {
                return;
            }
            let mut pending_handles = vec![];
            // begin loading dependent assets
            if let Some(asset) = map_assets.get(map_data_handle) {
                let map_data = asset.data.clone();
                pending_handles.push(asset_server.load_untyped(&map_data.background));
                for npc in &map_data.npc {
                    pending_handles.push(asset_server.load_untyped(&npc.asset));
                }
            } else {
                panic!("unexpected load state");
            }
            self.pending_assets = Some(pending_handles);
        }
    }
}

#[derive(Component)]
pub struct MapEntity;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapLoader>()
            .add_systems(OnEnter(GameState::LoadingMap), begin_load_map)
            .add_systems(OnExit(GameState::OnMap), exit_map)
            .add_systems(
                FixedUpdate,
                (update_map_loading).run_if(in_state(GameState::LoadingMap)),
            );
    }
}

fn update_map_loading(
    mut commands: Commands,
    mut loader: ResMut<MapLoader>,
    asset_server: Res<AssetServer>,
    map_assets: Res<Assets<MapDataAsset>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    loader.continue_loading(&asset_server, &map_assets);
    if loader.is_loaded() {
        let map_data = loader.map_data(&map_assets).unwrap();
        spawn_background(&mut commands, &asset_server, &map_data);
        spawn_npcs(&mut commands, &asset_server, &map_data);
        next_state.set(GameState::OnMap);
    }
}

fn exit_map(mut commands: Commands, old_map_query: Query<Entity, With<MapEntity>>) {
    for v in &old_map_query {
        commands.entity(v).despawn();
    }
}

fn begin_load_map(
    asset_server: Res<AssetServer>,
    mut loader: ResMut<MapLoader>,
    active_player_state: Res<ActivePlayerState>,
) {
    // Validate we have a map to load
    if active_player_state.0.is_none() {
        error!("Cannot start map load: no map name specified");
        return;
    }
    let active_player_state = active_player_state.0.as_ref().unwrap();

    println!("starting map load 2");
    // Start loading the map data
    let map_path = format!("maps/{}.map.json5", active_player_state.current_map);
    loader.begin_loading(
        active_player_state.current_map.clone(),
        map_path,
        asset_server,
    );
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
