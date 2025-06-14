use bevy::app::App;
use bevy::dev_tools::fps_overlay::FpsOverlayConfig;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::FontSmoothing;

use game_test::engine::entity::EntityInput;
use game_test::engine::GameEngine;
use game_test::MapData;

#[derive(Resource)]
pub struct ActiveGameEngine(pub GameEngine);

#[derive(Component)]
pub struct ActivePlayer;

#[derive(Component)]
pub struct GameEntity(pub u128);

impl Default for ActiveGameEngine {
    fn default() -> Self {
        let map_data_str = std::fs::read_to_string("./assets/maps/eastwatch.map.json5").unwrap();
        let map_data = json5::from_str::<MapData>(&map_data_str).unwrap();
        ActiveGameEngine(GameEngine::new(map_data))
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: 12.0,
                    font: default(),
                    font_smoothing: FontSmoothing::default(),
                    ..default()
                },
                // We can also change color of the overlay
                text_color: Color::WHITE,
                enabled: true,
                ..default()
            },
        },
    ))
    .init_resource::<ActiveGameEngine>()
    .add_systems(Update, (player_input, step_game, render_game, move_camera))
    .add_systems(Startup, (setup_camera, setup_engine, setup_map));

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_engine(mut commands: Commands, mut engine: ResMut<ActiveGameEngine>) {
    // bindings between entity and user (or mob/etc) is managed outside of the engine.
    // the engine has no concept of the game entities
    //
    // the physics is logically separate, even though the skills/abilities
    // dictate the physics
    let player_entity = engine.0.spawn_player_entity();
    let position = player_entity.position();
    let size = player_entity.size();
    commands.spawn((
        GameEntity(player_entity.id()),
        ActivePlayer,
        Transform::from_xyz(position.x, position.y, 10.0),
        Sprite {
            custom_size: Some(size),
            color: Color::srgb(1.0, 0.0, 0.0),
            anchor: Anchor::BottomLeft,
            ..default()
        },
    ));
}

fn setup_map(mut commands: Commands, engine: Res<ActiveGameEngine>) {
    let map = &engine.0.map;
    for platform in &map.platforms {
        commands.spawn((
            Transform::from_xyz(platform.position.x, platform.position.y, 0.0),
            Sprite {
                custom_size: Some(platform.size),
                color: Color::srgb(0.0, 0.0, 1.0),
                anchor: Anchor::BottomLeft,
                ..default()
            },
        ));
    }
}

fn player_input(
    mut engine: ResMut<ActiveGameEngine>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&GameEntity, With<ActivePlayer>>,
) {
    if let Ok(player) = player_query.get_single() {
        let input = EntityInput {
            jump: keyboard.pressed(KeyCode::Space),
            move_left: keyboard.pressed(KeyCode::ArrowLeft),
            move_right: keyboard.pressed(KeyCode::ArrowRight),
            crouch: keyboard.pressed(KeyCode::ArrowDown),
            attack: keyboard.pressed(KeyCode::KeyA),
        };
        engine.0.register_input(None, player.0, input);
    }
}

fn step_game(mut engine: ResMut<ActiveGameEngine>) {
    engine.0.step();
}

fn move_camera(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    player_query: Query<&Transform, (With<ActivePlayer>, Without<Camera2d>)>,
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        if let Ok(player_transform) = player_query.get_single() {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }
    }
}

fn render_game(mut query: Query<(&GameEntity, &mut Transform)>, engine: Res<ActiveGameEngine>) {
    for (entity, mut transform) in &mut query {
        let id = entity.0;
        if let Some(engine_entity) = engine.0.entities.get(&id) {
            let pos = engine_entity.position();
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        } else {
            println!("did not find entity in engine!");
        }
    }
}
