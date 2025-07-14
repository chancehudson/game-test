use bevy::prelude::*;

use game_common::AnimationData;
use game_common::entity::EntityInput;
use game_common::entity::player::PlayerEntity;
use game_common::game_event::EngineEvent;
use game_common::network::Action;

use crate::components::damage::DamageComponent;
use crate::plugins::animated_sprite::AnimatedSprite;
use crate::plugins::engine::ActiveGameEngine;
use crate::plugins::engine::ActivePlayerEntityId;
use crate::plugins::engine::GameEntityComponent;
use crate::plugins::help_gui::HelpGuiState;
use crate::plugins::player_inventory::PlayerInventoryState;
use crate::plugins::text_input::TextInput;
use crate::plugins::text_input::spawn_text_input;
use crate::sprite_data_loader::SpriteManager;

use crate::GameState;
use crate::network::NetworkAction;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct PlayerComponent;

impl PlayerComponent {
    pub fn default_animation() -> AnimationData {
        AnimationData {
            frame_count: 2,
            fps: 2,
            sprite_sheet: "sprites/banana/standing.png".to_string(),
            width: 52,
            height: 52,
        }
    }

    pub fn default_sprite(
        sprite_manager: &SpriteManager,
    ) -> (PlayerComponent, AnimatedSprite, Sprite) {
        let animation = Self::default_animation();
        let (handle, atlas) = sprite_manager.atlas(&animation.sprite_sheet).unwrap();

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
    active_engine: Res<ActiveGameEngine>,
) {
    let engine = &active_engine.0;
    for (entity, mut sprite) in entity_query.iter_mut() {
        if let Some(entity) = engine.entity_by_id::<PlayerEntity>(&entity.entity_id, None) {
            sprite.flip_x = !entity.facing_left;
        }
    }
}

fn damage_text_system(
    mut commands: Commands,
    mut entity_query: Query<&GameEntityComponent, With<PlayerComponent>>,
    active_engine: Res<ActiveGameEngine>,
) {
    let engine = &active_engine.0;
    for entity in entity_query.iter_mut() {
        if let Some(entity) = engine.entity_by_id::<PlayerEntity>(&entity.entity_id, None) {
            if entity.received_damage_this_step.0 {
                commands.spawn(DamageComponent::player_damage(
                    engine.step_index,
                    &entity,
                    entity.received_damage_this_step.1,
                ));
            }
        }
    }
}

fn iframe_blink_system(
    mut entity_query: Query<(&GameEntityComponent, &mut Sprite), With<PlayerComponent>>,
    active_engine: Res<ActiveGameEngine>,
) {
    let engine = &active_engine.0;
    let blink_step_interval = 8;
    let blink = (engine.step_index / blink_step_interval) % 2 == 0;
    for (entity, mut sprite) in entity_query.iter_mut() {
        if let Some(entity) = engine.entity_by_id::<PlayerEntity>(&entity.entity_id, None) {
            if entity.receiving_damage_until.is_some() {
                let alpha = if blink { 0.4 } else { 1.0 };
                sprite.color.set_alpha(alpha);
            } else {
                sprite.color.set_alpha(1.0);
            }
        }
    }
}

/// hello i'm storing keybindings complexity here
fn input_system(
    mut inventory_next_state: ResMut<NextState<PlayerInventoryState>>,
    inventory_state: ResMut<State<PlayerInventoryState>>,
    mut help_next_state: ResMut<NextState<HelpGuiState>>,
    help_state: ResMut<State<HelpGuiState>>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
    mut active_game_engine: ResMut<ActiveGameEngine>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action_events: EventWriter<NetworkAction>,
    text_inputs: Query<&TextInput>,
    mut commands: Commands,
) {
    let engine = &mut active_game_engine.0;
    if text_inputs.is_empty() && keyboard.just_pressed(KeyCode::Enter) {
        spawn_text_input(
            &mut commands,
            Vec2::new(50., 200.),
            Vec2::new(300., 30.),
            TextInput::new().with_max_length(50),
        );
        return;
    } else if !text_inputs.is_empty() {
        return;
    }
    // request engine reload if p key is pressed
    if keyboard.just_pressed(KeyCode::KeyP) {
        action_events.write(NetworkAction(Action::RequestEngineReload(
            engine.id,
            engine.step_index,
        )));
        return;
    }

    if keyboard.just_pressed(KeyCode::Slash) && keyboard.pressed(KeyCode::ShiftLeft) {
        match help_state.get() {
            HelpGuiState::Visible => help_next_state.set(HelpGuiState::Hidden),
            HelpGuiState::Hidden => help_next_state.set(HelpGuiState::Visible),
        }
    }

    if keyboard.just_pressed(KeyCode::KeyI) {
        match inventory_state.get() {
            PlayerInventoryState::Visible => inventory_next_state.set(PlayerInventoryState::Hidden),
            PlayerInventoryState::Hidden => inventory_next_state.set(PlayerInventoryState::Visible),
        }
    }

    // allow general input if spawned
    if let Some(entity_id) = active_player_entity_id.0 {
        // input currently being received
        let input = EntityInput {
            jump: !keyboard.pressed(KeyCode::ArrowDown) && keyboard.pressed(KeyCode::Space),
            jump_down: keyboard.pressed(KeyCode::ArrowDown)
                && keyboard.just_pressed(KeyCode::Space),
            move_left: keyboard.pressed(KeyCode::ArrowLeft),
            move_right: keyboard.pressed(KeyCode::ArrowRight),
            crouch: keyboard.pressed(KeyCode::ArrowDown),
            attack: keyboard.just_pressed(KeyCode::KeyA),
            enter_portal: keyboard.pressed(KeyCode::ArrowUp),
            admin_enable_debug_markers: keyboard.just_pressed(KeyCode::Digit9),
            show_emoji: keyboard.just_pressed(KeyCode::KeyQ),
            respawn: keyboard.just_pressed(KeyCode::KeyR),
            pick_up: keyboard.just_pressed(KeyCode::KeyZ),
        };
        let (_, latest_input) =
            if let Some(player_entity) = engine.entity_by_id::<PlayerEntity>(&entity_id, None) {
                &player_entity.input_system.latest_input
            } else {
                println!("WARNING: player entity not found for input");
                return;
            };

        if latest_input == &input {
            return;
        }
        let input_event = EngineEvent::Input {
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
