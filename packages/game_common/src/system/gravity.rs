use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct GravitySystem {}

#[typetag::serde]
impl EEntitySystem for GravitySystem {
    fn step(
        &self,
        _engine: &GameEngine,
        _entity: &mut dyn SEEntity,
    ) -> Option<Box<dyn EEntitySystem>> {
        None
    }

    fn clone_box(&self) -> Box<dyn EEntitySystem> {
        Box::new(self.clone())
    }
}
