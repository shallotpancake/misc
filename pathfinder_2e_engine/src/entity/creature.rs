use bevy::prelude::*;

use super::ability::AbilityScores;
use super::proficiency::{SavingThrows, SkillProficiencies};
use crate::action::ActionPool;
use crate::condition::Conditions;
use crate::spatial::GridPosition;

/// Marker component identifying an entity as a creature.
#[derive(Component, Debug)]
pub struct Creature;

/// Component: a creature's level.
#[derive(Component, Debug, Clone, Copy)]
pub struct Level(pub i32);

/// Component: hit points (current and max).
#[derive(Component, Debug, Clone)]
pub struct HitPoints {
    pub current: i32,
    pub max: i32,
    pub temporary: i32,
}

impl HitPoints {
    pub fn new(max: i32) -> Self {
        Self {
            current: max,
            max,
            temporary: 0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    pub fn effective(&self) -> i32 {
        self.current + self.temporary
    }
}

/// Component: armor class (base value, before situational modifiers).
#[derive(Component, Debug, Clone, Copy)]
pub struct ArmorClass(pub i32);

/// Component: movement speed in feet.
#[derive(Component, Debug, Clone, Copy)]
pub struct Speed(pub u32);

/// Component: a creature's name for display/identification.
#[derive(Component, Debug, Clone)]
pub struct CreatureName(pub String);

/// A bundle of all components that make up a creature.
/// This is just data — the mechanics systems operate on these components.
#[derive(Bundle)]
pub struct CreatureBundle {
    pub creature: Creature,
    pub name: CreatureName,
    pub level: Level,
    pub hp: HitPoints,
    pub ac: ArmorClass,
    pub speed: Speed,
    pub abilities: AbilityScores,
    pub saves: SavingThrows,
    pub skills: SkillProficiencies,
    pub conditions: Conditions,
    pub position: GridPosition,
    pub actions: ActionPool,
}

impl CreatureBundle {
    /// Create a minimal creature with sensible defaults.
    pub fn new(name: impl Into<String>, level: i32, hp: i32, ac: i32) -> Self {
        use crate::spatial::position::Position;

        Self {
            creature: Creature,
            name: CreatureName(name.into()),
            level: Level(level),
            hp: HitPoints::new(hp),
            ac: ArmorClass(ac),
            speed: Speed(25),
            abilities: AbilityScores::default(),
            saves: SavingThrows::default(),
            skills: SkillProficiencies::default(),
            conditions: Conditions::default(),
            position: GridPosition(Position::new(0, 0)),
            actions: ActionPool::new_turn(),
        }
    }
}
