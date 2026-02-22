//! # Bestiary — Factory functions for iconic PF2e enemies
//!
//! Each function is a pure factory: it returns an `EnemyStatBlock` containing
//! all the component data needed to spawn a fully-formed enemy entity.
//! No systems, no side effects — just data assembly.
//!
//! These are the "blueprints" that describe what matter looks like before
//! it is placed into the spacetime field via `Commands`.

use bevy::prelude::*;

use crate::action::ActionPool;
use crate::condition::Conditions;
use crate::mechanics::dice::Die;
use crate::mechanics::traits::{GameTrait, TraitCategory};
use crate::spatial::position::Position;
use crate::spatial::GridPosition;

use super::ability::AbilityScores;
use super::creature::{ArmorClass, Creature, CreatureName, HitPoints, Level, Speed};
use super::enemy::{
    AttackData, CreatureSize, CreatureType, DamageResistance, DamageWeakness, EnemyAbilities,
    EnemyData, Sense, SpecialAbility,
};
use super::proficiency::{SavingThrows, SkillProficiencies};
use crate::action::ActionCost;

// ---------------------------------------------------------------------------
// EnemyStatBlock — the spawn-ready data bundle
// ---------------------------------------------------------------------------

/// A complete enemy stat block containing all component data needed to
/// spawn the creature as a Bevy entity. This is not itself a Bundle;
/// instead it provides a `spawn` method that inserts the correct
/// components onto an entity via `Commands`.
#[derive(Debug, Clone)]
pub struct EnemyStatBlock {
    pub name: String,
    pub level: i32,
    pub hp: i32,
    pub ac: i32,
    pub speed: u32,
    pub ability_scores: AbilityScores,
    pub enemy_data: EnemyData,
    pub enemy_abilities: EnemyAbilities,
}

impl EnemyStatBlock {
    /// Spawn this enemy into the world, returning the `Entity` id.
    /// The creature is placed at grid origin (0, 0) — the caller can
    /// reposition it after spawning.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        commands
            .spawn((
                Creature,
                CreatureName(self.name),
                Level(self.level),
                HitPoints::new(self.hp),
                ArmorClass(self.ac),
                Speed(self.speed),
                self.ability_scores,
                SavingThrows::default(),
                SkillProficiencies::default(),
                Conditions::default(),
                GridPosition(Position::new(0, 0)),
                ActionPool::new_turn(),
                self.enemy_data,
                self.enemy_abilities,
            ))
            .id()
    }
}

// ---------------------------------------------------------------------------
// Helper builders
// ---------------------------------------------------------------------------

fn melee_strike(
    name: &str,
    attack_bonus: i32,
    dice: Die,
    dice_count: u32,
    damage_bonus: i32,
    damage_type: &str,
    traits: Vec<GameTrait>,
    reach: u32,
) -> AttackData {
    AttackData {
        name: name.into(),
        attack_bonus,
        damage_dice: dice,
        damage_dice_count: dice_count,
        damage_bonus,
        damage_type: damage_type.into(),
        traits,
        reach_in_feet: reach,
    }
}

// ---------------------------------------------------------------------------
// Bestiary entries
// ---------------------------------------------------------------------------

/// **Goblin Warrior** — Level -1 Creature
///
/// Small, scrappy humanoid. Weak individually but dangerous in numbers.
/// AC 16, HP 6, Speed 25 ft.
pub fn goblin_warrior() -> EnemyStatBlock {
    EnemyStatBlock {
        name: "Goblin Warrior".into(),
        level: -1,
        hp: 6,
        ac: 16,
        speed: 25,
        ability_scores: AbilityScores {
            strength: 12,
            dexterity: 16,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        },
        enemy_data: EnemyData {
            creature_type: CreatureType::Humanoid,
            size: CreatureSize::Small,
            alignment: "CE".into(),
            senses: vec![Sense::Darkvision],
            languages: vec!["Common".into(), "Goblin".into()],
            immunities: vec![],
            resistances: vec![],
            weaknesses: vec![],
        },
        enemy_abilities: EnemyAbilities {
            strikes: vec![
                melee_strike(
                    "Dogslicer",
                    8,       // attack bonus
                    Die::D6, // 1d6
                    1,
                    1,       // +1 damage
                    "slashing",
                    vec![
                        GameTrait::new("Agile", TraitCategory::Attack),
                        GameTrait::new("Backstabber", TraitCategory::Custom),
                        GameTrait::new("Finesse", TraitCategory::Custom),
                    ],
                    5, // reach
                ),
                melee_strike(
                    "Shortbow",
                    7,
                    Die::D6,
                    1,
                    0,
                    "piercing",
                    vec![GameTrait::new("Deadly d10", TraitCategory::Custom)],
                    0, // ranged, reach not applicable
                ),
            ],
            special_abilities: vec![SpecialAbility {
                name: "Goblin Scuttle".into(),
                action_cost: ActionCost::Reaction,
                description: "When a goblin ally adjacent to the warrior uses a move action, the warrior can Step.".into(),
                traits: vec![],
            }],
        },
    }
}

/// **Skeleton Guard** — Level -1 Creature
///
/// Undead. Immune to death effects, disease, paralyzed, poison, unconscious.
/// AC 16, HP 4, Speed 25 ft.
pub fn skeleton_guard() -> EnemyStatBlock {
    EnemyStatBlock {
        name: "Skeleton Guard".into(),
        level: -1,
        hp: 4,
        ac: 16,
        speed: 25,
        ability_scores: AbilityScores {
            strength: 12,
            dexterity: 14,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        },
        enemy_data: EnemyData {
            creature_type: CreatureType::Undead,
            size: CreatureSize::Medium,
            alignment: "NE".into(),
            senses: vec![Sense::Darkvision],
            languages: vec![],
            immunities: vec![
                "death effects".into(),
                "disease".into(),
                "paralyzed".into(),
                "poison".into(),
                "unconscious".into(),
            ],
            resistances: vec![
                DamageResistance::new("cold", 5),
                DamageResistance::new("electricity", 5),
                DamageResistance::new("fire", 5),
                DamageResistance::new("piercing", 5),
                DamageResistance::new("slashing", 5),
            ],
            weaknesses: vec![DamageWeakness::new("bludgeoning", 5)],
        },
        enemy_abilities: EnemyAbilities {
            strikes: vec![
                melee_strike(
                    "Scimitar",
                    7,
                    Die::D6,
                    1,
                    1,
                    "slashing",
                    vec![
                        GameTrait::new("Forceful", TraitCategory::Custom),
                        GameTrait::new("Sweep", TraitCategory::Custom),
                    ],
                    5,
                ),
                melee_strike(
                    "Claw",
                    7,
                    Die::D4,
                    1,
                    1,
                    "slashing",
                    vec![GameTrait::new("Agile", TraitCategory::Attack)],
                    5,
                ),
            ],
            special_abilities: vec![],
        },
    }
}

/// **Kobold Scout** — Level -1 Creature
///
/// Small reptilian humanoid. Sneaky and trap-savvy.
/// AC 16, HP 8, Speed 25 ft.
pub fn kobold_scout() -> EnemyStatBlock {
    EnemyStatBlock {
        name: "Kobold Scout".into(),
        level: -1,
        hp: 8,
        ac: 16,
        speed: 25,
        ability_scores: AbilityScores {
            strength: 8,
            dexterity: 16,
            constitution: 10,
            intelligence: 12,
            wisdom: 12,
            charisma: 10,
        },
        enemy_data: EnemyData {
            creature_type: CreatureType::Humanoid,
            size: CreatureSize::Small,
            alignment: "LE".into(),
            senses: vec![Sense::Darkvision],
            languages: vec!["Common".into(), "Draconic".into()],
            immunities: vec![],
            resistances: vec![],
            weaknesses: vec![],
        },
        enemy_abilities: EnemyAbilities {
            strikes: vec![
                melee_strike(
                    "Shortsword",
                    8,
                    Die::D6,
                    1,
                    0,
                    "piercing",
                    vec![
                        GameTrait::new("Agile", TraitCategory::Attack),
                        GameTrait::new("Finesse", TraitCategory::Custom),
                        GameTrait::new("Versatile S", TraitCategory::Custom),
                    ],
                    5,
                ),
                melee_strike(
                    "Crossbow",
                    8,
                    Die::D8,
                    1,
                    0,
                    "piercing",
                    vec![],
                    0, // ranged
                ),
            ],
            special_abilities: vec![SpecialAbility {
                name: "Sneak Attack".into(),
                action_cost: ActionCost::Free,
                description:
                    "The kobold scout deals an extra 1d6 precision damage to flat-footed creatures."
                        .into(),
                traits: vec![],
            }],
        },
    }
}

/// **Orc Warrior** — Level 1 Creature
///
/// Medium humanoid. Tough and aggressive.
/// AC 18, HP 23, Speed 25 ft.
pub fn orc_warrior() -> EnemyStatBlock {
    EnemyStatBlock {
        name: "Orc Warrior".into(),
        level: 1,
        hp: 23,
        ac: 18,
        speed: 25,
        ability_scores: AbilityScores {
            strength: 18,
            dexterity: 12,
            constitution: 16,
            intelligence: 10,
            wisdom: 12,
            charisma: 10,
        },
        enemy_data: EnemyData {
            creature_type: CreatureType::Humanoid,
            size: CreatureSize::Medium,
            alignment: "CE".into(),
            senses: vec![Sense::Darkvision],
            languages: vec!["Common".into(), "Orcish".into()],
            immunities: vec![],
            resistances: vec![],
            weaknesses: vec![],
        },
        enemy_abilities: EnemyAbilities {
            strikes: vec![
                melee_strike(
                    "Greataxe",
                    9,
                    Die::D12,
                    1,
                    4,
                    "slashing",
                    vec![GameTrait::new("Sweep", TraitCategory::Custom)],
                    5,
                ),
                melee_strike(
                    "Javelin",
                    5,
                    Die::D6,
                    1,
                    4,
                    "piercing",
                    vec![GameTrait::new("Thrown 30 ft.", TraitCategory::Custom)],
                    0, // ranged
                ),
            ],
            special_abilities: vec![SpecialAbility {
                name: "Ferocity".into(),
                action_cost: ActionCost::Reaction,
                description: "When the orc is reduced to 0 HP, it can use its reaction to remain at 1 HP instead. It gains the wounded 1 condition (or increases its wounded value by 1).".into(),
                traits: vec![],
            }],
        },
    }
}

/// **Zombie Shambler** — Level -1 Creature
///
/// Undead. Slow and mindless, but surprisingly durable.
/// AC 12, HP 20, Speed 20 ft (Slow).
pub fn zombie_shambler() -> EnemyStatBlock {
    EnemyStatBlock {
        name: "Zombie Shambler".into(),
        level: -1,
        hp: 20,
        ac: 12,
        speed: 20, // Slow
        ability_scores: AbilityScores {
            strength: 14,
            dexterity: 8,
            constitution: 16,
            intelligence: 1, // Mindless
            wisdom: 8,
            charisma: 6,
        },
        enemy_data: EnemyData {
            creature_type: CreatureType::Undead,
            size: CreatureSize::Medium,
            alignment: "NE".into(),
            senses: vec![Sense::Darkvision],
            languages: vec![],
            immunities: vec![
                "death effects".into(),
                "disease".into(),
                "mental".into(),
                "paralyzed".into(),
                "poison".into(),
                "unconscious".into(),
            ],
            resistances: vec![],
            weaknesses: vec![
                DamageWeakness::new("positive", 5),
                DamageWeakness::new("slashing", 2),
            ],
        },
        enemy_abilities: EnemyAbilities {
            strikes: vec![melee_strike(
                "Fist",
                7,
                Die::D6,
                1,
                2,
                "bludgeoning",
                vec![],
                5,
            )],
            special_abilities: vec![
                SpecialAbility {
                    name: "Slow".into(),
                    action_cost: ActionCost::Free,
                    description:
                        "A zombie shambler is permanently slowed 1 and can't use reactions."
                            .into(),
                    traits: vec![],
                },
                SpecialAbility {
                    name: "Mindless".into(),
                    action_cost: ActionCost::Free,
                    description:
                        "Immune to all mental effects. Can't take any action that requires thought."
                            .into(),
                    traits: vec![],
                },
            ],
        },
    }
}

/// **Giant Rat** — Level -1 Creature
///
/// Animal. A common low-level threat found in sewers and cellars.
/// AC 15, HP 8, Speed 30 ft.
pub fn giant_rat() -> EnemyStatBlock {
    EnemyStatBlock {
        name: "Giant Rat".into(),
        level: -1,
        hp: 8,
        ac: 15,
        speed: 30,
        ability_scores: AbilityScores {
            strength: 10,
            dexterity: 16,
            constitution: 12,
            intelligence: 2, // Animal intelligence
            wisdom: 12,
            charisma: 6,
        },
        enemy_data: EnemyData {
            creature_type: CreatureType::Animal,
            size: CreatureSize::Small,
            alignment: "N".into(),
            senses: vec![Sense::LowLightVision, Sense::Scent(30)],
            languages: vec![],
            immunities: vec![],
            resistances: vec![],
            weaknesses: vec![],
        },
        enemy_abilities: EnemyAbilities {
            strikes: vec![melee_strike(
                "Jaws",
                8,
                Die::D6,
                1,
                1,
                "piercing",
                vec![
                    GameTrait::new("Agile", TraitCategory::Attack),
                    GameTrait::new("Finesse", TraitCategory::Custom),
                ],
                5,
            )],
            special_abilities: vec![SpecialAbility {
                name: "Filth Fever".into(),
                action_cost: ActionCost::Free,
                description: "The giant rat's jaws deliver filth fever on a critical hit (DC 14 Fortitude save).".into(),
                traits: vec![GameTrait::new("Disease", TraitCategory::Custom)],
            }],
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::enemy::CreatureType;

    #[test]
    fn goblin_warrior_has_correct_stats() {
        let goblin = goblin_warrior();
        assert_eq!(goblin.name, "Goblin Warrior");
        assert_eq!(goblin.level, -1);
        assert_eq!(goblin.hp, 6);
        assert_eq!(goblin.ac, 16);
        assert_eq!(goblin.speed, 25);
        assert_eq!(goblin.enemy_data.creature_type, CreatureType::Humanoid);
        assert_eq!(goblin.enemy_data.size, CreatureSize::Small);
        assert!(!goblin.enemy_abilities.strikes.is_empty());
    }

    #[test]
    fn skeleton_is_undead() {
        let skeleton = skeleton_guard();
        assert_eq!(skeleton.enemy_data.creature_type, CreatureType::Undead);
        assert!(skeleton.enemy_data.immunities.contains(&"death effects".to_string()));
        assert!(skeleton.enemy_data.immunities.contains(&"poison".to_string()));
    }

    #[test]
    fn orc_warrior_is_level_1() {
        let orc = orc_warrior();
        assert_eq!(orc.level, 1);
        assert_eq!(orc.hp, 23);
        assert_eq!(orc.ac, 18);
        assert_eq!(orc.enemy_data.creature_type, CreatureType::Humanoid);
        assert_eq!(orc.enemy_data.size, CreatureSize::Medium);
    }

    #[test]
    fn zombie_is_slow_and_mindless() {
        let zombie = zombie_shambler();
        assert_eq!(zombie.speed, 20); // Slow
        assert_eq!(zombie.enemy_data.creature_type, CreatureType::Undead);
        let ability_names: Vec<&str> = zombie
            .enemy_abilities
            .special_abilities
            .iter()
            .map(|a| a.name.as_str())
            .collect();
        assert!(ability_names.contains(&"Slow"));
        assert!(ability_names.contains(&"Mindless"));
    }

    #[test]
    fn giant_rat_is_animal() {
        let rat = giant_rat();
        assert_eq!(rat.enemy_data.creature_type, CreatureType::Animal);
        assert!(rat.enemy_data.senses.contains(&Sense::Scent(30)));
    }

    #[test]
    fn all_bestiary_entries_have_valid_data() {
        let entries: Vec<EnemyStatBlock> = vec![
            goblin_warrior(),
            skeleton_guard(),
            kobold_scout(),
            orc_warrior(),
            zombie_shambler(),
            giant_rat(),
        ];

        for entry in &entries {
            assert!(entry.hp > 0, "{} should have positive HP", entry.name);
            assert!(entry.ac > 0, "{} should have positive AC", entry.name);
            assert!(!entry.name.is_empty(), "Name should not be empty");
            assert!(
                !entry.enemy_abilities.strikes.is_empty(),
                "{} should have at least one strike",
                entry.name
            );
        }
    }

    #[test]
    fn kobold_scout_stats() {
        let kobold = kobold_scout();
        assert_eq!(kobold.name, "Kobold Scout");
        assert_eq!(kobold.level, -1);
        assert_eq!(kobold.hp, 8);
        assert_eq!(kobold.ac, 16);
        assert_eq!(kobold.enemy_data.creature_type, CreatureType::Humanoid);
        assert_eq!(kobold.enemy_data.size, CreatureSize::Small);
    }
}
