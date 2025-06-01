use bevy::math::VectorSpace;
use bevy::prelude::*;

use bevy::transform;
use bevy::utils::HashMap;
use game_test::actor::GRAVITY_ACCEL;
use game_test::MapData;
use game_test::TICK_RATE_MS;
use game_test::TICK_RATE_S_F32;
use websocket::websocket_base::header::names;

use super::move_x;
use super::move_y;
use super::NetworkMessage;
use game_test::action::Response;
use game_test::mob::{MobData, MOB_DATA};

use crate::animated_sprite::AnimatedSprite;
use crate::mob_health_bar::MobHealthBar;

pub struct MobPlugin;

#[derive(Resource, Default)]
pub struct MobRegistry {
    pub mobs: HashMap<u64, Entity>,
}

#[derive(Component)]
pub struct DamageText {
    pub created_at: f64,
}

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MobRegistry>()
            .add_systems(FixedUpdate, handle_mob_change)
            .add_systems(Update, animate_mobs)
            .add_systems(Update, handle_mob_damage)
            .add_systems(Update, animate_mob_damage);
    }
}

fn handle_mob_change(
    mut action_events: EventReader<NetworkMessage>,
    mob_registry: Res<MobRegistry>,
    mut mob_query: Query<(&mut MobEntity, &Transform)>,
) {
    for event in action_events.read() {
        if let Response::MobChange(new_mob) = &event.0 {
            // We assume the mob is on map here. If it's not this is a noop
            if let Some(&entity) = mob_registry.mobs.get(&new_mob.id) {
                if let Ok((mut existing_mob, transform)) = mob_query.get_mut(entity) {
                    if (transform.translation.xy()).abs_diff_eq(new_mob.position, 50.0) {
                        // use our local position
                        existing_mob.mob.position = transform.translation.xy();
                    } else {
                        println!(
                            "overwriting, local mob is {}",
                            existing_mob.mob.position.x - new_mob.position.x
                        );
                        existing_mob.mob.position = new_mob.position;
                    }
                    existing_mob.tick(new_mob);
                }
            }
        }
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
    mut mob_query: Query<(&mut MobEntity, &Transform, Entity)>,
    time: Res<Time>,
) {
    for event in action_events.read() {
        if let Response::MobDamage(id, amount) = &event.0 {
            for (mut entity, transform, e) in mob_query.iter_mut() {
                if &entity.mob.id != id {
                    continue;
                }
                if amount >= &entity.mob.health {
                    entity.mob.health = 0;
                    println!("killed entity {}", entity.mob.id);
                    commands.entity(e).despawn_recursive();
                } else {
                    entity.mob.health -= amount;
                }
                let data = MOB_DATA.get(&entity.mob.mob_type).unwrap();
                commands.spawn((
                    DamageText {
                        created_at: time.elapsed_secs_f64(),
                    },
                    Transform::from_translation(
                        transform.translation + Vec3::new(0.0, data.size.y + 10.0, 99.0),
                    ),
                    Text2d::new(format!("{}", amount)),
                    TextColor(Color::srgba(1., 0.0, 0.2, 1.0)),
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
        let data = MOB_DATA.get(&mob.mob.mob_type).unwrap();
        if mob.velocity.x.abs() < 0.1 {
            if sprite.image != mob.standing_texture {
                sprite.image = mob.standing_texture.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: mob.standing_texture_atlas_layout.clone(),
                    index: 0,
                });
                animated_sprite.fps = data.standing.fps as u8;
                animated_sprite.frame_count = data.standing.frame_count as u8;
            }
        } else {
            if sprite.image != mob.walking_texture {
                sprite.image = mob.walking_texture.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: mob.walking_texture_atlas_layout.clone(),
                    index: 0,
                });
                animated_sprite.fps = data.walking.fps as u8;
                animated_sprite.frame_count = data.walking.frame_count as u8;
            }
        }
        if mob.velocity.x > 0. {
            sprite.flip_x = true;
        } else if mob.velocity.x < 0. {
            sprite.flip_x = false;
        }
    }
}

#[derive(Component)]
pub struct MobEntity {
    pub mob: MobData,
    pub velocity: Vec2,
    pub standing_texture: Handle<Image>,
    pub standing_texture_atlas_layout: Handle<TextureAtlasLayout>,
    pub walking_texture: Handle<Image>,
    pub walking_texture_atlas_layout: Handle<TextureAtlasLayout>,
}

impl MobEntity {
    pub fn new(
        mob: MobData,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) -> (Self, AnimatedSprite, Sprite) {
        let data = MOB_DATA.get(&mob.mob_type).unwrap();
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
                velocity: Vec2::ZERO,
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

    pub fn tick(&mut self, new_mob: &MobData) {
        self.mob.next_position = new_mob.next_position;
        if self.mob.position == self.mob.next_position {
            self.velocity = Vec2::ZERO;
        } else {
            self.velocity.x = (self.mob.next_position.x - self.mob.position.x) / TICK_RATE_S_F32;
        }
    }

    pub fn step(&mut self, step_len: f32, map: &MapData) {
        let data = MOB_DATA.get(&self.mob.mob_type).unwrap();
        self.velocity.y += -GRAVITY_ACCEL * step_len;
        let rect = Rect::new(
            self.mob.position.x,
            self.mob.position.y,
            self.mob.position.x + data.size.x,
            self.mob.position.y + data.size.y,
        );
        let (new_x, _) = move_x(rect, self.velocity, step_len * self.velocity.x, map);
        let (new_y, vel_y) = move_y(rect, self.velocity, step_len * self.velocity.y, map);
        self.mob.position = Vec2::new(new_x, new_y);
        self.velocity = Vec2::new(self.velocity.x, vel_y);
        // to avoid slight stutters between reaching the target coords and
        // receiving new ones
        const OVERRUN_DIST: f32 = 0.0;
        if (self.velocity.x > 0. && self.mob.position.x > self.mob.next_position.x + OVERRUN_DIST)
            || (self.velocity.x < 0.
                && self.mob.position.x + OVERRUN_DIST < self.mob.next_position.x)
        {
            self.mob.position.x = self.mob.next_position.x;
            self.velocity.x = 0.;
        }
        if (self.velocity.y > 0. && self.mob.position.y > self.mob.next_position.y)
            || (self.velocity.y < 0. && self.mob.position.y < self.mob.next_position.y)
        {
            self.mob.position.y = self.mob.next_position.y;
            self.velocity.y = 0.;
        }
    }
}
