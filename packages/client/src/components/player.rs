use bevy::prelude::*;

use engine::entity::EngineEntity;
use engine::entity::EntityInput;
use engine::game_event::EngineEvent;
use game_common::action::Action;

use crate::components::damage::DamageComponent;
use crate::plugins::animated_sprite::AnimatedSprite;
use crate::plugins::engine::ActiveGameEngine;
use crate::plugins::engine::ActivePlayerEntityId;
use crate::plugins::engine::GameEntityComponent;
use crate::sprite_data_loader::SpriteManager;

use crate::GameState;
use crate::network::NetworkAction;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct PlayerComponent;

impl PlayerComponent {
    pub fn default_sprite(
        sprite_manager: &SpriteManager,
    ) -> (PlayerComponent, AnimatedSprite, Sprite) {
        let (handle, atlas) = sprite_manager
            .sprite("sprites/banana/standing.png")
            .unwrap();

        (
            PlayerComponent,
            AnimatedSprite {
                fps: 2,
                frame_count: 2,
                time: 0.0,
            },
            Sprite {
                image: handle.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas.clone(),
                    index: 0,
                }),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
        )
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animation_system,
                input_system,
                iframe_blink_system,
                damage_text_system,
            )
                .run_if(in_state(GameState::OnMap)),
        );
        // .add_systems(OnEnter(GameState::LoggedOut), despawn_all_players);
    }
}

fn animation_system(
    mut entity_query: Query<(&GameEntityComponent, &mut Sprite), With<PlayerComponent>>,
) {
    for (entity, mut sprite) in entity_query.iter_mut() {
        if let Some(entity) = &entity.entity {
            match entity {
                EngineEntity::Player(p) => {
                    sprite.flip_x = !p.facing_left;
                }
                _ => unreachable!(),
            }
        }
    }
}

fn damage_text_system(
    mut commands: Commands,
    mut entity_query: Query<(&GameEntityComponent), With<PlayerComponent>>,
    active_engine: Res<ActiveGameEngine>,
) {
    let engine = &active_engine.0;
    for entity in entity_query.iter_mut() {
        if let Some(entity) = &entity.entity {
            match entity {
                EngineEntity::Player(p) => {
                    if p.received_damage_this_step.0 {
                        commands.spawn(DamageComponent::player_damage(
                            engine.step_index,
                            &p,
                            p.received_damage_this_step.1,
                        ));
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

fn iframe_blink_system(
    mut entity_query: Query<(&GameEntityComponent, &mut Sprite), With<PlayerComponent>>,
    active_engine: Res<ActiveGameEngine>,
) {
    let blink_step_interval = 8;
    let blink = (active_engine.0.step_index / blink_step_interval) % 2 == 0;
    for (entity, mut sprite) in entity_query.iter_mut() {
        if let Some(entity) = &entity.entity {
            match entity {
                EngineEntity::Player(p) => {
                    if let Some(_) = p.receiving_damage_until {
                        let alpha = if blink { 0.4 } else { 1.0 };
                        sprite.color.set_alpha(alpha);
                    } else {
                        sprite.color.set_alpha(1.0);
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

/// hello i'm storing keybindings complexity here
fn input_system(
    active_player_entity_id: Res<ActivePlayerEntityId>,
    mut active_game_engine: ResMut<ActiveGameEngine>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action_events: EventWriter<NetworkAction>,
) {
    let engine = &mut active_game_engine.0;
    // request engine reload if p key is pressed
    if keyboard.just_pressed(KeyCode::KeyP) {
        action_events.write(NetworkAction(Action::RequestEngineReload(
            engine.id,
            engine.step_index,
        )));
        return;
    }

    // allow general input if spawned
    if let Some(entity_id) = active_player_entity_id.0 {
        // input currently being received
        let input = EntityInput {
            jump: keyboard.pressed(KeyCode::Space),
            move_left: keyboard.pressed(KeyCode::ArrowLeft),
            move_right: keyboard.pressed(KeyCode::ArrowRight),
            crouch: keyboard.pressed(KeyCode::ArrowDown),
            attack: keyboard.just_pressed(KeyCode::KeyA),
            enter_portal: keyboard.pressed(KeyCode::ArrowUp),
            admin_enable_debug_markers: keyboard.just_pressed(KeyCode::Digit9),
            show_emoji: keyboard.just_pressed(KeyCode::KeyQ),
            respawn: keyboard.just_pressed(KeyCode::KeyR),
        };
        let (_latest_input_step, latest_input) = engine.latest_input(&entity_id);
        if latest_input == input {
            return;
        }
        let input_event = EngineEvent::Input {
            id: rand::random(), // generate a random value, will receive actual value in future ?
            input: input.clone(),
            entity_id,
            universal: true,
        };
        // register here, will get confirmation with an id change?
        // for now, no
        engine.register_event(None, input_event.clone());
        // send the new input to the server
        action_events.write(NetworkAction(Action::RemoteEngineEvent(
            engine.id,
            input_event,
            engine.step_index,
        )));
    }
}
