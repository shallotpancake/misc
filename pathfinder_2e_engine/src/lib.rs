//! # Pathfinder 2e Mechanics Engine
//!
//! A Bevy-based game engine implementing the Pathfinder 2nd Edition ruleset.
//!
//! ## Architecture: The Spacetime Metaphor
//!
//! Like classical relativity describes spacetime as a fabric that all matter
//! exists upon and is affected by, this engine separates the **rules of
//! reality** (mechanics) from the **things that exist** (entities).
//!
//! - **Mechanics Systems** are the laws of physics — they define how checks
//!   resolve, how modifiers stack, how conditions propagate. They operate
//!   uniformly on all entities. No entity is special.
//!
//! - **Components** are properties of matter — hit points, ability scores,
//!   conditions, position. They are inert data. They don't contain behavior.
//!
//! - **Events** are interactions — a check is requested, damage is dealt,
//!   a condition is applied. Systems listen for events and process them
//!   according to the rules.
//!
//! This separation means you can change what entities exist without touching
//! the rules, and you can modify rules without restructuring entities.
//!
//! ## Layers
//!
//! 0. `mechanics` — Check resolution, modifier stacking, dice (pure logic)
//! 1. `spatial` — Grid topology, terrain, movement costs
//! 2. `condition` — Condition tracking, effect propagation
//! 3. `action` — Three-action economy, MAP, trait restrictions
//! 4. `entity` — Creature/item components (the "matter")
//! 5. `game` — Turn management, initiative, encounter orchestration

pub mod mechanics;
pub mod spatial;
pub mod condition;
pub mod action;
pub mod entity;
pub mod game;

use bevy::prelude::*;

/// System sets that define execution order across the engine.
/// This ensures deterministic processing: mechanics resolve before
/// effects propagate, just as physical laws resolve before consequences.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineSet {
    /// Resolve mechanics: checks, movement requests
    ResolveMechanics,
    /// Apply effects: condition decay, status propagation
    ApplyEffects,
}

/// The top-level plugin that registers all subsystems.
pub struct PathfinderPlugin;

impl Plugin for PathfinderPlugin {
    fn build(&self, app: &mut App) {
        // Define system ordering: mechanics resolve before effects propagate
        app.configure_sets(
            Update,
            EngineSet::ResolveMechanics.before(EngineSet::ApplyEffects),
        );

        app.add_plugins((
            mechanics::MechanicsPlugin,
            spatial::SpatialPlugin,
            condition::ConditionPlugin,
            action::ActionPlugin,
            entity::EntityPlugin,
            game::GamePlugin,
        ));
    }
}
