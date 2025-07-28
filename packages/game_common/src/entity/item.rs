use bevy_math::IVec2;

use crate::prelude::*;

entity_struct!(
    pub struct ItemEntity {
        pub item_type: u64,
        pub count: u32,
        pub disappears_at_step: u64,
        // pub becomes_public_at_step: u64, // when non-player creator players may pick it up
        pub position_offset_y: i32,
        pub is_picked_up: bool,
    }
);

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
            state: BaseEntityState {
                id,
                position,
                size: IVec2 { x: 25, y: 25 },
                velocity: IVec2 { x: 0, y: 350 },
                player_creator_id: Some(player_creator_id),
            },
            systems: vec![Rc::new(EngineEntitySystem::from(DisappearSystem {
                at_step: current_step + 7200,
            }))],
            item_type,
            count,
            // becomes_public_at_step: current_step + 3600,
            ..Default::default()
        }
    }
}

impl SEEntity for ItemEntity {
    fn step(&self, engine: &GameEngine) -> Option<Self> {
        assert!(self.has_system::<DisappearSystem>());
        let mut next_self = self.clone();
        let step_index = engine.step_index();
        let self_rect = self.rect();
        if self.velocity().y <= 0 && actor::on_platform(self_rect, engine) {
            const ANIMATION_FRAME_LEN: i32 = 12;
            const ANIMATION_FRAME_COUNT: i32 = 4;
            const ANIMATION_STEP_LEN: i32 = ANIMATION_FRAME_COUNT * ANIMATION_FRAME_LEN;
            next_self.position_offset_y =
                ((step_index % ANIMATION_STEP_LEN as u64) as i32) / ANIMATION_FRAME_LEN;
            if (step_index / ANIMATION_STEP_LEN as u64) % 2 == 0 {
                next_self.position_offset_y = ANIMATION_FRAME_COUNT - next_self.position_offset_y;
            }
            next_self.state.velocity = IVec2::ZERO;
        } else {
            next_self.position_offset_y = 0;
            next_self.state.velocity.y += -20;
            let lower_speed_limit = IVec2::new(-250, -350);
            let upper_speed_limit = IVec2::new(250, 700);
            next_self.state.velocity = next_self
                .velocity()
                .clamp(lower_speed_limit, upper_speed_limit);
            let map_size = engine.size().clone();
            let y_pos = actor::move_y(
                self_rect,
                next_self.velocity().y / STEPS_PER_SECOND_I32,
                &engine.entities_by_type::<PlatformEntity>(),
                map_size,
            );
            next_self.state.position.y = y_pos;
        }
        Some(next_self)
    }
}
