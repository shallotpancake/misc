//! Enemy / NPC stat block system.
//!
//! Enemies are "matter" in the spacetime metaphor — they are described by
//! inert data (components) and operated on by universal mechanics. A goblin
//! warrior doesn't know *how* attack rolls work; it just has an attack bonus
//! that the mechanics layer consumes.

use bevy::prelude::*;

use crate::action::ActionCost;
use crate::mechanics::dice::Die;
use crate::mechanics::traits::GameTrait;

// ---------------------------------------------------------------------------
// Creature classification
// ---------------------------------------------------------------------------

/// PF2e creature types — the broad biological/magical classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreatureType {
    Aberration,
    Animal,
    Beast,
    Celestial,
    Construct,
    Dragon,
    Elemental,
    Fey,
    Fiend,
    Giant,
    Humanoid,
    Monitor,
    Ooze,
    Plant,
    Undead,
}

// ---------------------------------------------------------------------------
// Creature size
// ---------------------------------------------------------------------------

/// PF2e creature sizes, from Tiny to Gargantuan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CreatureSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

impl CreatureSize {
    /// The number of feet of space this creature occupies on the grid.
    /// Tiny creatures technically occupy 2.5 ft, but on a standard grid
    /// they share a 5-ft square, so we round up to 5 for grid purposes.
    pub fn space_in_feet(&self) -> u32 {
        match self {
            CreatureSize::Tiny => 5, // 2.5 ft rounded to 5 for grid
            CreatureSize::Small | CreatureSize::Medium => 5,
            CreatureSize::Large => 10,
            CreatureSize::Huge => 15,
            CreatureSize::Gargantuan => 20,
        }
    }

    /// The default melee reach for a creature of this size (in feet).
    /// Tiny creatures have 0 reach (they must enter a foe's square).
    pub fn reach_in_feet(&self) -> u32 {
        match self {
            CreatureSize::Tiny => 0,
            CreatureSize::Small | CreatureSize::Medium => 5,
            CreatureSize::Large => 10,
            CreatureSize::Huge => 15,
            CreatureSize::Gargantuan => 20,
        }
    }
}

// ---------------------------------------------------------------------------
// Senses
// ---------------------------------------------------------------------------

/// Creature senses. Parameterized variants carry range in feet.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sense {
    Darkvision,
    LowLightVision,
    /// Scent with a range in feet (e.g., 30).
    Scent(u32),
    /// Tremorsense with a range in feet.
    Tremorsense(u32),
    /// Blindsight with a range in feet.
    Blindsight(u32),
}

// ---------------------------------------------------------------------------
// Damage resistance / weakness
// ---------------------------------------------------------------------------

/// A resistance to a specific damage type for a fixed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageResistance {
    pub damage_type: String,
    pub value: u32,
}

impl DamageResistance {
    pub fn new(damage_type: impl Into<String>, value: u32) -> Self {
        Self {
            damage_type: damage_type.into(),
            value,
        }
    }
}

/// A weakness to a specific damage type for a fixed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageWeakness {
    pub damage_type: String,
    pub value: u32,
}

impl DamageWeakness {
    pub fn new(damage_type: impl Into<String>, value: u32) -> Self {
        Self {
            damage_type: damage_type.into(),
            value,
        }
    }
}

// ---------------------------------------------------------------------------
// EnemyData component
// ---------------------------------------------------------------------------

/// Component: the stat-block data unique to enemies/NPCs.
/// This sits alongside the standard creature components (HitPoints,
/// ArmorClass, etc.) to add enemy-specific metadata.
#[derive(Component, Debug, Clone)]
pub struct EnemyData {
    pub creature_type: CreatureType,
    pub size: CreatureSize,
    /// Alignment as a free-form string (e.g., "CE", "LG", "N").
    pub alignment: String,
    pub senses: Vec<Sense>,
    pub languages: Vec<String>,
    pub immunities: Vec<String>,
    pub resistances: Vec<DamageResistance>,
    pub weaknesses: Vec<DamageWeakness>,
}

// ---------------------------------------------------------------------------
// Attack data
// ---------------------------------------------------------------------------

/// A single melee or ranged Strike available to a creature.
#[derive(Debug, Clone)]
pub struct AttackData {
    pub name: String,
    pub attack_bonus: i32,
    pub damage_dice: Die,
    pub damage_dice_count: u32,
    pub damage_bonus: i32,
    pub damage_type: String,
    pub traits: Vec<GameTrait>,
    pub reach_in_feet: u32,
}

// ---------------------------------------------------------------------------
// Special abilities
// ---------------------------------------------------------------------------

/// A named special ability with an action cost and description.
#[derive(Debug, Clone)]
pub struct SpecialAbility {
    pub name: String,
    pub action_cost: ActionCost,
    pub description: String,
    pub traits: Vec<GameTrait>,
}

// ---------------------------------------------------------------------------
// EnemyAbilities component
// ---------------------------------------------------------------------------

/// Component: the offensive capabilities of an enemy — strikes and
/// special abilities. Separate from EnemyData so the mechanics layer
/// can query attacks independently of creature metadata.
#[derive(Component, Debug, Clone)]
pub struct EnemyAbilities {
    pub strikes: Vec<AttackData>,
    pub special_abilities: Vec<SpecialAbility>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creature_size_space() {
        assert_eq!(CreatureSize::Tiny.space_in_feet(), 5);
        assert_eq!(CreatureSize::Small.space_in_feet(), 5);
        assert_eq!(CreatureSize::Medium.space_in_feet(), 5);
        assert_eq!(CreatureSize::Large.space_in_feet(), 10);
        assert_eq!(CreatureSize::Huge.space_in_feet(), 15);
        assert_eq!(CreatureSize::Gargantuan.space_in_feet(), 20);
    }

    #[test]
    fn creature_size_reach() {
        assert_eq!(CreatureSize::Tiny.reach_in_feet(), 0);
        assert_eq!(CreatureSize::Small.reach_in_feet(), 5);
        assert_eq!(CreatureSize::Medium.reach_in_feet(), 5);
        assert_eq!(CreatureSize::Large.reach_in_feet(), 10);
        assert_eq!(CreatureSize::Huge.reach_in_feet(), 15);
        assert_eq!(CreatureSize::Gargantuan.reach_in_feet(), 20);
    }

    #[test]
    fn damage_resistance_construction() {
        let resist = DamageResistance::new("fire", 5);
        assert_eq!(resist.damage_type, "fire");
        assert_eq!(resist.value, 5);
    }

    #[test]
    fn damage_weakness_construction() {
        let weak = DamageWeakness::new("cold iron", 2);
        assert_eq!(weak.damage_type, "cold iron");
        assert_eq!(weak.value, 2);
    }
}
