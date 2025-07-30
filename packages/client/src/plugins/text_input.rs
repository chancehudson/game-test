use bevy::input::keyboard::Key;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use game_common::prelude::*;
use keind::prelude::*;

use crate::GameState;
use crate::map::MapEntity;
use crate::network::NetworkAction;
use crate::plugins::engine::ActiveGameEngine;
use crate::plugins::engine::ActivePlayerEntityId;

#[derive(Component)]
pub struct TextInput {
    pub text: String,
    pub max_length: Option<usize>,
    pub cursor_timer: Timer,
    pub show_cursor: bool,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            text: String::new(),
            max_length: None,
            cursor_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            show_cursor: true,
        }
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }
}

#[derive(Component)]
struct TextInputBackground;

#[derive(Component)]
struct TextInputDisplay;

pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_text_input_keyboard,
                update_text_input_display_fixed,
                update_cursor_blink,
            )
                .run_if(in_state(GameState::OnMap)),
        );
    }
}

pub fn spawn_text_input(
    commands: &mut Commands,
    position: Vec2,
    size: Vec2,
    text_input: TextInput,
) -> Entity {
    let input_entity = commands
        .spawn((
            text_input,
            MapEntity,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                width: Val::Px(size.x),
                height: Val::Px(size.y),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|parent| {
            // Text display
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextInputDisplay,
            ));
        })
        .id();

    input_entity
}

fn handle_text_input_keyboard(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut text_inputs: Query<(Entity, &mut TextInput)>,
    mut commands: Commands,
    mut active_game_engine: ResMut<ActiveGameEngine>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
    mut action_events: EventWriter<NetworkAction>,
) {
    let engine = &mut active_game_engine.0;
    let entity_id = active_player_entity_id.0.unwrap_or_default();
    if let Ok((entity, mut input)) = text_inputs.single_mut() {
        for event in keyboard_events.read() {
            if !event.state.is_pressed() {
                continue;
            }

            match &event.logical_key {
                Key::Character(char) => {
                    // Add character to text
                    if let Some(max_len) = input.max_length {
                        if input.text.len() >= max_len {
                            continue;
                        }
                    }
                    input.text.push_str(char);
                }
                Key::Space => {
                    input.text.push_str(" ");
                }
                Key::Backspace => {
                    // Remove last character
                    input.text.pop();
                }
                Key::Enter => {
                    // send
                    // let event = EngineEvent::Message {
                    //     text: input.text.clone(),
                    //     entity_id,
                    //     universal: true,
                    // };
                    // engine.register_event(None, event.clone());
                    // // send the new input to the server
                    // action_events.write(NetworkAction(Action::RemoteEngineEvent(
                    //     engine.id,
                    //     event,
                    //     engine.step_index,
                    // )));
                    commands.entity(entity).despawn();
                }
                Key::Escape => {
                    commands.entity(entity).despawn();
                }
                _ => {}
            }
        }
    }
}

fn update_text_input_display_fixed(
    text_inputs: Query<(Entity, &TextInput), Changed<TextInput>>,
    mut text_displays: Query<(&mut Text, &mut TextColor), With<TextInputDisplay>>,
    children: Query<&Children>,
) {
    for (entity, input) in text_inputs.iter() {
        if let Ok(children) = children.get(entity) {
            for child in children.iter() {
                if let Ok((mut text, mut text_color)) = text_displays.get_mut(child) {
                    let display_text = if input.text.is_empty() {
                        "".to_string()
                    } else {
                        input.text.clone()
                    };

                    let cursor = if input.show_cursor { "|" } else { "" };
                    **text = format!("{}{}", display_text, cursor);

                    // Change color based on whether it's placeholder text
                    text_color.0 = Color::WHITE;
                }
            }
        }
    }
}

fn update_cursor_blink(mut text_inputs: Query<&mut TextInput>, time: Res<Time>) {
    for mut input in text_inputs.iter_mut() {
        input.cursor_timer.tick(time.delta());
        if input.cursor_timer.just_finished() {
            input.show_cursor = !input.show_cursor;
        }
    }
}
