use bevy::prelude::*;

use game_test::MapData;

use super::map_data_loader::MapDataAsset;
use super::GameState;
use game_test::Mob;

pub struct MapPlugin;

#[derive(Component)]
pub struct LoadingView;

#[derive(Component)]
pub struct MapEntity;

#[derive(Component)]
pub struct MobEntity(pub Mob);

#[derive(Resource, Default)]
pub struct MapLoadingAssets {
    pub pending_map_data: Option<Handle<MapDataAsset>>,
    pub pending_assets: Option<Vec<Handle<Image>>>,
}

#[derive(Resource, Default)]
pub struct ActiveMap {
    pub name: String,
    pub solids: Vec<Rect>,
    pub size: Vec2,
    pub data: Option<MapData>,
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapLoadingAssets>()
            .init_resource::<ActiveMap>()
            .add_systems(OnEnter(GameState::LoadingMap), begin_load_map)
            .add_systems(OnEnter(GameState::OnMap), enter_map)
            .add_systems(Update, check_assets.run_if(in_state(GameState::LoadingMap)));
    }
}

fn check_assets(
    mut next_state: ResMut<NextState<GameState>>,
    mut game_assets: ResMut<MapLoadingAssets>,
    asset_server: Res<AssetServer>,
    map_datas: Res<Assets<MapDataAsset>>,
) {
    if let Some(pending_map_data) = &game_assets.pending_map_data {
        if asset_server.is_loaded(pending_map_data.id()) && game_assets.pending_assets.is_none() {
            let map_data = map_datas.get(pending_map_data).unwrap();
            let mut pending_assets = vec![];
            pending_assets.push(asset_server.load(map_data.data.background.clone()));
            for npc in &map_data.data.npc {
                pending_assets.push(asset_server.load(npc.asset.clone()));
            }
            game_assets.pending_assets = Some(pending_assets);
        }
    }
    if let Some(pending_assets) = &game_assets.pending_assets {
        for asset in pending_assets {
            if !asset_server.is_loaded(asset.id()) {
                return;
            }
        }
        next_state.set(GameState::OnMap);
    }
}

fn begin_load_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map_loading_assets: ResMut<MapLoadingAssets>,
    windows: Query<&Window>,
    old_map_query: Query<Entity, With<MapEntity>>,
    active_map: Res<ActiveMap>,
) {
    for v in &old_map_query {
        commands.entity(v).despawn();
    }
    // despawn all old map assets
    if map_loading_assets.pending_assets.is_some() || map_loading_assets.pending_map_data.is_some()
    {
        // we're already in the process of loading a map. Clear the original pending
        // loads and register our new ones
        println!("Began loading a second map before first one completed!");
    }
    map_loading_assets.pending_assets = None;
    let map_data_handle: Handle<MapDataAsset> =
        asset_server.load(format!("maps/{}.json5", active_map.name));
    map_loading_assets.pending_map_data = Some(map_data_handle);
    let window = windows.single();
    let width = window.resolution.width();
    let height = window.resolution.height();
    commands.spawn((
        LoadingView,
        Sprite {
            color: Color::srgb(1., 0., 0.),
            custom_size: Some(Vec2::new(width, height)),
            anchor: bevy::sprite::Anchor::BottomLeft,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
    ));
}

fn enter_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_assets: ResMut<MapLoadingAssets>,
    map_datas: Res<Assets<MapDataAsset>>,
    loading_query: Query<Entity, With<LoadingView>>,
    mut active_map: ResMut<ActiveMap>,
) {
    for v in &loading_query {
        commands.entity(v).despawn();
    }
    println!("Done loading!");
    let map_data = map_datas
        .get(game_assets.pending_map_data.as_ref().unwrap())
        .unwrap();
    let load_handle = asset_server.load(&map_data.data.background);
    commands.spawn((
        MapEntity,
        Sprite {
            anchor: bevy::sprite::Anchor::BottomLeft,
            image: load_handle,
            custom_size: Some(Vec2::new(map_data.data.size.x, map_data.data.size.y)),
            // color: Color::srgb(1., 0., 0.),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
    ));
    active_map.size = Vec2::new(map_data.data.size.x, map_data.data.size.y);
    active_map.solids = map_data
        .data
        .platforms
        .iter()
        .map(|p| {
            Rect::new(
                p.position.x,
                p.position.y,
                p.position.x + p.size.x,
                p.position.y + p.size.y,
            )
        })
        .collect();
    active_map.data = Some(map_data.data.clone());
    for platform in &map_data.data.platforms {
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
    for portal in &map_data.data.portals {
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
    for npc in &map_data.data.npc {
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
    println!("Map data: {:?}", map_data);
    game_assets.pending_assets = None;
    game_assets.pending_map_data = None;
}
