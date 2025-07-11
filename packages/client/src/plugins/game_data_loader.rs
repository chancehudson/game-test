use bevy::prelude::*;

use game_common::data::GameData;

#[derive(Resource, Default)]
pub struct GameDataResource(pub GameData);

pub struct GameDataLoaderPlugin;

impl Plugin for GameDataLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameDataResource>()
            .add_systems(Startup, load_game_data);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn load_game_data(mut game_data: ResMut<GameDataResource>) {
    game_data.0 = GameData::load().unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game_data(mut game_data: ResMut<GameDataResource>) {
    game_data.0 = GameData::load(std::path::Path::new("./assets")).unwrap();
}
