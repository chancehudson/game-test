use bevy::math::Rect;
use bevy::math::Vec2;

use super::MapData;

// in pixels per second per second
pub const GRAVITY_ACCEL: f32 = 1200.0;
pub const MAX_VELOCITY: Vec2 = Vec2::new(300.0, 400.0);

/// Are we standing with a platform beneath us, without a platform immediately above it?
pub fn on_platform(body: Rect, map: &MapData) -> bool {
    // check if the intersection is underneath the player
    let launch_rect = Rect::new(body.min.x, body.min.y - 2., body.max.x, body.min.y - 1.);
    let not_launch_rect = Rect::new(body.min.x, body.min.y + 1., body.max.x, body.min.y + 3.);
    return map.contains_platform(launch_rect) && !map.contains_platform(not_launch_rect);
}

pub fn move_x(body: Rect, dx: f32, map: &MapData) -> f32 {
    if dx == 0. {
        return body.min.x;
    }
    let mut body = body.clone();
    body.min.x += dx;
    body.max.x += dx;
    if body.max.x > map.size.x {
        body.min.x = map.size.x - body.width();
        body.max.x = map.size.x;
    } else if body.min.x < 0. {
        body.max.x = body.width();
        body.min.x = 0.;
    }
    body.min.x
}

pub fn move_y(body: bevy::math::Rect, dy: f32, map: &MapData) -> f32 {
    if dy == 0. {
        return body.min.y;
    }
    let sign = dy.signum();
    let dy_abs = dy.abs();
    let mut moved = 0.;
    let min_y = 0.0;
    let max_y = map.size.y - body.height();
    let mut position = body.min.clone();

    // if the character is jumping we don't care about collisions
    if dy.is_sign_positive() {
        position.y = (position.y + dy).clamp(min_y, max_y);
        return position.y;
    }
    while moved < dy_abs + 1. {
        let mut new_player_rect = body.clone();
        new_player_rect.min.y += sign * moved;
        new_player_rect.max.y += sign * moved;

        for solid in &map.platforms {
            let solid_rect = bevy::math::Rect::new(
                solid.position.x,
                solid.position.y,
                solid.position.x + solid.size.x,
                solid.position.y + solid.size.y,
            );
            let overlap = solid_rect.intersect(new_player_rect);
            if overlap.is_empty() {
                continue;
            }
            // only collide if we're at the top of the platform
            if overlap.height() < 1. && (overlap.min.y - solid_rect.max.y).abs() < 1. {
                // we've collided, stop
                return (new_player_rect.min.y - sign).clamp(min_y, max_y);
            }
        }
        moved += 1.;
    }
    // position.y += dy;
    (position.y + dy).clamp(min_y, max_y)
}
