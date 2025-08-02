use std::sync::Arc;

use db::AbilityExpRecord;
use serde::Deserialize;
use serde::Serialize;

use keind::prelude::*;

use crate::prelude::*;

/// Give some experience to a player. Uses
/// `db::PlayerStats` and `db::AbilityExpRecord`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerExpSystem {
    pub record: AbilityExpRecord,
}

impl EEntitySystem<KeindGameLogic> for PlayerExpSystem {
    fn step(
        &self,
        _engine: &GameEngine<KeindGameLogic>,
        _entity: &EngineEntity,
        next_entity: &mut EngineEntity,
    ) -> Option<Self> {
        // update the stats pointer on a player entity
        let player_entity = next_entity
            .extract_mut::<PlayerEntity>()
            .expect("PlayerExpSystem must be attached to a player entity");
        let mut stats_ptr = player_entity.stats_ptr.clone();
        let stats: &mut db::PlayerStats = RefPointer::make_mut(&mut stats_ptr);
        stats.increment(&self.record);
        player_entity.stats_ptr = stats_ptr;

        // Despawn
        None
    }
}
