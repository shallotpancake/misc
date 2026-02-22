//! # Mechanics Layer (Layer 0 — "The Laws of Physics")
//!
//! Pure game logic: how d20 checks resolve, how modifiers stack, dice math.
//! These are Bevy systems and events that encode the fundamental rules.
//! They know nothing about specific creatures — only about numbers in,
//! results out.

pub mod check;
pub mod dice;
pub mod modifier;
pub mod traits;

use bevy::prelude::*;

use crate::EngineSet;

pub use check::{CheckContext, CheckRequestedEvent, CheckResolvedEvent, DegreeOfSuccess};
pub use dice::{DicePool, DiceRoll, Die};
pub use modifier::{Modifier, ModifierStack, ModifierType};
pub use traits::{GameTrait, TraitCategory};

pub struct MechanicsPlugin;

impl Plugin for MechanicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CheckRequestedEvent>()
            .add_message::<CheckResolvedEvent>()
            .add_systems(
                Update,
                check::resolve_checks_system.in_set(EngineSet::ResolveMechanics),
            );
    }
}
