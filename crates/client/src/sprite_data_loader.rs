use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui;
use bevy_egui::egui::TextureHandle;

use game_common::AnimationData;

fn build_sprite_textures(animation_data: &AnimationData) -> bevy::image::TextureAtlasLayout {
    bevy::image::TextureAtlasLayout::from_grid(
        bevy_math::UVec2::new(animation_data.width as u32, animation_data.height as u32),
        animation_data.frame_count as u32,
        1,
        None,
        None,
    )
}

#[derive(Resource, Default)]
pub struct SpriteManager {
    // filepath keyed to handle
    pending_animations: HashMap<String, ()>,
    animation_data: HashMap<String, AnimationData>,

    sprite_image_handle_map: HashMap<String, Handle<Image>>,
    sprite_texture_atlas_map: HashMap<String, Handle<TextureAtlasLayout>>,

    egui_image_handle_map: HashMap<String, TextureHandle>,
}

impl SpriteManager {
    pub fn is_animation_loaded(
        &self,
        animation_data: &AnimationData,
        asset_server: &Res<AssetServer>,
    ) -> bool {
        if let Some(handle) = self
            .sprite_image_handle_map
            .get(&animation_data.sprite_sheet)
        {
            return asset_server.is_loaded(handle.id())
                && self
                    .sprite_texture_atlas_map
                    .contains_key(&animation_data.sprite_sheet);
        }
        false
    }

    pub fn load_animation(&mut self, data: &AnimationData) {
        self.animation_data
            .insert(data.sprite_sheet.clone(), data.clone());
        self.pending_animations
            .insert(data.sprite_sheet.clone(), ());
    }

    pub fn build_atlases(
        &mut self,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        contexts: &mut EguiContexts,
        images: Res<Assets<Image>>,
    ) {
        let pending_animations = self.pending_animations.drain().collect::<HashMap<_, _>>();
        for (name, _) in pending_animations {
            let animation_data = self.animation_data.get(&name).unwrap();
            if let Some(handle) = self.sprite_image_handle_map.get(&name) {
                if asset_server.is_loaded(handle) {
                    let atlas_handle =
                        texture_atlas_layouts.add(build_sprite_textures(&animation_data));
                    self.sprite_texture_atlas_map
                        .insert(name.clone(), atlas_handle);
                    // TODO: animation support
                    if animation_data.is_static() {
                        let bevy_image = images.get(handle).unwrap();
                        let egui_image = egui::ColorImage::from_rgba_unmultiplied(
                            [bevy_image.width() as usize, bevy_image.height() as usize],
                            bevy_image.data.as_ref().unwrap(),
                        );
                        let egui_texture_handle = contexts.ctx_mut().load_texture(
                            animation_data.sprite_sheet.clone(),
                            egui_image,
                            egui::TextureOptions::default(),
                        );
                        self.egui_image_handle_map.insert(name, egui_texture_handle);
                    }
                } else {
                    // continue loading
                    self.pending_animations.insert(name, ());
                }
            } else {
                // start loading
                self.pending_animations.insert(name.clone(), ());
                let handle = asset_server.load(&name);
                self.sprite_image_handle_map.insert(name, handle);
            }
        }
    }

    pub fn egui_handle(&self, name: &str) -> Option<&egui::TextureHandle> {
        self.egui_image_handle_map.get(name)
    }

    pub fn atlas(&self, name: &str) -> Option<(&Handle<Image>, &Handle<TextureAtlasLayout>)> {
        if let Some(handle) = self.sprite_image_handle_map.get(name) {
            if let Some(atlas) = self.sprite_texture_atlas_map.get(name) {
                return Some((handle, atlas));
            }
        }
        None
    }
}
