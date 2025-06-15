use bevy::dev_tools::fps_overlay::FpsOverlayConfig;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use bevy::text::FontSmoothing;

use bevy_egui::EguiPlugin;
pub use game_test::MapData;
pub use game_test::action::Action;
use game_test::action::PlayerState;
pub use game_test::action::Response;
use game_test::engine::GameEngine;

mod map;
mod map_data_loader;
mod mob;
mod network;
mod player;
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
        .add_plugins(plugins::engine_sync::DataHUDPlugin)
        .add_plugins(plugins::engine::EnginePlugin)
        .add_plugins(plugins::loading_screen::LoadingScreenPlugin)
        .add_plugins(plugins::smooth_camera::SmoothCameraPlugin)
        .add_plugins(plugins::animated_sprite::AnimatedSpritePlugin)
        .add_plugins(plugins::gui::GuiPlugin)
        // nonsene
        .add_plugins(map::MapPlugin)
        .add_plugins(map_data_loader::MapDataLoaderPlugin)
        .add_plugins(sprite_data_loader::SpriteDataLoaderPlugin)
        .add_plugins(network::NetworkPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(mob::MobPlugin)
        .add_systems(Update, load_sprite_manager)
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
