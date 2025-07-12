/// Handles loading all ability experience, worn items, etc. and consolidating it
/// Buffs will not be stored at this level
/// TODO: figure out how to handle relogging to drop debuffs (debuffs probably won't be a thing for a while)
use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;

use super::Ability;
use super::AbilityExpRecord;
use crate::ability_exp_record::ABILITY_EXP_TABLE;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct PlayerStats {
    pub player_id: String,
    pub ability_exp: HashMap<Ability, AbilityExpRecord>,
}

impl PlayerStats {
    pub fn level_by_ability(&self, ability: &Ability) -> u64 {
        if let Some(exp_record) = self.ability_exp.get(ability) {
            exp_record.calc_level()
        } else {
            0
        }
    }

    /// Compute the accuracy of an attack using a certain ability
    pub fn avoidability_by_ability(&self, ability: &Ability) -> u64 {
        
        self.level_by_ability(ability)
    }

    /// Compute the accuracy of an attack using a certain ability
    pub fn accuracy_by_ability(&self, ability: &Ability) -> u64 {
        
        self.level_by_ability(ability)
    }

    /// A players level is defined simply as the summation of all ability levels
    pub fn total_level(&self) -> u64 {
        let mut out = 0u64;
        for v in self.ability_exp.values() {
            out += v.calc_level();
        }
        out
    }

    pub fn level(&self, ability: &Ability) -> u64 {
        if let Some(exp) = self.ability_exp.get(ability) {
            exp.calc_level()
        } else {
            0
        }
    }

    pub fn exp(&self, ability: &Ability) -> u64 {
        if let Some(record) = self.ability_exp.get(ability) {
            record.amount
        } else {
            0
        }
    }

    /// Percent of the way to next level (0.0 to 1.0)
    /// total experience
    /// experience for next level
    /// next level
    pub fn next_level(&self, ability: &Ability) -> (f64, u64, u64, u64) {
        let current_level = self.level(ability);
        let current_level_exp = AbilityExpRecord::exp_for_level(current_level);
        let next_level_exp = AbilityExpRecord::exp_for_level(current_level + 1);

        let exp = self.exp(ability);
        let exp_for_level = next_level_exp - current_level_exp;
        let current_exp_for_level = exp - current_level_exp;

        let percent = (current_exp_for_level as f64) / (exp_for_level as f64);

        (percent, exp, next_level_exp, current_level + 1)
    }

    pub fn increment(&mut self, new_exp: &AbilityExpRecord) {
        // let new_exp = AbilityExpRecord::increment(db, new_exp)?;
        if let Some(exp) = self.ability_exp.get(&new_exp.ability) {
            self.ability_exp
                .insert(new_exp.ability.clone(), exp.combine(new_exp));
        } else {
            self.ability_exp
                .insert(new_exp.ability.clone(), new_exp.clone());
        }
    }

    pub fn max_health(&self) -> u64 {
        const DEFAULT_HEALTH: u64 = 10;
        if let Some(exp) = self.ability_exp.get(&Ability::Health) {
            exp.calc_level() * DEFAULT_HEALTH + DEFAULT_HEALTH
        } else {
            DEFAULT_HEALTH
        }
    }
}

impl PlayerStats {
    pub fn increment_db(&mut self, db: &redb::Database, new_exp: &AbilityExpRecord) -> Result<()> {
        let new_exp = AbilityExpRecord::increment(db, new_exp)?;
        // if our local ability_exp doesn't match we need to panic or overwrite
        if new_exp != *self.ability_exp.get(&new_exp.ability).unwrap() {
            println!("WARNING: mismatch between db exp and in memory exp");
            println!(
                "    expected: {:?} actual: {:?}",
                new_exp,
                self.ability_exp.get(&new_exp.ability)
            );
        }
        Ok(())
    }

    pub fn by_id(db: &redb::Database, player_id: &str) -> Result<Self> {
        let mut out = Self::default();
        let read = db.begin_read()?;
        let ability_exp_table = read.open_table(ABILITY_EXP_TABLE)?;
        for ability in Ability::iter() {
            let key = AbilityExpRecord::key(player_id, &ability)?;
            if let Some(ability_exp) = ability_exp_table.get(key)? {
                out.ability_exp.insert(ability.clone(), ability_exp.value());
            }
        }
        Ok(out)
    }
}
