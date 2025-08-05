use bevy::prelude::*;

pub struct AnimatedSpritePlugin;

#[derive(Component)]
pub struct AnimatedSprite {
    pub frame_count: u8,
    pub fps: u8,
    pub time: f32,
}

impl Plugin for AnimatedSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_sprites);
    }
}

fn animate_sprites(mut query: Query<(&mut Sprite, &mut AnimatedSprite)>, time: Res<Time>) {
    let delta = time.delta_secs();
    for (mut sprite, mut animation) in query.iter_mut() {
        animation.time += delta;
        let seconds_per_frame = 1. / (animation.fps as f32);
        let frame_index =
            ((animation.time / seconds_per_frame).floor() as u32) % (animation.frame_count as u32);
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = frame_index as usize;
        }
    }
}
