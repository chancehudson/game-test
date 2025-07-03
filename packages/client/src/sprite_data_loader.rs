use std::collections::HashMap;
use std::collections::HashSet;

use bevy::asset::AssetLoader;
use bevy::asset::AsyncReadExt;
use bevy::asset::LoadContext;
use bevy::asset::io::Reader;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::tasks::ConditionalSendFuture;

use game_common::mob::SPRITE_MANIFEST;
use game_common::mob::SpriteAnimationData;

#[derive(Resource, Default)]
pub struct SpriteManager {
    // first load the sprite data handle
    // then load the images specified in the data file
    sprite_data_handle_map: HashMap<u64, Handle<SpriteDataAsset>>,
    // filepath keyed to handle
    sprite_image_handle_map: HashMap<String, Handle<Image>>,
    sprite_texture_atlas_map: HashMap<String, Handle<TextureAtlasLayout>>,
    sprite_id_to_images: HashMap<u64, HashSet<String>>,
}

impl SpriteManager {
    pub fn is_loaded(&self, id: &u64, sprite_data: &Res<Assets<SpriteDataAsset>>) -> bool {
        if let Some(data) = self.sprite_data_maybe(id, sprite_data) {
            for (name, _atlas) in data.sprite_sheets() {
                if self.sprite_texture_atlas_map.contains_key(&name)
                    && self.sprite_image_handle_map.contains_key(&name)
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn image_handle(&self, image_path: &str) -> Handle<Image> {
        self.sprite_image_handle_map
            .get(image_path)
            .unwrap()
            .clone()
    }

    pub fn is_image_loaded(&self, image_path: &str, asset_server: &Res<AssetServer>) -> bool {
        if let Some(handle) = self.sprite_image_handle_map.get(image_path) {
            if asset_server.is_loaded(handle.id()) {
                return true;
            }
        }
        false
    }

    pub fn load_image(&mut self, image_path: String, asset_server: &Res<AssetServer>) {
        self.sprite_image_handle_map
            .insert(image_path.clone(), asset_server.load(image_path));
    }

    pub fn load(&mut self, id: u64, asset_server: &Res<AssetServer>) {
        if let Some(_) = self.sprite_data_handle_map.get(&id) {
            return;
        }
        if let Some(filepath) = SPRITE_MANIFEST.get(&id) {
            let handle = asset_server.load(filepath);
            self.sprite_data_handle_map.insert(id, handle);
        }
    }

    pub fn sprite(&self, name: &str) -> Option<(&Handle<Image>, &Handle<TextureAtlasLayout>)> {
        if let Some(handle) = self.sprite_image_handle_map.get(name) {
            if let Some(atlas) = self.sprite_texture_atlas_map.get(name) {
                return Some((handle, atlas));
            }
        }
        None
    }

    pub fn sprite_data_maybe(
        &self,
        id: &u64,
        sprite_data: &Res<Assets<SpriteDataAsset>>,
    ) -> Option<SpriteAnimationData> {
        if let Some(handle) = self.sprite_data_handle_map.get(id) {
            sprite_data.get(handle).and_then(|v| Some(v.0.clone()))
        } else {
            None
        }
    }

    pub fn continue_loading(
        &mut self,
        asset_server: &Res<AssetServer>,
        sprite_data: &Res<Assets<SpriteDataAsset>>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) {
        let loading_sprites = self
            .sprite_data_handle_map
            .iter()
            .filter(|(id, _handle)| !self.sprite_id_to_images.contains_key(id))
            .collect::<Vec<_>>();
        for (id, handle) in loading_sprites {
            if let Some(data) = sprite_data.get(handle) {
                let mut sprite_id_images = HashSet::new();
                for (name, atlas) in data.0.sprite_sheets() {
                    let atlas_handle = texture_atlas_layouts.add(atlas);
                    self.sprite_texture_atlas_map
                        .insert(name.clone(), atlas_handle);
                    self.sprite_image_handle_map
                        .insert(name.clone(), asset_server.load(&name));
                    sprite_id_images.insert(name.clone());
                }
                self.sprite_id_to_images.insert(*id, sprite_id_images);
            }
        }
    }
}

// Custom asset to hold the config
#[derive(Asset, TypePath, Debug)]
pub struct SpriteDataAsset(pub SpriteAnimationData);

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
            Ok(SpriteDataAsset(data))
        }
    }

    fn extensions(&self) -> &[&str] {
        &["sprite.json5"]
    }
}

pub struct SpriteDataLoaderPlugin;

impl Plugin for SpriteDataLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SpriteDataAsset>()
            .init_asset_loader::<SpriteDataLoader>();
    }
}
