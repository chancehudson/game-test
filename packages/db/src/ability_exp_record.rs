/// Experience will be received and applied upon dealing damage to a mob
/// Experience is applied to the stat that was used to deal the damage
///
use std::cell::LazyCell;
use std::collections::BTreeMap;

use anyhow::Result;
use redb::ReadableTable;
use redb::TableDefinition;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumIter;

pub const ABILITY_EXP_TABLE: redb::TableDefinition<(Ability, String), AbilityExpRecord> =
    TableDefinition::new("player_ability_exp");

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Hash,
    Eq,
    EnumIter,
    PartialOrd,
    Ord,
    strum::IntoStaticStr,
)]
#[repr(u8)]
#[derive(Default)]
pub enum Ability {
    Health = 0,
    #[default]
    Strength = 1,
    Dexterity = 2,
    Intelligence = 3,
}


const EXP_LEVEL_PRECALC: LazyCell<BTreeMap<u64, u64>> = LazyCell::new(|| {
    let mut out = BTreeMap::default();
    for lvl in 0..1024 {
        let level_exp = AbilityExpRecord::exp_for_level(lvl);
        out.insert(level_exp, lvl);
    }
    out
});

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct AbilityExpRecord {
    pub player_id: String,
    pub ability: Ability,
    pub amount: u64,
}

impl AbilityExpRecord {
    pub fn combine(&self, other: &Self) -> Self {
        assert_eq!(self.player_id, other.player_id);
        assert_eq!(self.ability, other.ability);
        AbilityExpRecord {
            player_id: self.player_id.clone(),
            ability: self.ability.clone(),
            amount: self.amount + other.amount,
        }
    }

    /// Scale this with
    /// y = x^4 + 100 * x - x^3
    /// y = exp
    /// x = level
    pub fn exp_for_level(target_level: u64) -> u64 {
        target_level.pow(4) + 100 * target_level - target_level.pow(3)
    }

    pub fn level_from_exp(exp: u64) -> u64 {
        if let Some((_exp, lvl)) = EXP_LEVEL_PRECALC.range(..=exp).last() {
            *lvl
        } else {
            println!("exp: {exp}");
            panic!("you've looked for a level that is not precalculated")
        }
    }

    /// Calculate the ability level based on the amount of stored experience
    pub fn calc_level(&self) -> u64 {
        // exp curve the same for all to start
        Self::level_from_exp(self.amount)
    }
}

impl AbilityExpRecord {
    pub fn init(db: &redb::Database) -> Result<()> {
        let write = db.begin_write()?;
        write.open_table(ABILITY_EXP_TABLE)?;
        write.commit()?;
        Ok(())
    }

    pub fn key(player_id: &str, ability: &Ability) -> Result<(Ability, String)> {
        Ok((ability.clone(), player_id.to_string()))
    }

    /// Register some new experience
    pub fn increment(db: &redb::Database, new_exp: &AbilityExpRecord) -> Result<Self> {
        let write = db.begin_write()?;
        let mut ability_exp_table = write.open_table(ABILITY_EXP_TABLE)?;
        let key = Self::key(&new_exp.player_id, &new_exp.ability)?;
        let new_record = match ability_exp_table.get(&key)? {
            Some(old_record) => {
                let old_record = old_record.value();
                old_record.combine(new_exp)
            }
            None => new_exp.clone(),
        };
        ability_exp_table.insert(key, new_record.clone())?;
        drop(ability_exp_table);
        write.commit()?;
        Ok(new_record)
    }
}

impl redb::Value for AbilityExpRecord {
    type SelfType<'a> = AbilityExpRecord;
    type AsBytes<'a> = Vec<u8>;

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        bincode::serialize(value).unwrap()
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::deserialize(data).unwrap()
    }
    fn type_name() -> redb::TypeName {
        redb::TypeName::new("AbilityExpRecord")
    }

    fn fixed_width() -> Option<usize> {
        None
    }
}

impl redb::Key for Ability {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        let ability1: Ability = bincode::deserialize(data1).unwrap();
        let ability2: Ability = bincode::deserialize(data2).unwrap();
        ability1.cmp(&ability2)
    }
}

impl redb::Value for Ability {
    type SelfType<'a> = Ability;
    type AsBytes<'a> = Vec<u8>;

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        bincode::serialize(value).unwrap()
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::deserialize(data).unwrap()
    }
    fn type_name() -> redb::TypeName {
        redb::TypeName::new("Ability")
    }

    fn fixed_width() -> Option<usize> {
        Some(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_values() {
        for i in 0..120 {
            let exp = AbilityExpRecord::exp_for_level(i);
            println!("Level {i} requires ~{exp} exp ");
            assert_eq!(AbilityExpRecord::level_from_exp(exp), i);
        }
    }
}
