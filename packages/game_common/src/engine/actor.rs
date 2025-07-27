/// game physics logic
/// probably needs it's own crate
use bevy_math::IRect;
use bevy_math::IVec2;

use crate::prelude::*;

pub fn contains_platform(engine: &GameEngine, rect: IRect) -> bool {
    for platform in engine.entities_by_type::<PlatformEntity>() {
        let intersection = rect.intersect(platform.rect());
        if intersection.width() >= 1 && intersection.height() >= 1 {
            return true;
        }
    }
    false
}

/// Are we standing with a platform beneath us, without a platform immediately above it?
pub fn on_platform(body: IRect, engine: &GameEngine) -> bool {
    // check if the intersection is underneath the player
    let launch_rect = IRect::new(body.min.x, body.min.y - 2, body.max.x, body.min.y - 1);
    let not_launch_rect = IRect::new(body.min.x, body.min.y + 1, body.max.x, body.min.y + 3);
    return contains_platform(engine, launch_rect) && !contains_platform(engine, not_launch_rect);
}

pub fn can_move_left_right(body: IRect, engine: &GameEngine) -> (bool, bool) {
    (
        body.min.x > 2,
        body.max.x + body.width() < engine.size().x - 2,
    )
}

// returns if we can move left or right
pub fn can_move_left_right_without_falling(body: IRect, engine: &GameEngine) -> (bool, bool) {
    let dist = 2;
    let left_check = IRect::new(
        body.min.x - (2 * dist),
        body.min.y - (2 * dist),
        body.min.x - dist,
        body.min.y - dist,
    );

    let right_check = IRect::new(
        body.min.x + body.width() + dist,
        body.min.y - (2 * dist),
        body.min.x + body.width() + (2 * dist),
        body.min.y - dist,
    );

    (
        contains_platform(engine, left_check),
        contains_platform(engine, right_check),
    )
}

pub fn move_x(body: IRect, dx: i32, engine: &GameEngine) -> i32 {
    if dx == 0 {
        return body.min.x;
    }
    let mut body = body.clone();
    body.min.x += dx;
    body.max.x += dx;
    if body.max.x > engine.size().x {
        body.min.x = engine.size().x - body.width();
        body.max.x = engine.size().x;
    } else if body.min.x < 0 {
        body.max.x = body.width();
        body.min.x = 0;
    }
    body.min.x
}

pub fn move_y<T: EEntity>(body: IRect, dy: i32, platforms: &[Rc<T>], map_size: IVec2) -> i32 {
    if dy == 0 {
        return body.min.y;
    }
    let sign = dy.signum();
    let dy_abs = dy.abs();
    let mut moved = 0;
    let min_y = 0;
    let max_y = map_size.y - body.height();
    let mut position = body.min.clone();

    // if the character is jumping we don't care about collisions
    if dy.signum() == 1 {
        position.y = (position.y + dy).clamp(min_y, max_y);
        return position.y;
    }
    while moved < dy_abs + 1 {
        let mut new_player_rect = body.clone();
        new_player_rect.min.y += sign * moved;
        new_player_rect.max.y += sign * moved;

        for solid in platforms {
            let solid_rect = solid.rect();
            let overlap = solid_rect.intersect(new_player_rect);
            if overlap.is_empty() {
                continue;
            }
            // only collide if we're at the top of the platform
            if overlap.height() == 1 && (overlap.min.y - solid_rect.max.y).abs() == 1 {
                // we've collided, stop
                return (new_player_rect.min.y - sign * (overlap.height() + 1)).clamp(min_y, max_y);
            }
        }
        moved += 1;
    }
    // position.y += dy;
    (position.y + dy).clamp(min_y, max_y)
}
