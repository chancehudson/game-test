use crate::engine::entity::SEEntity;
use crate::entity_struct;

use super::EEntity;

entity_struct!(
    pub struct PlatformEntity {}
);

impl SEEntity for PlatformEntity {}
