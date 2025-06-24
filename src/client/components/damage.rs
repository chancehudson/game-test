use bevy::prelude::*;
use game_test::engine::entity::{EEntity, mob::MobEntity, player::PlayerEntity};

use crate::plugins::engine::ActiveGameEngine;

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animation_system);
    }
}

#[derive(Component)]
pub struct DamageComponent {
    pub disappears_at_step: u64,
}

impl DamageComponent {
    pub fn mob_damage(
        step_index: u64,
        mob: &MobEntity,
        amount: u64,
    ) -> (DamageComponent, Text2d, TextColor, TextFont, Transform) {
        let mob_center = mob.center();
        (
            DamageComponent {
                disappears_at_step: step_index + 30,
            },
            Text2d(amount.to_string()),
            TextColor(Color::srgb(1.0, 0., 0.)),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            Transform::from_translation(Vec3::new(
                mob_center.x as f32,
                (mob_center.y + mob.size.y + 10) as f32,
                100.0,
            )),
        )
    }

    pub fn player_damage(
        step_index: u64,
        player: &PlayerEntity,
        amount: u64,
    ) -> (DamageComponent, Text2d, TextColor, TextFont, Transform) {
        let player_center = player.center();
        (
            DamageComponent {
                disappears_at_step: step_index + 30,
            },
            Text2d(amount.to_string()),
            TextColor(Color::srgb(0.627, 0.125, 0.941)),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            Transform::from_translation(Vec3::new(
                player_center.x as f32,
                (player_center.y + player.size.y + 10) as f32,
                100.0,
            )),
        )
    }
}

fn animation_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &DamageComponent)>,
    active_engine: Res<ActiveGameEngine>,
) {
    for (entity, mut transform, damage) in query.iter_mut() {
        transform.translation.y += 1.;
        if active_engine.0.step_index >= damage.disappears_at_step {
            commands.entity(entity).despawn();
        }
    }
}
