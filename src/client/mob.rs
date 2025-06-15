use bevy::prelude::*;

use game_test::engine::entity::EngineEntity;
use game_test::engine::entity::mob::MobEntity;

use crate::plugins::animated_sprite::AnimatedSprite;
use crate::plugins::engine::ActiveGameEngine;
use crate::plugins::engine::GameEntityComponent;
use crate::sprite_data_loader::SpriteDataAsset;
use crate::sprite_data_loader::SpriteManager;

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            animate_mobs.run_if(in_state(crate::GameState::OnMap)),
        );
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
    sprite_data: Res<Assets<SpriteDataAsset>>,
    sprite_manager: Res<SpriteManager>,
) {
    for (e, mob, mut animated_sprite, mut sprite) in &mut query {
        let entity = active_game_engine.0.entities.get(&e.entity_id);
        if entity.is_none() {
            continue;
        }
        let entity = entity.unwrap();
        if let EngineEntity::Mob(mob_data) = &entity {
            let data = sprite_manager
                .sprite_data_maybe(&mob_data.mob_type, &sprite_data)
                .unwrap();
            if mob_data.velocity.x.abs() < 1 {
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
        sprite_data: &Res<Assets<SpriteDataAsset>>,
        sprite_manager: &SpriteManager,
    ) -> (Self, AnimatedSprite, Sprite) {
        let data = sprite_manager
            .sprite_data_maybe(&mob.mob_type, sprite_data)
            .unwrap();
        let (walking_handle, walking_atlas) =
            sprite_manager.sprite(&data.walking.sprite_sheet).unwrap();
        let (standing_handle, standing_atlas) =
            sprite_manager.sprite(&data.standing.sprite_sheet).unwrap();
        (
            MobComponent {
                standing_texture: standing_handle.clone(),
                walking_texture: walking_handle.clone(),
                standing_texture_atlas_layout: standing_atlas.clone(),
                walking_texture_atlas_layout: walking_atlas.clone(),
            },
            AnimatedSprite {
                frame_count: data.standing.frame_count as u8,
                fps: data.standing.fps as u8,
                time: 0.0,
            },
            Sprite {
                image: standing_handle.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: standing_atlas.clone(),
                    index: 0,
                }),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
        )
    }
}
