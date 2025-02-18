use macroquad::prelude::*;

use super::AssetBuffer;

pub struct Sprite {
    texture: Texture2D,
    width: f32,
    height: f32,
}

impl Sprite {
    // Create a new sprite from the spritesheet
    pub fn new(texture_name: &str, frame_count: usize) -> Self {
        // Don't filter the texture, keeping pixel art sharp
        let texture = AssetBuffer::texture(texture_name);
        texture.set_filter(FilterMode::Nearest);
        let sprite_width = texture.width() / frame_count as f32;
        Self {
            width: sprite_width,
            height: texture.height(),
            texture,
        }
    }

    // Draw a specific frame from the sprite sheet
    pub fn draw_frame(&self, frame: usize, x: f32, y: f32, flip_x: bool) {
        let frames_per_row = (self.texture.width() / self.width) as usize;
        let row = frame / frames_per_row;
        let col = frame % frames_per_row;

        let source_rect = Rect::new(
            col as f32 * self.width,
            row as f32 * self.height,
            self.width,
            self.height,
        );

        draw_texture_ex(
            &self.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                source: Some(source_rect),
                dest_size: Some(Vec2::new(self.width, self.height)),
                flip_x,
                ..Default::default()
            },
        );
    }
}

// Example structure for an animated entity
pub struct AnimatedEntity {
    sprite: Sprite,
    pub position: Vec2,
    pub flip_x: bool,
    current_frame: usize,
    frame_timer: f32,
    animation_fps: f32,
    frame_count: usize,
    current_animation: usize,
}

impl AnimatedEntity {
    pub fn new(sprite_path: &str, frame_count: usize) -> Self {
        Self {
            sprite: Sprite::new(sprite_path, frame_count),
            flip_x: false,
            position: Vec2::new(0.0, 0.0),
            current_frame: 0,
            frame_timer: 0.0,
            animation_fps: 5.0,
            frame_count,
            current_animation: 0,
        }
    }

    pub fn update(&mut self) {
        self.frame_timer += get_frame_time();
        if self.frame_timer >= 1.0 / self.animation_fps {
            self.frame_timer = 0.0;
            self.current_frame = (self.current_frame + 1) % self.frame_count;
        }
    }

    pub fn draw(&self) {
        let frame = self.current_animation * self.frame_count + self.current_frame;
        self.sprite
            .draw_frame(frame, self.position.x, self.position.y, self.flip_x);
        // draw_rectangle(self.position.x, self.position.y, 50., 50., PINK);
    }

    // Change the current animation (e.g., walking, jumping)
    pub fn set_animation(&mut self, animation_index: usize) {
        if self.current_animation != animation_index {
            self.current_animation = animation_index;
            self.current_frame = 0;
            self.frame_timer = 0.0;
        }
    }
}
