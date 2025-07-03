use rand::Rng;

use db::Ability;
use db::PlayerStats;

pub fn compute_damage<R: Rng>(
    attack_ability: &Ability,
    attacker: &PlayerStats,
    defender: &PlayerStats,
    rng: &mut R,
) -> u64 {
    // if accuracy is higher than avoidability, it becomes more likely that the attacker hits
    // if avoidabiltiy is higher than accuracy it becomes more likely the attacker misses
    // if they're the same it's 50/50
    //
    // the point at which the difference in accuracy/avoidability causes odds of hit/miss to become overwhelming
    let acc_curve = 30.0;
    let attacker_accuracy = attacker.accuracy_by_ability(attack_ability);
    let defender_avoidability = defender.avoidability_by_ability(attack_ability);

    let is_hit = if attacker_accuracy > defender_avoidability {
        let accuracy_diff = (attacker_accuracy - defender_avoidability) as f64;
        // clamp to 0.0..1.0 and then move to range 0..0.5
        let odds = (accuracy_diff / acc_curve).clamp(0.0, 1.0) / 2.0;
        rng.random_bool(0.5 + odds)
    } else if attacker_accuracy < defender_avoidability {
        let accuracy_diff = (defender_avoidability - attacker_accuracy) as f64;
        // clamp to 0.0..1.0 and then move to range 0..0.5
        let odds = (accuracy_diff / acc_curve).clamp(0.0, 1.0) / 2.0;
        rng.random_bool(0.5 - odds)
    } else {
        rng.random_bool(0.5)
    };
    if !is_hit {
        return 0;
    }
    // calculate a base hit amount based on strength
    let attacker_level = attacker.level_by_ability(attack_ability);

    let defender_level = defender.level_by_ability(attack_ability);
    let relative_level = attacker_level - defender_level.min(attacker_level);

    let min_hit_amount = relative_level * 2 + 1;
    let max_hit_amount = relative_level * 3 + 3;
    let hit_amount = rng.random_range(min_hit_amount..max_hit_amount);

    // reduce the damage based on armor amount
    //
    hit_amount
}
