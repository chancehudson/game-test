use std::collections::HashMap;
use std::fs;

use macroquad::prelude::*;
use once_cell::sync::OnceCell;
use walkdir::WalkDir;

static ASSET_BUFFER: OnceCell<AssetBuffer> = OnceCell::new();

/// Singleton hashmap that stores all assets in memory
/// Textures are keyed by their path relative to the assets directory
/// e.g. assets/stick.png
pub struct AssetBuffer {
    pub assets: HashMap<String, Texture2D>,
    pub fonts: HashMap<String, Font>,
}

impl AssetBuffer {
    /// Load all png files in the assets directory
    /// Key them as their path relative to the assets directory
    /// e.g. assets/stick.png
    pub async fn init() -> anyhow::Result<()> {
        let mut asset_buffer = AssetBuffer {
            assets: HashMap::new(),
            fonts: HashMap::new(),
        };
        println!("Loading assets...");
        for entry in WalkDir::new("assets") {
            let entry = entry?;
            let path = entry.path();
            let path_str = path.to_str().unwrap();

            if let Some(extension) = path.extension() {
                if extension != "png" {
                    continue;
                }

                if let Some(_file_name) = entry.file_name().to_str() {
                    asset_buffer
                        .assets
                        .insert(path_str.to_string(), load_texture(path_str).await?);
                }
            }
        }
        println!("Done loading assets!");
        let fonts_dir = fs::read_dir("assets/fonts")?;
        for entry in fonts_dir {
            let entry = entry?;
            let path = entry.path();
            let path_str = path.to_str().unwrap();

            if let Some(extension) = path.extension() {
                if extension != "ttf" {
                    continue;
                }
                if let Some(name) = path.file_stem() {
                    let name = name.to_str().unwrap();
                    let font = load_ttf_font(path_str).await?;
                    asset_buffer
                        .fonts
                        .insert(path_str.to_string(), font.clone());
                    asset_buffer.fonts.insert(name.to_string(), font.clone());
                }
            }
        }
        ASSET_BUFFER.set(asset_buffer).ok();
        println!("Done loading fonts!");
        Ok(())
    }

    pub fn font(name: &str) -> Option<&Font> {
        ASSET_BUFFER.get().unwrap().fonts.get(name)
    }

    pub fn texture(name: &str) -> Texture2D {
        if let Some(texture) = ASSET_BUFFER.get().unwrap().assets.get(name).cloned() {
            texture
        } else {
            Texture2D::empty()
            // panic!("Texture not found: {name}");
        }
    }
}
