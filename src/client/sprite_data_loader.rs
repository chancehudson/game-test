use bevy::asset::io::Reader;
use bevy::asset::AssetLoader;
use bevy::asset::AsyncReadExt;
use bevy::asset::LoadContext;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::tasks::ConditionalSendFuture;

use game_test::mob::SpriteAnimationData;

// Custom asset to hold the config
#[derive(Asset, TypePath, Debug)]
pub struct SpriteDataAsset {
    pub data: SpriteAnimationData,
}

// Asset loader for JSON files
#[derive(Default)]
struct SpriteDataLoader;

// Implement asset loader
impl AssetLoader for SpriteDataLoader {
    type Asset = SpriteDataAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &self,
        reader: &'a mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture
           + futures_util::Future<
        Output = Result<<Self as AssetLoader>::Asset, <Self as AssetLoader>::Error>,
    > {
        async move {
            let mut data_str = String::new();
            reader.read_to_string(&mut data_str).await?;
            let data = json5::from_str(&data_str)?;
            Ok(SpriteDataAsset { data })
        }
    }

    fn extensions(&self) -> &[&str] {
        &["json5"]
    }
}

pub struct SpriteDataLoaderPlugin;

impl Plugin for SpriteDataLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SpriteDataAsset>()
            .init_asset_loader::<SpriteDataLoader>();
    }
}
