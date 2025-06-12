use super::entity::EEntity;
use super::entity::SEEntity;

crate::entity_struct!(
    pub struct ImageTmpEntity {
        attached_to: Option<u128>,
        disappears_at_step_index: u64,
    }
);

impl SEEntity for ImageTmpEntity {
    fn step(&self, _engine: &mut super::GameEngine, _step_index: &u64) -> Self
    where
        Self: Sized + Clone,
    {
        self.clone()
    }
}
