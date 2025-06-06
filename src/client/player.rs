use bevy::prelude::*;

use game_test::action::Action;
use game_test::action::PlayerState;
use game_test::action::Response;
use game_test::actor::move_x;
use game_test::actor::move_y;
use game_test::engine::entity::Entity;
use game_test::engine::entity::EntityInput;
use game_test::timestamp;
use game_test::STEP_DELAY;
use game_test::TICK_RATE_S;

use crate::animated_sprite::AnimatedSprite;
use crate::ActiveGameEngine;
use crate::ActivePlayerEntityId;

use super::map::ActiveMap;
use super::network::NetworkAction;
use super::GameState;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct PlayerComponent {
    pub state: PlayerState,
    pub entity_id: u128,
}

impl PlayerComponent {
    pub fn default_sprite(
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) -> (AnimatedSprite, Sprite) {
        let texture = asset_server.load("banana.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::splat(52), 2, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        (
            AnimatedSprite {
                fps: 2,
                frame_count: 2,
                time: 0.0,
            },
            Sprite {
                image: texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout,
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
    let entity = active_game_engine.0.entities.get(active_player_entity_id);
    if entity.is_none() {
        println!("WARNING: no entity for input");
        return;
    }
    let entity = entity.unwrap();
    let input = EntityInput {
        jump: false,
        move_left: keyboard.pressed(KeyCode::ArrowLeft),
        move_right: keyboard.pressed(KeyCode::ArrowRight),
        crouch: keyboard.pressed(KeyCode::ArrowDown),
        attack: keyboard.pressed(KeyCode::KeyA),
    };
    if let Some(last_input) = active_game_engine.0.latest_input(active_player_entity_id) {
        if last_input == input {
            return;
        }
    }
    // send the new input to the server
    action_events.send(NetworkAction(Action::PlayerInput(
        // 30 is map_instance::STEP_DELAY
        active_game_engine.0.step_index + STEP_DELAY,
        entity.position(),
        input.clone(),
    )));
    active_game_engine
        .0
        .register_input(None, *active_player_entity_id, input);
}
