use macroquad::prelude::*;


/// We'll separate solids and visuals
pub struct Map {
    pub solids: Vec<Rect>,
    background_texture: Texture2D,
    pub size: Vec2,
}

impl Map {
    pub async fn new() -> Self {
        Self {
            size: Vec2::new(1500., 1000.),
            background_texture: load_texture("assets/sky_background.png").await.unwrap(),
            solids: vec![
                Rect::new(0., 50., 1000., 100.),
                Rect::new(0., 250., 1000., 100.),
                Rect::new(0., 400., 1000., 100.),
                Rect::new(0., 5000., 100000., 100.),
                Rect::new(0., 1000., 1500., 10.)
            ]
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

    }
}
