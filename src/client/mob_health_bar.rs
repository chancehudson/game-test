use bevy::prelude::*;
use game_test::{mob::MOB_DATA, MobData};

use crate::mob::MobEntity;

pub struct MobHealthBarPlugin;

#[derive(Component)]
pub struct MobHealthBar;

impl Plugin for MobHealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_mob_health_bars);
    }
}

fn update_mob_health_bars(
    mob_query: Query<&MobEntity>,
    mut mob_health_bar_query: Query<(&mut Sprite, &mut Transform, &mut Parent), With<MobHealthBar>>,
) {
    for (mut sprite, _transform, parent) in mob_health_bar_query.iter_mut() {
        if let Ok(mob) = mob_query.get(parent.get()) {
            let data = MOB_DATA.get(&mob.mob.mob_type).unwrap();
            let size = Vec2::new(
                data.size.x * 2.0 * (mob.mob.health as f32 / mob.mob.max_health as f32),
                10.,
            );
            sprite.custom_size = Some(size);
        }
    }
}

impl MobHealthBar {
    pub fn new(mob: MobData) -> (Self, Transform, Sprite) {
        let data = MOB_DATA.get(&mob.mob_type).unwrap();
        let size = Vec2::new(data.size.x * 2.0, 10.);
        (
            MobHealthBar,
            // position is relative to the parent, which is the mob
            Transform::from_translation(Vec3::new(-data.size.x / 2., -10., 1.0)),
            Sprite {
                color: Color::srgb(0., 1.0, 0.),
                custom_size: Some(size),
                anchor: bevy::sprite::Anchor::TopLeft,
                ..default()
            },
        )
    }
}
