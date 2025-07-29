use serde::Deserialize;
use serde::Serialize;

use keind::prelude::*;

use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct GravitySystem {}

impl EEntitySystem<KeindGameLogic> for GravitySystem {}
