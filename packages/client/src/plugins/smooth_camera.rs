use bevy::prelude::*;

use game_common::entity::EEntity;

use crate::map;
use crate::map_data_loader::MapDataAsset;
use crate::plugins::engine::ActiveGameEngine;
use crate::plugins::engine::ActivePlayerEntityId;

const CAMERA_ACCELERATION: f32 = 1500.0;
const CAMERA_MAX_SPEED: f32 = 300.0;
pub const CAMERA_Y_PADDING: f32 = 200.0;

pub struct SmoothCameraPlugin;

#[derive(Component)]
pub struct DebugMarker;

#[derive(Component)]
pub struct CameraMovement {
    pub is_moving_x: bool,
    pub is_moving_y: bool,
    pub velocity: Vec2,
}

impl Plugin for SmoothCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera).add_systems(
            Update,
            (player_camera, move_debug_marker).run_if(in_state(crate::GameState::OnMap)),
        );
    }
}

fn setup_camera(mut commands: Commands /*windows: Query<&Window>*/) {
    commands.spawn((
        Camera2d,
        CameraMovement {
            is_moving_x: false,
            is_moving_y: false,
            velocity: Vec2::ZERO,
        },
    ));
    // spawn a box onscreen showing the movement range
    // let window = windows.single();
    // let screen_width = window.resolution.width();
    // let screen_height = window.resolution.height();
    // let movement_range = Vec2::new(f32::min(100., screen_width), f32::min(100., screen_height));
    // commands.spawn((
    //     DebugMarker,
    //     Sprite {
    //         custom_size: Some(movement_range * Vec2::splat(2.0)),
    //         color: Color::srgba(1.0, 0.0, 0.0, 0.5),
    //         ..default()
    //     },
    //     Transform::from_translation(Vec3::new(
    //         screen_width / 2.0 - movement_range.x,
    //         screen_height / 2.0 - movement_range.y,
    //         10.0,
    //     )),
    // ));
}

fn move_debug_marker(
    mut query: Query<(&DebugMarker, &mut Transform), Without<Camera2d>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    if query.is_empty() {
        return;
    }
    let (_, mut transform) = query.single_mut().unwrap();
    let camera_transform = camera_query.single_mut().unwrap();
    transform.translation.x = camera_transform.translation.x;
    transform.translation.y = camera_transform.translation.y;
}

pub fn snap_to_position(
    camera_query: &mut Query<(&mut Transform, &mut CameraMovement), With<Camera2d>>,
    map_loader: &Res<map::MapLoader>,
    map_assets: &Res<Assets<MapDataAsset>>,
    windows: Query<&Window>,
    reset_velocity: bool,
) {
    if let Ok((mut camera_transform, mut camera_movement)) = camera_query.single_mut() {
        if reset_velocity {
            camera_movement.is_moving_x = false;
            camera_movement.is_moving_y = false;
            camera_movement.velocity = Vec2::ZERO;
        }

        let active_map = map_loader.map_data(map_assets);
        if active_map.is_none() {
            return;
        }
        let active_map = active_map.unwrap();
        if active_map.size == IVec2::ZERO {
            return;
        }
        if reset_velocity {
            println!("snapping to position");
        }
        let window = windows.single().unwrap();
        let screen_width = window.resolution.width();
        let screen_height = window.resolution.height();

        let map_size = Vec2::new(active_map.size.x as f32, active_map.size.y as f32);
        // if the game window is larger than the size of the map the client
        // crashes
        //
        // then bound it to the viewbox
        if screen_width >= map_size.x {
            camera_transform.translation.x = map_size.x / 2.0;
        } else {
            camera_transform.translation.x = camera_transform
                .translation
                .x
                .clamp(screen_width / 2., map_size.x - screen_width / 2.);
        }
        if screen_height >= map_size.y {
            camera_transform.translation.y = map_size.y / 2.0;
        } else {
            // we leave space at the bottom of the screen for the GUI
            camera_transform.translation.y = camera_transform.translation.y.clamp(
                screen_height / 2. - CAMERA_Y_PADDING,
                map_size.y - screen_height / 2.,
            );
        }
    }
}

/// We'll allow the player to move in a small square near the center of
/// the screen. If the player moves out of this square the camera will
/// smoothly accelerate to follow the player
fn player_camera(
    active_player_entity_id: Res<ActivePlayerEntityId>,
    active_game_engine: Res<ActiveGameEngine>,
    mut camera: Query<(&mut Transform, &mut CameraMovement), With<Camera2d>>,
    windows: Query<&Window>,
    map_loader: Res<map::MapLoader>,
    map_assets: Res<Assets<MapDataAsset>>,
    time: Res<Time>,
) {
    if active_player_entity_id.0.is_none() {
        if let Ok((mut camera_transform, _)) = camera.single_mut() {
            camera_transform.translation = Vec3::new(0.0, 0.0, 0.0);
        }
        snap_to_position(&mut camera, &map_loader, &map_assets, windows, false);
        return;
    }
    let active_player_entity_id = active_player_entity_id.0.as_ref().unwrap();
    let engine = &active_game_engine.0;
    let entity = engine
        .entities_at_step(engine.step_index)
        .get(active_player_entity_id);
    if entity.is_none() {
        return;
    }
    let entity = entity.unwrap();
    let delta = time.delta_secs();
    if let Ok((mut camera_transform, mut camera_movement)) = camera.single_mut() {
        let window = windows.single().unwrap();
        let screen_width = window.resolution.width();
        let screen_height = window.resolution.height();
        let movement_range = Vec2::new(f32::min(150., screen_width), f32::min(100., screen_height));

        // centered position
        let player_pos = entity.position_f32() + entity.size_f32() / Vec2::splat(2.0);
        let dist = player_pos - camera_transform.translation.xy();
        // adjust the x velocity
        if dist.x.abs() > movement_range.x && !camera_movement.is_moving_x {
            camera_movement.is_moving_x = true;
            camera_movement.velocity.x = dist.x.signum();
        } else if (dist.x.abs() < 5.0
            || (dist.x.abs() < movement_range.x && camera_movement.velocity.x.abs() < 10.0))
            && camera_movement.is_moving_x
        {
            camera_movement.is_moving_x = false;
            camera_movement.velocity.x = 0.0;
        } else if camera_movement.is_moving_x {
            if dist.x.abs() < movement_range.x {
                let accel_diff_abs = (camera_movement.velocity.x.abs().powf(1.15))
                    .max(10.0)
                    .min(CAMERA_ACCELERATION)
                    * delta;
                // moving toward 0
                camera_movement.velocity.x = (camera_movement.velocity.x.abs() - accel_diff_abs)
                    .max(0.0)
                    * dist.x.signum().signum();
            } else {
                // moving toward max velocity
                let accel_diff_abs =
                    (2.0 * dist.x.abs()).max(10.0).min(CAMERA_ACCELERATION) * delta;
                camera_movement.velocity.x = (camera_movement.velocity.x.abs() + accel_diff_abs)
                    .min(CAMERA_MAX_SPEED)
                    * dist.x.signum();
            }
        }
        // adjust the y velocity
        if dist.y.abs() > movement_range.y && !camera_movement.is_moving_y {
            camera_movement.is_moving_y = true;
            camera_movement.velocity.y = dist.y.signum();
        } else if (dist.y.abs() < 5.0
            || (dist.y.abs() < movement_range.y && camera_movement.velocity.y.abs() < 10.0))
            && camera_movement.is_moving_y
        {
            camera_movement.is_moving_y = false;
            camera_movement.velocity.y = 0.0;
        } else if camera_movement.is_moving_y {
            if dist.y.abs() < movement_range.y {
                let accel_diff_abs = (camera_movement.velocity.y.abs().powf(1.4))
                    .max(10.0)
                    .min(CAMERA_ACCELERATION)
                    * delta;
                // moving toward 0
                camera_movement.velocity.y = (camera_movement.velocity.y.abs() - accel_diff_abs)
                    .max(0.0)
                    * dist.y.signum().signum();
            } else {
                // moving toward may velocity
                let accel_diff_abs =
                    (2.0 * dist.y.abs()).max(10.0).min(CAMERA_ACCELERATION) * delta;
                camera_movement.velocity.y = (camera_movement.velocity.y.abs() + accel_diff_abs)
                    .min(CAMERA_MAX_SPEED)
                    * dist.y.signum();
            }
        }
        camera_movement.velocity = camera_movement.velocity.clamp(
            -Vec2::splat(CAMERA_MAX_SPEED),
            Vec2::splat(CAMERA_MAX_SPEED),
        );
        // handle actually moving the camera based on the velocity
        camera_transform.translation.x += camera_movement.velocity.x * delta;
        camera_transform.translation.y += camera_movement.velocity.y * delta;
    }
    snap_to_position(&mut camera, &map_loader, &map_assets, windows, false);
}
