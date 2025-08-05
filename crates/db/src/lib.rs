/// I want to encapsulate the complexity around redb manipulation here.
/// Outside this package it should be impossible to determine the underlying
/// storage implementation.
///
mod ability_exp_record;
mod player_inventory;
mod player_record;
mod player_stats;

pub use ability_exp_record::Ability;
pub use ability_exp_record::AbilityExpRecord;
pub use player_inventory::PlayerInventory;
pub use player_record::PlayerRecord;
pub use player_stats::PlayerStats;

pub const DEFAULT_MAP: &str = "digital_skyscrapers_1";

pub fn init(db: redb::Database) -> anyhow::Result<std::sync::Arc<redb::Database>> {
    AbilityExpRecord::init(&db)?;
    PlayerRecord::init(&db)?;
    PlayerInventory::init(&db)?;
    Ok(std::sync::Arc::new(db))
}
