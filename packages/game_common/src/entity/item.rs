use bevy_math::IVec2;

use keind::prelude::*;

use crate::prelude::*;

entity_struct!(
    KeindGameLogic,
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
        current_step: &u64,
    ) -> Self {
        Self {
            state: BaseEntityState {
                id,
                position,
                size: IVec2 { x: 25, y: 25 },
                velocity: IVec2 { x: 0, y: 350 },
                player_creator_id: Some(player_creator_id),
            },
            systems: vec![
                RefPointer::new(
                    DisappearSystem {
                        at_step: current_step + 7200,
                    }
                    .into(),
                ),
                RefPointer::new(GravitySystem::default().into()),
                RefPointer::new(AtomicMoveSystem::default().into()),
            ],
            item_type,
            count,
            // becomes_public_at_step: current_step + 3600,
            ..Default::default()
        }
    }
}

impl SEEntity<KeindGameLogic> for ItemEntity {
    fn prestep(&self, _engine: &GameEngine<KeindGameLogic>) -> bool {
        assert!(self.has_system::<DisappearSystem>());
        true
    }

    fn step(&self, engine: &GameEngine<KeindGameLogic>, next_self: &mut Self) {
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
        } else {
            next_self.position_offset_y = 0;
        }
    }
}
