use macroquad::prelude::*;
use serde::Deserialize;
use json5::Deserializer;

use super::AssetBuffer;

// Custom deserializer for Vec2
fn deserialize_vec2<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let arr: [f32; 2] = Deserialize::deserialize(deserializer)?;
    Ok(Vec2::new(arr[0], arr[1]))
}

#[derive(Debug, Deserialize)]
pub struct Portal {
    #[serde(deserialize_with = "deserialize_vec2")]
    pub position: Vec2,
    pub to: String,
}

#[derive(Debug, Deserialize)]
pub struct Platform {
    #[serde(deserialize_with = "deserialize_vec2")]
    pub position: Vec2,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: Vec2,
}

#[derive(Debug, Deserialize)]
pub struct MapData {
    pub name: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub spawn_location: Vec2,
    background: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: Vec2,
    pub portals: Vec<Portal>,
    pub platforms: Vec<Platform>,
}

/// We'll separate solids and visuals
pub struct Map {
    pub solids: Vec<Rect>,
    pub spawn_location: Vec2,
    pub background_texture: Texture2D,
    pub data: MapData,
    pub size: Vec2,
}

impl Map {
    pub async fn new(name: &str) -> Self {
        let data_str = std::fs::read_to_string(format!("maps/{name}.json5")).unwrap();
        let data = json5::from_str::<MapData>(&data_str).unwrap();
        Self {
            spawn_location: data.spawn_location,
            background_texture: AssetBuffer::texture(&data.background),
            solids: data.platforms.iter().map(|p| Rect::new(p.position.x, p.position.y, p.size.x, p.size.y)).collect::<_>(),
            size: data.size,
            data,
        }
    }

    pub fn render_portals(&self) {
        for portal in &self.data.portals {
            draw_rectangle(portal.position.x, portal.position.y - 150., 150., 150., RED);
        }
    }

    pub fn render(&self, _step_len: f32, player_pos: Vec2) {
        push_camera_state();
        set_default_camera();

        let scale = Vec2::new(1.1, 1.1);
        let offset_x = (player_pos.x.clamp(0., self.size.x) / self.size.x) * (scale.x - 1.0) * screen_width();
        let offset_y = (player_pos.y.clamp(0., self.size.y) / self.size.y) * (scale.y - 1.0) * screen_height();
        draw_texture_ex(&self.background_texture, -offset_x, -offset_y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(scale.x * screen_width(), scale.y * screen_height())),
            ..Default::default()
        });
        pop_camera_state();

        for solid in &self.solids {
            draw_rectangle(solid.x, solid.y, solid.w, solid.h, BLUE);
        }

        draw_circle(screen_width() - 300.0, screen_height() - 300.0, 15.0, YELLOW);
        // custom rendering
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);

        draw_text("IT WORKS!", 20.0, 80.0, 30.0, DARKGRAY);

        self.render_portals();
    }
}
