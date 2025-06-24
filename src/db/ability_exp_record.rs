use std::cell::LazyCell;
use std::collections::BTreeMap;

/// Experience will be received and applied upon dealing damage to a mob
/// Experience is applied to the stat that was used to deal the damage
///
use serde::Deserialize;
use serde::Serialize;
use strum::EnumIter;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq, EnumIter)]
#[repr(u8)]
pub enum Ability {
    Health = 0,
    Strength = 1,
    Dexterity = 2,
    Intelligence = 3,
}

impl Default for Ability {
    fn default() -> Self {
        Ability::Strength
    }
}

pub const ABILITY_EXP_TREE: &str = "player_ability_exp";

const EXP_LEVEL_PRECALC: LazyCell<BTreeMap<u64, u64>> = LazyCell::new(|| {
    let mut out = BTreeMap::default();
    for lvl in 0..1024 {
        let level_exp = AbilityExpRecord::exp_for_level(lvl);
        out.insert(level_exp, lvl);
    }
    out
});

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
            println!("exp: {}", exp);
            panic!("you've looked for a level that is not precalculated")
        }
    }

    /// Calculate the ability level based on the amount of stored experience
    pub fn calc_level(&self) -> u64 {
        // exp curve the same for all to start
        match self.ability {
            _ => Self::level_from_exp(self.amount),
        }
    }
}

#[cfg(feature = "server")]
mod server_only {
    use super::*;

    use anyhow::Result;
    use sled::IVec;

    impl Into<IVec> for AbilityExpRecord {
        fn into(self) -> IVec {
            let bytes = bincode::serialize(&self).unwrap();
            IVec::from(bytes)
        }
    }

    impl From<&[u8]> for AbilityExpRecord {
        fn from(value: &[u8]) -> Self {
            bincode::deserialize(value.as_ref()).unwrap()
        }
    }

    impl From<IVec> for AbilityExpRecord {
        fn from(value: IVec) -> Self {
            bincode::deserialize(value.as_ref()).unwrap()
        }
    }

    impl AbilityExpRecord {
        pub fn key(player_id: String, ability: &Ability) -> Result<Vec<u8>> {
            let mut ability_bytes = bincode::serialize(ability)?;
            let mut player_id_bytes = player_id.as_bytes().to_vec();
            //
            // these keys are serialized to little endian sled uses big endian for ordering
            // so by default iteration will happen in reverse
            //
            let mut key_bytes = vec![];
            key_bytes.append(&mut ability_bytes);
            key_bytes.append(&mut player_id_bytes);
            Ok(key_bytes)
        }

        /// Register some new experience
        pub fn increment(db: sled::Db, new_exp: &AbilityExpRecord) -> Result<Self> {
            let key = Self::key(new_exp.player_id.clone(), &new_exp.ability)?;
            let ability_exp_tree = db.open_tree(ABILITY_EXP_TREE)?;
            if let Some(old_bytes) =
                ability_exp_tree.fetch_and_update(key, |old_value| match old_value {
                    Some(bytes) => {
                        let current_record = Self::from(bytes);
                        Some(current_record.combine(new_exp))
                    }
                    None => Some(new_exp.clone()),
                })?
            {
                Ok(Self::from(old_bytes).combine(new_exp))
            } else {
                Ok(new_exp.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_values() {
        for i in 0..120 {
            let exp = AbilityExpRecord::exp_for_level(i);
            println!("Level {} requires ~{} exp ", i, exp);
            assert_eq!(AbilityExpRecord::level_from_exp(exp), i);
        }
    }
}
