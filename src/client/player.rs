use bevy::prelude::*;

use game_test::action::Action;
use game_test::action::PlayerState;
use game_test::engine::entity::player::PlayerEntity;
use game_test::engine::entity::text::TextEntity;
use game_test::engine::entity::EngineEntity;
use game_test::engine::entity::EntityInput;
use game_test::engine::game_event::GameEvent;

use crate::animated_sprite::AnimatedSprite;
use crate::sprite_data_loader::SpriteManager;
use crate::ActiveGameEngine;
use crate::ActivePlayerEntityId;
use crate::ActivePlayerState;

use super::network::NetworkAction;
use super::GameState;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct PlayerComponent {
    pub state: PlayerState,
    pub entity_id: u128,
}

impl PlayerComponent {
    pub fn default_sprite(sprite_manager: &SpriteManager) -> (AnimatedSprite, Sprite) {
        let (handle, atlas) = sprite_manager
            .sprite("sprites/banana/standing.png")
            .unwrap();

        (
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
        app.add_systems(Update, input_system.run_if(in_state(GameState::OnMap)));
        // .add_systems(OnEnter(GameState::LoggedOut), despawn_all_players);
    }
}

/// hello i'm storing keybindings complexity here
fn input_system(
    mut active_player_entity_id: ResMut<ActivePlayerEntityId>,
    mut active_game_engine: ResMut<ActiveGameEngine>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action_events: EventWriter<NetworkAction>,
    active_player_state: Res<ActivePlayerState>,
) {
    let engine = &mut active_game_engine.0;
    if active_player_state.0.is_none() {
        // this shows an error message onscreen using the local game engine
        let mut text_entity = TextEntity::new_pure(IVec2::ZERO, IVec2::splat(50));
        text_entity.text = "no player state!".to_string();
        text_entity.disappears_at_step_index = engine.step_index + 30;
        engine.spawn_entity(EngineEntity::Text(text_entity), None, false);
        return;
    }
    let player_state = active_player_state.0.as_ref().unwrap();

    // request engine reload if p key is pressed
    if keyboard.just_pressed(KeyCode::KeyP) {
        action_events.write(NetworkAction(Action::RequestEngineReload(engine.id)));
        return;
    }

    // handle spawn requests
    if keyboard.just_pressed(KeyCode::KeyJ) && active_player_entity_id.0.is_none() {
        let mut entity = PlayerEntity::new_with_ids(0, player_state.id.clone());
        entity.id = rand::random();
        active_player_entity_id.0 = Some(entity.id);
        entity.is_active = true;
        entity.position = engine.map.spawn_location;
        let spawn_event = GameEvent::SpawnEntity {
            id: rand::random(),
            entity: EngineEntity::Player(entity),
            universal: true,
        };
        // register the event locally
        engine.register_event(None, spawn_event.clone());
        // send the new input to the server
        action_events.write(NetworkAction(Action::EngineEvent(
            engine.id,
            spawn_event,
            engine.step_index,
        )));
    }

    if keyboard.just_pressed(KeyCode::KeyK) && active_player_entity_id.0.is_some() {
        let despawn_event = GameEvent::RemoveEntity {
            id: rand::random(),
            entity_id: active_player_entity_id.0.unwrap(),
            universal: true,
        };
        // register the event locally
        engine.register_event(None, despawn_event.clone());
        // send the new input to the server
        action_events.write(NetworkAction(Action::EngineEvent(
            engine.id,
            despawn_event,
            engine.step_index,
        )));
        active_player_entity_id.0 = None;
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
        };
        let (_latest_input_step, latest_input) = engine.latest_input(&entity_id);
        if latest_input == input {
            return;
        }
        let input_event = GameEvent::Input {
            id: rand::random(), // generate a random value, will receive actual value in future ?
            input: input.clone(),
            entity_id,
            universal: true,
        };
        // register here, will get confirmation with an id change?
        // for now, no
        engine.register_event(None, input_event.clone());
        // send the new input to the server
        action_events.write(NetworkAction(Action::EngineEvent(
            engine.id,
            input_event,
            engine.step_index,
        )));
    }
}
