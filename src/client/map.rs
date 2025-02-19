use macroquad::prelude::*;

use game_test::Actor;

use crate::renderable::MobRenderable;
use crate::Renderable;

use super::AssetBuffer;
use super::MapData;

/// We'll separate solids and visuals
pub struct Map {
    pub solids: Vec<Rect>,
    pub entities: Vec<MobRenderable>,
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
            solids: data
                .platforms
                .iter()
                .map(|p| Rect::new(p.position.x, p.position.y, p.size.x, p.size.y))
                .collect::<_>(),
            size: data.size,
            entities: vec![],
            data,
        }
    }

    pub fn render_portals(&self) {
        for portal in &self.data.portals {
            let r = portal.rect();
            draw_rectangle(r.x, r.y, r.w, r.h, RED);
        }
    }

    pub fn step_physics(&mut self, step_len: f32) {
        for entity in self.entities.iter_mut() {
            entity.step_physics(step_len, &self.data);
        }
    }

    pub fn render(&mut self, step_len: f32, player_pos: Vec2) {
        push_camera_state();
        set_default_camera();

        let scale = Vec2::new(0.7, 0.7);
        let x_size = f32::max(self.background_texture.size().x * scale.x, screen_width());
        let y_size = f32::max(self.background_texture.size().y * scale.y, screen_height());
        let x_range = x_size - screen_width();
        let y_range = y_size - screen_height();
        let offset_x = (player_pos.x.clamp(0., self.size.x) / self.size.x) * x_range;
        let offset_y = (player_pos.y.clamp(0., self.size.y) / self.size.y) * y_range;
        draw_texture_ex(
            &self.background_texture,
            -offset_x,
            -offset_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(x_size, y_size)),
                ..Default::default()
            },
        );
        pop_camera_state();

        for solid in &self.solids {
            draw_rectangle(solid.x, solid.y, solid.w, solid.h, BLUE);
        }

        for npc in &self.data.npc {
            let sprite = AssetBuffer::texture(&npc.asset);
            draw_texture_ex(
                &sprite,
                npc.position.x,
                npc.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(npc.size),
                    ..Default::default()
                },
            );
        }

        for entity in &mut self.entities {
            entity.render(step_len);
        }

        // draw_circle(
        //     screen_width() - 300.0,
        //     screen_height() - 300.0,
        //     15.0,
        //     YELLOW,
        // );
        // // custom rendering
        // draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        // draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);

        // draw_text("IT WORKS!", 20.0, 80.0, 30.0, DARKGRAY);

        self.render_portals();
    }
}
