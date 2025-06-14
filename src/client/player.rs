use bevy::prelude::*;

use game_test::action::Action;
use game_test::action::PlayerState;
use game_test::engine::entity::EntityInput;
use game_test::engine::game_event::GameEvent;
use game_test::generate_strong_u128;

use crate::animated_sprite::AnimatedSprite;
use crate::sprite_data_loader::SpriteManager;
use crate::ActiveGameEngine;
use crate::ActivePlayerEntityId;

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

fn input_system(
    active_player_entity_id: Res<ActivePlayerEntityId>,
    mut active_game_engine: ResMut<ActiveGameEngine>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action_events: EventWriter<NetworkAction>,
) {
    if active_player_entity_id.0.is_none() {
        println!("WARNING: no active entity id for player");
        return;
    }
    let active_player_entity_id = active_player_entity_id.0.as_ref().unwrap();
    let input = EntityInput {
        jump: keyboard.pressed(KeyCode::Space),
        move_left: keyboard.pressed(KeyCode::ArrowLeft),
        move_right: keyboard.pressed(KeyCode::ArrowRight),
        crouch: keyboard.pressed(KeyCode::ArrowDown),
        attack: keyboard.pressed(KeyCode::KeyA),
        enter_portal: keyboard.pressed(KeyCode::ArrowUp),
        admin_enable_debug_markers: keyboard.pressed(KeyCode::Digit9),
        show_emoji: keyboard.pressed(KeyCode::KeyQ),
    };
    let (latest_input_step, latest_input) =
        active_game_engine.0.latest_input(active_player_entity_id);
    if latest_input == input {
        return;
    }
    let event = GameEvent::Input {
        id: generate_strong_u128(),
        input: input.clone(),
        entity_id: *active_player_entity_id,
        universal: true,
    };
    active_game_engine.0.register_event(None, event.clone());
    // send the new input to the server
    action_events.write(NetworkAction(Action::EngineEvent(
        active_game_engine.0.id,
        event,
        active_game_engine.0.step_index,
    )));
}
