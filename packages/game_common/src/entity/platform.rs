use crate::prelude::*;

entity_struct!(
    pub struct PlatformEntity {}
);

#[typetag::serde]
impl SEEntity for PlatformEntity {}
