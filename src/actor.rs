/// game physics logic
/// probably needs it's own crate
use std::mem::discriminant;

use bevy_math::Rect;
use bevy_math::Vec2;

use crate::engine::entity::EEntity;
use crate::engine::entity::EngineEntity;
use crate::engine::platform::PlatformEntity;
use crate::engine::rect::RectEntity;
use crate::engine::GameEngine;

use super::MapData;

pub fn contains_platform(engine: &mut GameEngine, rect: Rect) -> bool {
    let platforms = engine
        .grouped_entities()
        .get(&discriminant(&EngineEntity::Platform(
            PlatformEntity::default(),
        )));
    if let Some(platforms) = platforms {
        for platform in platforms {
            let intersection = rect.intersect(platform.rect());
            if intersection.width() >= 1.0 && intersection.height() >= 1.0 {
                return true;
            }
        }
    }
    false
}

/// Are we standing with a platform beneath us, without a platform immediately above it?
pub fn on_platform(body: Rect, engine: &mut GameEngine) -> bool {
    // check if the intersection is underneath the player
    let launch_rect = Rect::new(body.min.x, body.min.y - 2., body.max.x, body.min.y - 1.);
    let not_launch_rect = Rect::new(body.min.x, body.min.y + 1., body.max.x, body.min.y + 3.);
    return contains_platform(engine, launch_rect) && !contains_platform(engine, not_launch_rect);
}

pub fn can_move_left_right(body: Rect, engine: &mut GameEngine) -> (bool, bool) {
    (
        body.min.x.round() > 2.0,
        (body.max.x + body.width()).round() < engine.map.size.x - 2.0,
    )
}

// returns if we can move left or right
pub fn can_move_left_right_without_falling(body: Rect, engine: &mut GameEngine) -> (bool, bool) {
    let dist = 2.0;
    let left_check = Rect::new(
        body.min.x - (2. * dist),
        body.min.y - (2. * dist),
        body.min.x - dist,
        body.min.y - dist,
    );

    let right_check = Rect::new(
        body.min.x + body.width() + dist,
        body.min.y - (2. * dist),
        body.min.x + body.width() + (2. * dist),
        body.min.y - dist,
    );
    if engine.enable_debug_markers {
        let mut debug_rect = RectEntity::default();
        debug_rect.pure = true;
        debug_rect.id = engine.generate_id();
        debug_rect.disappears_at_step_index = Some(engine.step_index + 120);
        debug_rect.color = bevy_math::Vec3::new(1.0, 0.0, 0.0);
        debug_rect.position = left_check.min;
        debug_rect.size = left_check.size();
        engine.spawn_entity(EngineEntity::Rect(debug_rect), None, true);
        let mut debug_rect = RectEntity::default();
        debug_rect.pure = true;
        debug_rect.id = engine.generate_id();
        debug_rect.disappears_at_step_index = Some(engine.step_index + 120);
        debug_rect.color = bevy_math::Vec3::new(1.0, 0.0, 0.0);
        debug_rect.position = right_check.min;
        debug_rect.size = right_check.size();
        engine.spawn_entity(EngineEntity::Rect(debug_rect), None, true);
    }

    (
        contains_platform(engine, left_check),
        contains_platform(engine, right_check),
    )
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

pub fn move_y(body: Rect, dy: f32, platforms: &[EngineEntity], map_size: Vec2) -> f32 {
    if dy == 0. {
        return body.min.y;
    }
    let sign = dy.signum();
    let dy_abs = dy.abs();
    let mut moved = 0.;
    let min_y = 0.0;
    let max_y = map_size.y - body.height();
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

        for solid in platforms {
            let solid_rect = Rect::new(
                solid.position().x,
                solid.position().y,
                solid.position().x + solid.size().x,
                solid.position().y + solid.size().y,
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
