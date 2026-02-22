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
pub mod creature;
pub mod proficiency;

use bevy::prelude::*;

pub use ability::{AbilityScores, AbilityType};
pub use creature::CreatureBundle;
pub use proficiency::{Proficiency, SkillProficiencies, SavingThrows};

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, _app: &mut App) {
        // Entity layer is purely declarative — components, no systems.
        // The mechanics systems in other layers operate on these components.
    }
}
