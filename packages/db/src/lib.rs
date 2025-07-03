/// I want to encapsulate the complexity around redb manipulation here.
/// Outside this package it should be impossible to determine the underlying
/// storage implementation.
///
mod ability_exp_record;
mod player_record;
mod player_stats;

pub use ability_exp_record::Ability;
pub use ability_exp_record::AbilityExpRecord;
pub use player_record::PlayerRecord;
pub use player_stats::PlayerStats;

pub const DEFAULT_MAP: &str = "digital_skyscrapers_1";
