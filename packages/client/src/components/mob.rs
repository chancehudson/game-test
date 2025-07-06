use bevy::prelude::*;

use game_common::entity::EngineEntity;
use game_common::entity::mob::MobEntity;

use crate::components::damage::DamageComponent;
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
            (animate_mobs, animate_left_right_system, damage_text_system),
        );
    }
}

fn damage_text_system(
    mut commands: Commands,
    mut entity_query: Query<&GameEntityComponent, With<MobComponent>>,
    active_engine: Res<ActiveGameEngine>,
) {
    let engine = &active_engine.0;
    for entity in entity_query.iter_mut() {
        if let Some(entity) = &entity.entity {
            match entity {
                EngineEntity::Mob(p) => {
                    for amount in &p.received_damage_this_step {
                        if let Some((_aggro_to_entity_id, _)) = p.aggro_to {
                            commands.spawn(DamageComponent::mob_damage(
                                engine.step_index,
                                &p,
                                *amount,
                            ));
                        } else {
                            unreachable!();
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

fn animate_left_right_system(
    mut entity_query: Query<(&GameEntityComponent, &mut Sprite), With<MobComponent>>,
) {
    for (entity, mut sprite) in entity_query.iter_mut() {
        if let Some(entity) = &entity.entity {
            match entity {
                EngineEntity::Mob(p) => {
                    if p.moving_sign > 0 {
                        sprite.flip_x = true;
                    }
                    if p.moving_sign < 0 {
                        sprite.flip_x = false;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

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
                // color: Color::BLACK,
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
        )
    }
}
