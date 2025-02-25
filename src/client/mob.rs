use bevy::prelude::*;

use super::NetworkMessage;
use game_test::action::Response;
use game_test::mob::Mob;

use crate::animated_sprite::AnimatedSprite;

pub struct MobPlugin;

#[derive(Component)]
pub struct DamageText {
    pub created_at: f64,
}

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_mobs)
            .add_systems(Update, handle_mob_damage)
            .add_systems(Update, animate_mob_damage);
    }
}

fn animate_mob_damage(
    mut commands: Commands,
    mut damage_query: Query<(Entity, &DamageText, &mut Transform, &mut TextColor)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs_f64();
    for (entity, text, mut transform, mut color) in &mut damage_query {
        if current_time - text.created_at > 0.7 {
            let new_alpha = color.0.alpha() - 0.05;
            color.0.set_alpha(new_alpha);
        }
        if current_time - text.created_at > 3.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation.y += 1.0;
        }
    }
}

fn handle_mob_damage(
    mut action_events: EventReader<NetworkMessage>,
    mut commands: Commands,
    mob_query: Query<(&MobEntity, &Transform)>,
    time: Res<Time>,
) {
    for event in action_events.read() {
        if let Response::MobDamage(id, amount) = &event.0 {
            for (entity, transform) in &mob_query {
                if &entity.mob.id != id {
                    continue;
                }
                commands.spawn((
                    DamageText {
                        created_at: time.elapsed_secs_f64(),
                    },
                    Transform::from_translation(
                        transform.translation
                            + Vec3::new(0.0, entity.mob.data().size.y + 10.0, 99.0),
                    ),
                    Text2d::new(format!("{}", amount)),
                    TextColor(Color::srgba(0.7, 0.0, 0.0, 1.0)),
                    TextFont {
                        font_size: 25.0,
                        ..default()
                    },
                ));
            }
        }
    }
}

fn animate_mobs(mut query: Query<(&MobEntity, &mut AnimatedSprite, &mut Sprite)>) {
    for (mob, mut animated_sprite, mut sprite) in &mut query {
        if mob.mob.velocity.x == 0.0 {
            sprite.image = mob.standing_texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: mob.standing_texture_atlas_layout.clone(),
                index: 0,
            });
            animated_sprite.fps = mob.mob.data().standing.fps as u8;
            animated_sprite.frame_count = mob.mob.data().standing.frame_count as u8;
        } else {
            sprite.image = mob.walking_texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: mob.walking_texture_atlas_layout.clone(),
                index: 0,
            });
            animated_sprite.fps = mob.mob.data().walking.fps as u8;
            animated_sprite.frame_count = mob.mob.data().walking.frame_count as u8;
        }
        if mob.mob.velocity.x > 0.0 {
            sprite.flip_x = true;
        } else if mob.mob.velocity.x < 0.0 {
            sprite.flip_x = false;
        }
    }
}

#[derive(Component)]
pub struct MobEntity {
    pub mob: Mob,
    pub standing_texture: Handle<Image>,
    pub standing_texture_atlas_layout: Handle<TextureAtlasLayout>,
    pub walking_texture: Handle<Image>,
    pub walking_texture_atlas_layout: Handle<TextureAtlasLayout>,
}

impl MobEntity {
    pub fn new(
        mob: Mob,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) -> (Self, AnimatedSprite, Sprite) {
        let data = mob.data();
        let standing_texture = asset_server.load(data.standing.sprite_sheet.clone());
        let walking_texture = asset_server.load(data.walking.sprite_sheet.clone());
        let standing_layout = TextureAtlasLayout::from_grid(
            UVec2::new(data.standing.width as u32, data.size.y as u32),
            data.standing.frame_count as u32,
            1,
            None,
            None,
        );
        let walking_layout = TextureAtlasLayout::from_grid(
            UVec2::new(data.walking.width as u32, data.size.y as u32),
            data.walking.frame_count as u32,
            1,
            None,
            None,
        );
        let standing_texture_atlas_layout = texture_atlas_layouts.add(standing_layout);
        let walking_texture_atlas_layout = texture_atlas_layouts.add(walking_layout);
        (
            MobEntity {
                mob: mob.clone(),
                standing_texture: standing_texture.clone(),
                walking_texture,
                standing_texture_atlas_layout: standing_texture_atlas_layout.clone(),
                walking_texture_atlas_layout,
            },
            AnimatedSprite {
                frame_count: data.standing.frame_count as u8,
                fps: data.standing.fps as u8,
                time: 0.0,
            },
            Sprite {
                image: standing_texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: standing_texture_atlas_layout,
                    index: 0,
                }),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
        )
    }
}
