use std::{collections::HashMap, fs};
use std::sync::Mutex;

use macroquad::prelude::*;
use once_cell::sync::Lazy;

static ASSET_BUFFER: Lazy<Mutex<HashMap<String, Texture2D>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct AssetBuffer {
    pub assets: HashMap<String, Texture2D>,
}

impl AssetBuffer {
    /// Load all png files in the assets directory
    /// Key them as their path relative to the assets directory
    /// e.g. assets/stick.png
    pub async fn init() -> anyhow::Result<()> {
        println!("Loading assets...");
        let mut assets = ASSET_BUFFER.lock().unwrap();
        let assets_dir = fs::read_dir("assets")?;
        for entry in assets_dir {
            let entry = entry?;
            let path = entry.path();
            let path_str = path.to_str().unwrap();

            if let Some(extension) = path.extension() {
                if extension != "png" {
                    continue;
                }

                if let Some(_file_name) = entry.file_name().to_str() {
                    assets.insert(path_str.to_string(), load_texture(path_str).await?);
                }
            }
        }
        println!("Done loading assets!");
        Ok(())
    }

    pub async fn reload_assets() -> anyhow::Result<()> {
        {
            let mut assets = ASSET_BUFFER.lock().unwrap();
            assets.clear();
        }
        AssetBuffer::init().await
    }

    pub fn texture(name: &str) -> Texture2D {
        if let Some(texture) = ASSET_BUFFER.lock().unwrap().get(name).cloned() {
            texture
        } else {
            panic!("Texture not found: {name}");
        }
    }
}
