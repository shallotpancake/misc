//! # Entity Layer (Layer 4 — "Matter")
//!
//! Entities are the things that exist within the mechanics field.
//! They are purely data — bundles of components. They don't define
//! how checks work, how modifiers stack, or how conditions propagate.
//! They just have numbers that feed into the mechanics layer.
//!
//! A creature is: stats + position + conditions + action pool.
//! The mechanics systems operate on these components uniformly.

pub mod ability;
pub mod bestiary;
pub mod class;
pub mod creature;
pub mod enemy;
pub mod item;
pub mod proficiency;

use bevy::prelude::*;

pub use ability::{AbilityScores, AbilityType};
pub use class::{CharacterClass, ClassData, ClassProgression, compute_max_hp};
pub use creature::CreatureBundle;
pub use item::{
    ArmorCategory, ArmorData, Bulk, ConsumableData, ConsumableType, DamageType, Inventory, Item,
    ItemRarity, ItemType, ShieldData, WeaponData, armor_modifier, weapon_modifier_bonus,
};
pub use proficiency::{Proficiency, SkillProficiencies, SavingThrows};
pub use enemy::{
    AttackData, CreatureSize, CreatureType, DamageResistance, DamageWeakness, EnemyAbilities,
    EnemyData, Sense, SpecialAbility,
};
pub use bestiary::EnemyStatBlock;

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, _app: &mut App) {
        // Entity layer is purely declarative — components, no systems.
        // The mechanics systems in other layers operate on these components.
    }
}
