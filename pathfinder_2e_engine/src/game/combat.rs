//! Combat resolution — strike attacks and damage application.
//!
//! This module contains pure game logic functions for resolving melee and
//! ranged Strikes in Pathfinder 2e. These functions are called from a
//! synchronous game loop rather than being Bevy systems.
//!
//! The flow for a Strike is:
//! 1. Roll 1d20 and build a modifier stack (attack bonus, MAP, conditions)
//! 2. Resolve the check against the target's AC
//! 3. On a hit, roll damage dice; on a critical hit, double the dice count
//! 4. Apply damage to the target's hit points

use rand::Rng;

use crate::entity::creature::HitPoints;
use crate::entity::enemy::AttackData;
use crate::mechanics::check::{resolve_check, CheckContext, DegreeOfSuccess};
use crate::mechanics::dice::{Die, DicePool};
use crate::mechanics::modifier::{Modifier, ModifierStack, ModifierType};

/// Result of a Strike action, containing everything the game loop needs
/// to display feedback and update state.
#[derive(Debug, Clone)]
pub struct StrikeResult {
    /// The degree of success for the attack check.
    pub degree: DegreeOfSuccess,
    /// The natural d20 roll (1-20), before any modifiers.
    pub natural_roll: i32,
    /// The total check result (natural roll + all modifiers).
    pub check_total: i32,
    /// The amount of damage dealt. Zero on a miss or critical failure.
    pub damage_dealt: i32,
    /// A human-readable description of the strike outcome.
    pub description: String,
}

/// Resolve a Strike: roll the attack, determine the degree of success,
/// and roll damage if the attack hits.
///
/// This function implements the full PF2e Strike action resolution:
/// 1. Rolls 1d20 using `Die::D20.roll(rng)`
/// 2. Builds a `ModifierStack` with the attack bonus, MAP penalty, and
///    any extra modifiers from conditions, items, etc.
/// 3. Creates a `CheckContext` and calls `resolve_check` against the target AC
/// 4. On success, rolls normal damage; on critical success, doubles the dice
///    count (but not the flat damage bonus)
/// 5. Returns a `StrikeResult` with the full outcome
///
/// # Arguments
/// * `attack_bonus` - The attacker's total attack bonus (from `AttackData.attack_bonus`)
/// * `target_ac` - The target's Armor Class value
/// * `map_penalty` - Current Multiple Attack Penalty (0, -5, -10, etc.)
/// * `extra_modifiers` - Additional modifiers from conditions, flanking, etc.
/// * `attack` - The `AttackData` describing the weapon/attack being used
/// * `rng` - A mutable reference to a random number generator
pub fn resolve_strike(
    attack_bonus: i32,
    target_ac: i32,
    map_penalty: i32,
    extra_modifiers: &[Modifier],
    attack: &AttackData,
    rng: &mut impl Rng,
) -> StrikeResult {
    // Step 1: Roll the d20
    let natural_roll = Die::D20.roll(rng);

    // Step 2: Build the modifier stack
    let mut modifiers = ModifierStack::new()
        .with(Modifier::new(attack_bonus, ModifierType::Untyped, "attack bonus"));

    if map_penalty != 0 {
        modifiers = modifiers.with(Modifier::new(
            map_penalty,
            ModifierType::Untyped,
            "multiple attack penalty",
        ));
    }

    for modifier in extra_modifiers {
        modifiers = modifiers.with(modifier.clone());
    }

    // Step 3: Resolve the check against target AC
    let context = CheckContext {
        natural_roll,
        modifiers,
        dc: target_ac,
        degree_shifts: 0,
    };
    let check_result = resolve_check(&context);

    // Step 4: Roll damage if the attack hit
    let (damage_dealt, degree) = match check_result.final_degree {
        DegreeOfSuccess::CriticalSuccess => {
            let damage = roll_damage(attack, true, rng);
            (damage, DegreeOfSuccess::CriticalSuccess)
        }
        DegreeOfSuccess::Success => {
            let damage = roll_damage(attack, false, rng);
            (damage, DegreeOfSuccess::Success)
        }
        DegreeOfSuccess::Failure => (0, DegreeOfSuccess::Failure),
        DegreeOfSuccess::CriticalFailure => (0, DegreeOfSuccess::CriticalFailure),
    };

    // Step 5: Build the human-readable description
    let description = build_strike_description(
        &attack.name,
        natural_roll,
        check_result.total,
        target_ac,
        degree,
        damage_dealt,
        &attack.damage_type,
    );

    StrikeResult {
        degree,
        natural_roll,
        check_total: check_result.total,
        damage_dealt,
        description,
    }
}

/// Roll damage for a strike.
///
/// On a normal hit, rolls `damage_dice_count` dice of the attack's damage die
/// and adds the flat damage bonus. On a critical success, the number of damage
/// dice is doubled (but the flat bonus is NOT doubled), per PF2e rules.
///
/// The result is always at least 1 (minimum damage).
///
/// # Arguments
/// * `attack` - The `AttackData` containing dice type, count, and bonus
/// * `critical` - Whether this is a critical hit (doubles dice count)
/// * `rng` - A mutable reference to a random number generator
pub fn roll_damage(attack: &AttackData, critical: bool, rng: &mut impl Rng) -> i32 {
    let dice_count = if critical {
        attack.damage_dice_count * 2
    } else {
        attack.damage_dice_count
    };

    let pool = DicePool::new(dice_count, attack.damage_dice);
    let roll = pool.roll(rng);
    let total = roll.total() + attack.damage_bonus;

    // PF2e: minimum 1 damage on a hit
    total.max(1)
}

/// Apply damage to a creature's hit points.
///
/// Damage is first absorbed by temporary hit points, then applied to
/// current hit points. Returns `true` if the creature's current HP
/// dropped to 0 or below (i.e., the creature is dying/dead).
///
/// # Arguments
/// * `hp` - Mutable reference to the creature's `HitPoints`
/// * `damage` - The amount of damage to apply (must be non-negative)
pub fn apply_damage(hp: &mut HitPoints, damage: i32) -> bool {
    if damage <= 0 {
        return !hp.is_alive();
    }

    let mut remaining_damage = damage;

    // Temporary HP absorbs damage first
    if hp.temporary > 0 {
        let absorbed = remaining_damage.min(hp.temporary);
        hp.temporary -= absorbed;
        remaining_damage -= absorbed;
    }

    // Remaining damage reduces current HP
    hp.current -= remaining_damage;

    !hp.is_alive()
}

/// Build a human-readable description of a strike result.
fn build_strike_description(
    attack_name: &str,
    natural_roll: i32,
    check_total: i32,
    target_ac: i32,
    degree: DegreeOfSuccess,
    damage_dealt: i32,
    damage_type: &str,
) -> String {
    match degree {
        DegreeOfSuccess::CriticalSuccess => {
            format!(
                "{} (d20={}, total={} vs AC {}): Critical Hit! {} {} damage",
                attack_name, natural_roll, check_total, target_ac, damage_dealt, damage_type
            )
        }
        DegreeOfSuccess::Success => {
            format!(
                "{} (d20={}, total={} vs AC {}): Hit for {} {} damage",
                attack_name, natural_roll, check_total, target_ac, damage_dealt, damage_type
            )
        }
        DegreeOfSuccess::Failure => {
            format!(
                "{} (d20={}, total={} vs AC {}): Miss",
                attack_name, natural_roll, check_total, target_ac
            )
        }
        DegreeOfSuccess::CriticalFailure => {
            format!(
                "{} (d20={}, total={} vs AC {}): Critical Miss",
                attack_name, natural_roll, check_total, target_ac
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::dice::Die;
    use rand::SeedableRng;

    /// Helper: create a basic melee attack for testing.
    fn test_attack() -> AttackData {
        AttackData {
            name: "Longsword".to_string(),
            attack_bonus: 9,
            damage_dice: Die::D8,
            damage_dice_count: 1,
            damage_bonus: 4,
            damage_type: "slashing".to_string(),
            traits: vec![],
            reach_in_feet: 5,
        }
    }

    /// Helper: create a deterministic RNG from a seed.
    fn seeded_rng(seed: u64) -> rand::rngs::StdRng {
        rand::rngs::StdRng::seed_from_u64(seed)
    }

    #[test]
    fn roll_damage_normal_hit() {
        let attack = test_attack();
        let mut rng = seeded_rng(42);

        let damage = roll_damage(&attack, false, &mut rng);

        // 1d8 + 4: result must be in [5, 12] (die range 1-8 + 4)
        assert!(damage >= 5 && damage <= 12, "damage was {}", damage);
    }

    #[test]
    fn roll_damage_critical_doubles_dice() {
        let attack = test_attack();
        // Run many trials to verify critical damage range is wider
        let mut min_crit = i32::MAX;
        let mut max_crit = i32::MIN;

        for seed in 0..100 {
            let mut rng = seeded_rng(seed);
            let damage = roll_damage(&attack, true, &mut rng);
            min_crit = min_crit.min(damage);
            max_crit = max_crit.max(damage);
        }

        // Critical: 2d8 + 4, range [6, 20]
        assert!(min_crit >= 6, "min crit damage was {}", min_crit);
        assert!(max_crit <= 20, "max crit damage was {}", max_crit);
        // The max should be higher than normal max (12) in 100 trials
        assert!(max_crit > 12, "critical max {} should exceed normal max 12", max_crit);
    }

    #[test]
    fn apply_damage_reduces_hp() {
        let mut hp = HitPoints::new(20);
        let died = apply_damage(&mut hp, 8);

        assert!(!died);
        assert_eq!(hp.current, 12);
        assert!(hp.is_alive());
    }

    #[test]
    fn apply_damage_kills_at_zero() {
        let mut hp = HitPoints::new(10);
        let died = apply_damage(&mut hp, 10);

        assert!(died);
        assert_eq!(hp.current, 0);
        assert!(!hp.is_alive());
    }

    #[test]
    fn apply_damage_overkill() {
        let mut hp = HitPoints::new(10);
        let died = apply_damage(&mut hp, 25);

        assert!(died);
        assert_eq!(hp.current, -15);
    }

    #[test]
    fn apply_damage_absorbs_temporary_hp_first() {
        let mut hp = HitPoints::new(20);
        hp.temporary = 5;

        let died = apply_damage(&mut hp, 8);

        assert!(!died);
        assert_eq!(hp.temporary, 0);
        assert_eq!(hp.current, 17); // 8 - 5 temp = 3 to current; 20 - 3 = 17
    }

    #[test]
    fn apply_damage_zero_is_noop() {
        let mut hp = HitPoints::new(20);
        let died = apply_damage(&mut hp, 0);

        assert!(!died);
        assert_eq!(hp.current, 20);
    }

    #[test]
    fn resolve_strike_miss_deals_no_damage() {
        // Use a very high AC to guarantee a miss
        let attack = test_attack();
        let mut rng = seeded_rng(1);

        let result = resolve_strike(
            attack.attack_bonus,
            50, // impossibly high AC
            0,
            &[],
            &attack,
            &mut rng,
        );

        assert_eq!(result.damage_dealt, 0);
        assert!(
            result.degree == DegreeOfSuccess::Failure
                || result.degree == DegreeOfSuccess::CriticalFailure
        );
        assert!(result.description.contains("Miss"));
    }

    #[test]
    fn resolve_strike_hit_deals_damage() {
        // Use a very low AC to guarantee a hit
        let attack = test_attack();
        let mut rng = seeded_rng(42);

        let result = resolve_strike(
            attack.attack_bonus,
            1, // impossibly low AC
            0,
            &[],
            &attack,
            &mut rng,
        );

        assert!(result.damage_dealt > 0);
        assert!(
            result.degree == DegreeOfSuccess::Success
                || result.degree == DegreeOfSuccess::CriticalSuccess
        );
    }

    #[test]
    fn resolve_strike_map_penalty_applied() {
        let attack = test_attack();
        // Run with MAP=-5 and MAP=0 using the same seed; MAP=-5 should have a lower total
        let mut rng1 = seeded_rng(99);
        let mut rng2 = seeded_rng(99);

        let result_no_map = resolve_strike(
            attack.attack_bonus,
            15,
            0,
            &[],
            &attack,
            &mut rng1,
        );

        let result_with_map = resolve_strike(
            attack.attack_bonus,
            15,
            -5,
            &[],
            &attack,
            &mut rng2,
        );

        // Same natural roll, so total should differ by exactly 5
        assert_eq!(result_no_map.natural_roll, result_with_map.natural_roll);
        assert_eq!(
            result_no_map.check_total - result_with_map.check_total,
            5
        );
    }

    #[test]
    fn resolve_strike_description_contains_attack_name() {
        let attack = test_attack();
        let mut rng = seeded_rng(7);

        let result = resolve_strike(
            attack.attack_bonus,
            15,
            0,
            &[],
            &attack,
            &mut rng,
        );

        assert!(
            result.description.contains("Longsword"),
            "description should contain attack name: {}",
            result.description
        );
    }

    #[test]
    fn roll_damage_minimum_one() {
        // Create an attack with negative bonus to test minimum damage
        let attack = AttackData {
            name: "Weak Punch".to_string(),
            attack_bonus: 0,
            damage_dice: Die::D4,
            damage_dice_count: 1,
            damage_bonus: -10,
            damage_type: "bludgeoning".to_string(),
            traits: vec![],
            reach_in_feet: 5,
        };

        let mut rng = seeded_rng(0);
        let damage = roll_damage(&attack, false, &mut rng);

        assert_eq!(damage, 1, "minimum damage should be 1");
    }
}
