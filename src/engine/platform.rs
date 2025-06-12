use crate::engine::entity::SEEntity;
use crate::entity_struct;

use super::entity::EEntity;

entity_struct!(
    pub struct PlatformEntity {}
);

impl SEEntity for PlatformEntity {}
