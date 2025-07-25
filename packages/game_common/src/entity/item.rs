use bevy_math::IVec2;
use bevy_math::Vec2;

use crate::prelude::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default, PartialEq)]
pub struct ItemEntity {
    #[serde(default)]
    pub id: u128,
    pub item_type: u64,
    pub count: u32,
    #[serde(default)]
    pub position: bevy_math::IVec2,
    #[serde(default)]
    pub size: bevy_math::IVec2,
    #[serde(default)]
    pub velocity: bevy_math::IVec2,
    #[serde(default)]
    pub player_creator_id: Option<u128>,
    pub disappears_at_step: u64,
    // pub becomes_public_at_step: u64, // when non-player creator players may pick it up
    pub position_offset_y: i32,
    pub is_picked_up: bool,
}

impl ItemEntity {
    pub fn new_item(
        id: u128,
        position: IVec2,
        item_type: u64,
        count: u32,
        player_creator_id: u128,
        current_step: u64,
    ) -> Self {
        Self {
            id,
            item_type,
            count,
            position,
            size: IVec2 { x: 25, y: 25 },
            velocity: IVec2 { x: 0, y: 350 },
            player_creator_id: Some(player_creator_id),
            disappears_at_step: current_step + 7200,
            // becomes_public_at_step: current_step + 3600,
            ..Default::default()
        }
    }
}

impl SEEntity for ItemEntity {
    fn step<T: super::GameEngine>(&self, engine: &T) -> Self {
        let mut next_self = self.clone();
        let step_index = engine.step_index();
        if &self.disappears_at_step <= step_index {
            engine.remove_entity(&self.id, None, false);
            return next_self;
        }
        let self_rect = self.rect();
        if self.velocity.y <= 0 && actor::on_platform(self_rect, engine) {
            const ANIMATION_FRAME_LEN: i32 = 12;
            const ANIMATION_FRAME_COUNT: i32 = 4;
            const ANIMATION_STEP_LEN: i32 = ANIMATION_FRAME_COUNT * ANIMATION_FRAME_LEN;
            next_self.position_offset_y =
                ((step_index % ANIMATION_STEP_LEN as u64) as i32) / ANIMATION_FRAME_LEN;
            if (step_index / ANIMATION_STEP_LEN as u64) % 2 == 0 {
                next_self.position_offset_y = ANIMATION_FRAME_COUNT - next_self.position_offset_y;
            }
            next_self.velocity = IVec2::ZERO;
        } else {
            next_self.position_offset_y = 0;
            next_self.velocity.y += -20;
            let lower_speed_limit = IVec2::new(-250, -350);
            let upper_speed_limit = IVec2::new(250, 700);
            next_self.velocity = next_self
                .velocity
                .clamp(lower_speed_limit, upper_speed_limit);
            let map_size = engine.size().clone();
            let platforms = engine.entities_by_type::<PlatformEntity>();
            let y_pos = actor::move_y(
                self_rect,
                next_self.velocity.y / STEPS_PER_SECOND_I32,
                &platforms.collect::<Vec<_>>(),
                map_size,
            );
            next_self.position.y = y_pos;
        }
        next_self
    }
}

impl EEntity for ItemEntity {
    fn id(&self) -> u128 {
        self.id
    }

    fn position_f32(&self) -> bevy_math::Vec2 {
        Vec2::new(
            self.position.x as f32,
            (self.position.y + self.position_offset_y) as f32,
        )
    }

    /// We want to animate the items with a bobbing movement. We'll use a periodic function around the y position
    fn position(&self) -> bevy_math::IVec2 {
        self.position
    }

    fn size(&self) -> bevy_math::IVec2 {
        self.size
    }

    fn velocity(&self) -> bevy_math::IVec2 {
        self.velocity
    }

    fn player_creator_id(&self) -> Option<u128> {
        self.player_creator_id
    }
}
