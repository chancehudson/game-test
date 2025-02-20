use bevy::asset::io::Reader;
use bevy::asset::AssetLoader;
use bevy::asset::AsyncReadExt;
use bevy::asset::LoadContext;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::utils::ConditionalSendFuture;

use game_test::MapData;

// Custom asset to hold the config
#[derive(Asset, TypePath, Debug)]
pub struct MapDataAsset {
    pub data: MapData,
}

// Asset loader for JSON files
#[derive(Default)]
struct MapDataLoader;

// Implement asset loader
impl AssetLoader for MapDataLoader {
    type Asset = MapDataAsset;
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
        Box::pin(async move {
            let mut data_str = String::new();
            reader.read_to_string(&mut data_str).await?;
            let data = json5::from_str(&data_str)?;
            Ok(MapDataAsset { data })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json5"]
    }
}

pub struct MapDataLoaderPlugin;

impl Plugin for MapDataLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MapDataAsset>()
            .init_asset_loader::<MapDataLoader>();
    }
}
