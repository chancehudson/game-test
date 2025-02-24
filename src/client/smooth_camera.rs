use bevy::prelude::*;

use crate::map;
use crate::player::Player;
use crate::ActivePlayer;

const CAMERA_ACCELERATION: f32 = 500.0;
const CAMERA_MAX_SPEED: f32 = 300.0;

pub struct SmoothCameraPlugin;

#[derive(Component)]
pub struct CameraMovement {
    pub is_moving_x: bool,
    pub is_moving_y: bool,
    pub velocity: Vec2,
}

impl Plugin for SmoothCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, player_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        CameraMovement {
            is_moving_x: false,
            is_moving_y: false,
            velocity: Vec2::ZERO,
        },
    ));
}

/// We'll allow the player to move in a small square near the center of
/// the screen. If the player moves out of this square the camera will
/// smoothly accelerate to follow the player
fn player_camera(
    player: Query<(&Player, &Transform, &ActivePlayer), Without<Camera2d>>,
    mut camera: Query<(&mut Transform, &mut CameraMovement), With<Camera2d>>,
    windows: Query<&Window>,
    active_map: Res<map::ActiveMap>,
    time: Res<Time>,
) {
    if player.is_empty() {
        return;
    }
    let delta = time.delta_secs();
    let (player, player_transform, _) = player.single();
    let (mut camera_transform, mut camera_movement) = camera.single_mut();
    let window = windows.single();
    let screen_width = window.resolution.width();
    let screen_height = window.resolution.height();
    let movement_range = Vec2::new(f32::min(300., screen_width), f32::min(150., screen_height));

    let player_pos = player_transform.translation.xy() + player.body.size / Vec2::splat(2.0);
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
        let toward = if dist.x.abs() < movement_range.x {
            Vec2::ZERO
        } else {
            Vec2::splat(dist.x.signum() * CAMERA_MAX_SPEED)
        };
        let x_diff = (dist.x.abs() - movement_range.x).abs();
        camera_movement.velocity.x = camera_movement
            .velocity
            .move_towards(
                toward,
                (15.0 * x_diff).max(10.0).min(CAMERA_ACCELERATION) * delta,
            )
            .x;
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
        let toward = if dist.y.abs() < movement_range.y {
            Vec2::ZERO
        } else {
            Vec2::splat(dist.y.signum() * CAMERA_MAX_SPEED)
        };
        let y_diff = (dist.y.abs() - movement_range.y).abs();
        // println!("{y_diff}, {}", 5.0 * y_diff);
        camera_movement.velocity.y = camera_movement
            .velocity
            .move_towards(
                toward,
                (5.0 * y_diff).max(10.0).min(CAMERA_ACCELERATION) * delta,
            )
            .y;
    }
    camera_movement.velocity = camera_movement.velocity.clamp(
        -Vec2::splat(CAMERA_MAX_SPEED),
        Vec2::splat(CAMERA_MAX_SPEED),
    );
    camera_transform.translation.x += camera_movement.velocity.x * delta;
    camera_transform.translation.y += camera_movement.velocity.y * delta;

    // don't allow the player to move offscreen
    let max_x_dist = screen_width / 2.0 - 150.0;
    if dist.x.abs() > max_x_dist && player.body.velocity.x.abs() > camera_movement.velocity.x.abs()
    {
        camera_transform.translation.x = player_pos.x - max_x_dist * dist.x.signum();
    }
    let max_y_dist = screen_height / 2.0 - 150.0;
    if dist.y.abs() > max_y_dist && player.body.velocity.y.abs() > camera_movement.velocity.y.abs()
    {
        camera_transform.translation.y = player_pos.y - max_y_dist * dist.y.signum();
    }
    // clamp the camera as needed
    if active_map.size == Vec2::ZERO {
        return;
    }
    camera_transform.translation.x = camera_transform
        .translation
        .x
        .clamp(screen_width / 2., active_map.size.x - screen_width / 2.);
    camera_transform.translation.y = camera_transform
        .translation
        .y
        .clamp(screen_height / 2., active_map.size.y - screen_height / 2.);
}
