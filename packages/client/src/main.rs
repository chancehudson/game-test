use bevy::prelude::*;

use bevy_egui::EguiPlugin;

use engine::GameEngine;
pub use game_common::MapData;
pub use game_common::action::Action;
pub use game_common::action::Response;

mod components;
mod interpolation;
mod map;
mod map_data_loader;
mod network;
mod plugins;
mod sprite_data_loader;

use network::NetworkMessage;

use crate::sprite_data_loader::SpriteDataAsset;
use crate::sprite_data_loader::SpriteManager;

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    Disconnected,
    Waiting,
    LoggedOut,
    LoadingMap,
    OnMap,
}

// Event for incoming messages
#[derive(Event, Debug)]
pub struct LoadSpriteRequest(pub u64);

fn main() {
    let mut app = App::new();
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(bevy_web_asset::WebAssetPlugin::default());
    app.add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()),))
        .init_state::<GameState>()
        .init_resource::<SpriteManager>()
        .add_event::<LoadSpriteRequest>()
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        // state stuff
        .add_plugins(plugins::engine_sync::DataHUDPlugin)
        .add_plugins(plugins::engine::EnginePlugin)
        .add_plugins(plugins::loading_screen::LoadingScreenPlugin)
        .add_plugins(plugins::smooth_camera::SmoothCameraPlugin)
        .add_plugins(plugins::animated_sprite::AnimatedSpritePlugin)
        .add_plugins(plugins::login_gui::LoginGuiPlugin)
        .add_plugins(plugins::gui::GuiPlugin)
        // components
        .add_plugins(components::player::PlayerPlugin)
        .add_plugins(components::mob::MobPlugin)
        .add_plugins(components::damage::DamagePlugin)
        // nonsene
        .add_plugins(map::MapPlugin)
        .add_plugins(map_data_loader::MapDataLoaderPlugin)
        .add_plugins(sprite_data_loader::SpriteDataLoaderPlugin)
        .add_plugins(network::NetworkPlugin)
        .add_systems(FixedUpdate, load_sprite_manager)
        .run();
}

fn load_sprite_manager(
    mut sprite_manager: ResMut<SpriteManager>,
    asset_server: Res<AssetServer>,
    sprite_data: Res<Assets<SpriteDataAsset>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    sprite_manager.continue_loading(&asset_server, &sprite_data, &mut texture_atlas_layouts);
}
