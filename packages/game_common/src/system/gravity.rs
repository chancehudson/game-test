use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct GravitySystem {}

impl EEntitySystem for GravitySystem {}
