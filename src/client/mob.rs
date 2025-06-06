use bevy::prelude::*;

use bevy::utils::HashMap;
use game_test::engine::entity::EngineEntity;
use game_test::engine::mob::MobEntity;

use game_test::mob::MOB_DATA;

use crate::animated_sprite::AnimatedSprite;
use crate::ActiveGameEngine;
use crate::GameEntityComponent;

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
            // .add_systems(FixedUpdate, handle_mob_change)
            .add_systems(Update, animate_mobs);
        // .add_systems(Update, handle_mob_damage)
        // .add_systems(Update, animate_mob_damage);
    }
}

// fn animate_mob_damage(
//     mut commands: Commands,
//     mut damage_query: Query<(Entity, &DamageText, &mut Transform, &mut TextColor)>,
//     time: Res<Time>,
// ) {
//     let current_time = time.elapsed_secs_f64();
//     for (entity, text, mut transform, mut color) in &mut damage_query {
//         if current_time - text.created_at > 0.7 {
//             let new_alpha = color.0.alpha() - 0.05;
//             color.0.set_alpha(new_alpha);
//         }
//         if current_time - text.created_at > 3.0 {
//             commands.entity(entity).despawn();
//         } else {
//             transform.translation.y += 1.0;
//         }
//     }
// }

// fn handle_mob_damage(
//     mob_registry: Res<MobRegistry>,
//     mut action_events: EventReader<NetworkMessage>,
//     mut commands: Commands,
//     mut mob_query: Query<(&mut MobEntity, &Transform)>,
//     time: Res<Time>,
// ) {
//     for event in action_events.read() {
//         if let Response::MobDamage(id, amount) = &event.0 {
//             if let Some(&entity) = mob_registry.mobs.get(id) {
//                 if let Ok((mut mob, transform)) = mob_query.get_mut(entity) {
//                     if amount >= &mob.mob.health {
//                         mob.mob.health = 0;
//                         println!("killed entity {}", mob.mob.id);
//                         commands.entity(entity).despawn_recursive();
//                     } else {
//                         mob.mob.health -= amount;
//                     }
//                     let data = MOB_DATA.get(&mob.mob.mob_type).unwrap();
//                     commands.spawn((
//                         DamageText {
//                             created_at: time.elapsed_secs_f64(),
//                         },
//                         Transform::from_translation(
//                             transform.translation + Vec3::new(0.0, data.size.y + 10.0, 99.0),
//                         ),
//                         Text2d::new(format!("{}", amount)),
//                         TextColor(Color::srgba(1., 0.0, 0.2, 1.0)),
//                         TextFont {
//                             font_size: 25.0,
//                             ..default()
//                         },
//                     ));
//                 }
//             }
//         }
//     }
// }

fn animate_mobs(
    mut query: Query<(
        &GameEntityComponent,
        &MobComponent,
        &mut AnimatedSprite,
        &mut Sprite,
    )>,
    active_game_engine: Res<ActiveGameEngine>,
) {
    for (e, mob, mut animated_sprite, mut sprite) in &mut query {
        let entity = active_game_engine.0.entities.get(&e.entity_id);
        if entity.is_none() {
            continue;
        }
        let entity = entity.unwrap();
        if let EngineEntity::Mob(mob_data) = &entity {
            let data = MOB_DATA.get(&mob_data.mob_type).unwrap();
            if mob_data.velocity.x.abs() < 0.1 {
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
        } else {
            println!("WARNING: MobComponent is keyed to a non-mob engine entity");
            continue;
        };
    }
}

#[derive(Component)]
pub struct MobComponent {
    pub standing_texture: Handle<Image>,
    pub standing_texture_atlas_layout: Handle<TextureAtlasLayout>,
    pub walking_texture: Handle<Image>,
    pub walking_texture_atlas_layout: Handle<TextureAtlasLayout>,
}

impl MobComponent {
    pub fn new(
        mob: MobEntity,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) -> (Self, AnimatedSprite, Sprite) {
        let data = MOB_DATA.get(&mob.mob_type).unwrap();
        let standing_texture = asset_server.load(data.standing.sprite_sheet.clone());
        let walking_texture: Handle<Image> = asset_server.load(data.walking.sprite_sheet.clone());
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
            MobComponent {
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
