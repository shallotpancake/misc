use bevy::prelude::*;

use super::ability::AbilityType;
use super::proficiency::Proficiency;

/// The 12 core PF2e character classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterClass {
    Alchemist,
    Barbarian,
    Bard,
    Champion,
    Cleric,
    Druid,
    Fighter,
    Monk,
    Ranger,
    Rogue,
    Sorcerer,
    Wizard,
}

/// Component: identifies an entity's class and level within that class.
/// This is just data — the mechanics layer reads it to apply class-based rules.
#[derive(Component, Debug, Clone)]
pub struct ClassData {
    pub class: CharacterClass,
    pub level: u32,
    pub key_ability: AbilityType,
}

/// Pure reference data describing what a class provides.
/// This is NOT a component — it's returned by a pure function.
/// The mechanics layer reads this to compute derived values.
#[derive(Debug, Clone)]
pub struct ClassProgression {
    pub hit_points_per_level: u32,
    pub key_ability: AbilityType,
    pub perception_proficiency: Proficiency,
    pub fortitude_proficiency: Proficiency,
    pub reflex_proficiency: Proficiency,
    pub will_proficiency: Proficiency,
    pub attack_proficiency: Proficiency,
    pub defense_proficiency: Proficiency,
}

impl ClassProgression {
    /// Returns the correct PF2e progression data for each class.
    ///
    /// For classes with a Str/Dex key ability choice (Fighter, Ranger, Monk),
    /// this defaults to Strength. The actual choice is character-specific and
    /// should be set on the ClassData component when building the entity.
    pub fn for_class(class: CharacterClass) -> ClassProgression {
        match class {
            CharacterClass::Fighter => ClassProgression {
                hit_points_per_level: 10,
                // Fighter key ability is Str or Dex — defaulting to Strength.
                key_ability: AbilityType::Strength,
                perception_proficiency: Proficiency::Expert,
                fortitude_proficiency: Proficiency::Expert,
                reflex_proficiency: Proficiency::Expert,
                will_proficiency: Proficiency::Expert,
                attack_proficiency: Proficiency::Expert,
                defense_proficiency: Proficiency::Expert,
            },
            CharacterClass::Rogue => ClassProgression {
                hit_points_per_level: 8,
                key_ability: AbilityType::Dexterity,
                perception_proficiency: Proficiency::Expert,
                fortitude_proficiency: Proficiency::Trained,
                reflex_proficiency: Proficiency::Expert,
                will_proficiency: Proficiency::Expert,
                attack_proficiency: Proficiency::Trained,
                defense_proficiency: Proficiency::Trained,
            },
            CharacterClass::Wizard => ClassProgression {
                hit_points_per_level: 6,
                key_ability: AbilityType::Intelligence,
                perception_proficiency: Proficiency::Trained,
                fortitude_proficiency: Proficiency::Trained,
                reflex_proficiency: Proficiency::Trained,
                will_proficiency: Proficiency::Expert,
                attack_proficiency: Proficiency::Trained,
                defense_proficiency: Proficiency::Trained,
            },
            CharacterClass::Cleric => ClassProgression {
                hit_points_per_level: 8,
                key_ability: AbilityType::Wisdom,
                perception_proficiency: Proficiency::Trained,
                fortitude_proficiency: Proficiency::Trained,
                reflex_proficiency: Proficiency::Trained,
                will_proficiency: Proficiency::Expert,
                attack_proficiency: Proficiency::Trained,
                defense_proficiency: Proficiency::Trained,
            },
            CharacterClass::Ranger => ClassProgression {
                hit_points_per_level: 10,
                // Ranger key ability is Str or Dex — defaulting to Strength.
                key_ability: AbilityType::Strength,
                perception_proficiency: Proficiency::Expert,
                fortitude_proficiency: Proficiency::Expert,
                reflex_proficiency: Proficiency::Expert,
                will_proficiency: Proficiency::Trained,
                attack_proficiency: Proficiency::Expert,
                defense_proficiency: Proficiency::Trained,
            },
            CharacterClass::Barbarian => ClassProgression {
                hit_points_per_level: 12,
                key_ability: AbilityType::Strength,
                perception_proficiency: Proficiency::Expert,
                fortitude_proficiency: Proficiency::Expert,
                reflex_proficiency: Proficiency::Trained,
                will_proficiency: Proficiency::Expert,
                attack_proficiency: Proficiency::Trained,
                defense_proficiency: Proficiency::Trained,
            },
            CharacterClass::Bard => ClassProgression {
                hit_points_per_level: 8,
                key_ability: AbilityType::Charisma,
                perception_proficiency: Proficiency::Expert,
                fortitude_proficiency: Proficiency::Trained,
                reflex_proficiency: Proficiency::Expert,
                will_proficiency: Proficiency::Expert,
                attack_proficiency: Proficiency::Trained,
                defense_proficiency: Proficiency::Trained,
            },
            CharacterClass::Champion => ClassProgression {
                hit_points_per_level: 10,
                key_ability: AbilityType::Strength,
                perception_proficiency: Proficiency::Trained,
                fortitude_proficiency: Proficiency::Expert,
                reflex_proficiency: Proficiency::Trained,
                will_proficiency: Proficiency::Expert,
                attack_proficiency: Proficiency::Expert,
                defense_proficiency: Proficiency::Expert,
            },
            CharacterClass::Druid => ClassProgression {
                hit_points_per_level: 8,
                key_ability: AbilityType::Wisdom,
                perception_proficiency: Proficiency::Trained,
                fortitude_proficiency: Proficiency::Expert,
                reflex_proficiency: Proficiency::Trained,
                will_proficiency: Proficiency::Expert,
                attack_proficiency: Proficiency::Trained,
                defense_proficiency: Proficiency::Trained,
            },
            CharacterClass::Monk => ClassProgression {
                hit_points_per_level: 10,
                // Monk key ability is Str or Dex — defaulting to Strength.
                key_ability: AbilityType::Strength,
                perception_proficiency: Proficiency::Trained,
                fortitude_proficiency: Proficiency::Expert,
                reflex_proficiency: Proficiency::Expert,
                will_proficiency: Proficiency::Expert,
                // Expert in unarmed attacks.
                attack_proficiency: Proficiency::Expert,
                defense_proficiency: Proficiency::Trained,
            },
            CharacterClass::Sorcerer => ClassProgression {
                hit_points_per_level: 6,
                key_ability: AbilityType::Charisma,
                perception_proficiency: Proficiency::Trained,
                fortitude_proficiency: Proficiency::Trained,
                reflex_proficiency: Proficiency::Trained,
                will_proficiency: Proficiency::Expert,
                attack_proficiency: Proficiency::Trained,
                defense_proficiency: Proficiency::Trained,
            },
            CharacterClass::Alchemist => ClassProgression {
                hit_points_per_level: 8,
                key_ability: AbilityType::Intelligence,
                perception_proficiency: Proficiency::Trained,
                fortitude_proficiency: Proficiency::Expert,
                reflex_proficiency: Proficiency::Trained,
                will_proficiency: Proficiency::Expert,
                attack_proficiency: Proficiency::Trained,
                defense_proficiency: Proficiency::Trained,
            },
        }
    }
}

/// Compute max HP contribution from class levels.
///
/// This is a pure function: class_hp_per_level * level + con_modifier * level.
/// Ancestry HP is added separately — this only covers the class contribution.
pub fn compute_max_hp(class: &ClassProgression, level: u32, con_modifier: i32) -> i32 {
    let level_i32 = level as i32;
    (class.hit_points_per_level as i32) * level_i32 + con_modifier * level_i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fighter_progression() {
        let prog = ClassProgression::for_class(CharacterClass::Fighter);
        assert_eq!(prog.hit_points_per_level, 10);
        assert_eq!(prog.perception_proficiency, Proficiency::Expert);
        assert_eq!(prog.fortitude_proficiency, Proficiency::Expert);
        assert_eq!(prog.reflex_proficiency, Proficiency::Expert);
        assert_eq!(prog.will_proficiency, Proficiency::Expert);
        assert_eq!(prog.attack_proficiency, Proficiency::Expert);
        assert_eq!(prog.defense_proficiency, Proficiency::Expert);
    }

    #[test]
    fn wizard_progression() {
        let prog = ClassProgression::for_class(CharacterClass::Wizard);
        assert_eq!(prog.hit_points_per_level, 6);
        assert_eq!(prog.key_ability, AbilityType::Intelligence);
        assert_eq!(prog.perception_proficiency, Proficiency::Trained);
        assert_eq!(prog.fortitude_proficiency, Proficiency::Trained);
        assert_eq!(prog.reflex_proficiency, Proficiency::Trained);
        assert_eq!(prog.will_proficiency, Proficiency::Expert);
        assert_eq!(prog.attack_proficiency, Proficiency::Trained);
        assert_eq!(prog.defense_proficiency, Proficiency::Trained);
    }

    #[test]
    fn hp_computation_positive_con() {
        let prog = ClassProgression::for_class(CharacterClass::Fighter);
        // Level 5 Fighter with +3 CON: (10 * 5) + (3 * 5) = 65
        assert_eq!(compute_max_hp(&prog, 5, 3), 65);
    }

    #[test]
    fn hp_computation_zero_con() {
        let prog = ClassProgression::for_class(CharacterClass::Wizard);
        // Level 3 Wizard with +0 CON: (6 * 3) + (0 * 3) = 18
        assert_eq!(compute_max_hp(&prog, 3, 0), 18);
    }

    #[test]
    fn hp_computation_negative_con() {
        let prog = ClassProgression::for_class(CharacterClass::Wizard);
        // Level 4 Wizard with -1 CON: (6 * 4) + (-1 * 4) = 20
        assert_eq!(compute_max_hp(&prog, 4, -1), 20);
    }

    #[test]
    fn hp_computation_level_one() {
        let prog = ClassProgression::for_class(CharacterClass::Barbarian);
        // Level 1 Barbarian with +4 CON: (12 * 1) + (4 * 1) = 16
        assert_eq!(compute_max_hp(&prog, 1, 4), 16);
    }

    #[test]
    fn all_classes_have_valid_progressions() {
        let all_classes = [
            CharacterClass::Alchemist,
            CharacterClass::Barbarian,
            CharacterClass::Bard,
            CharacterClass::Champion,
            CharacterClass::Cleric,
            CharacterClass::Druid,
            CharacterClass::Fighter,
            CharacterClass::Monk,
            CharacterClass::Ranger,
            CharacterClass::Rogue,
            CharacterClass::Sorcerer,
            CharacterClass::Wizard,
        ];

        for class in &all_classes {
            let prog = ClassProgression::for_class(*class);
            // Every class should have at least 6 HP per level.
            assert!(
                prog.hit_points_per_level >= 6,
                "{:?} has less than 6 HP per level",
                class
            );
            // Will save should be at least Trained for all classes.
            assert!(
                prog.will_proficiency >= Proficiency::Trained,
                "{:?} has Untrained will save",
                class
            );
            // HP computation should not panic.
            let hp = compute_max_hp(&prog, 10, 2);
            assert!(hp > 0, "{:?} produced non-positive HP at level 10", class);
        }
    }
}
