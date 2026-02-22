//! # Game Layer (Layer 5 — "The Observer")
//!
//! Orchestrates the encounter: initiative order, turn progression,
//! round tracking. This layer coordinates the other layers but
//! doesn't define mechanics — it just asks the mechanics layer
//! to resolve things in the right order.

pub mod ai;
pub mod combat;
pub mod faction;
pub mod initiative;
pub mod runner;
pub mod sandbox;

use bevy::prelude::*;

pub use initiative::{EncounterPhase, EncounterState, InitiativeEntry, InitiativeOrder};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EncounterPhase>()
            .add_message::<TurnStartEvent>()
            .add_message::<TurnEndEvent>()
            .add_message::<RoundStartEvent>()
            .add_message::<EncounterStartEvent>()
            .add_message::<EncounterEndEvent>();
    }
}

/// Message: a creature's turn is starting.
#[derive(Message, Debug, Clone)]
pub struct TurnStartEvent {
    pub entity: Entity,
    pub round: u32,
}

/// Message: a creature's turn is ending.
#[derive(Message, Debug, Clone)]
pub struct TurnEndEvent {
    pub entity: Entity,
    pub round: u32,
}

/// Message: a new round is starting.
#[derive(Message, Debug, Clone)]
pub struct RoundStartEvent {
    pub round: u32,
}

/// Message: an encounter has begun.
#[derive(Message, Debug, Clone)]
pub struct EncounterStartEvent;

/// Message: an encounter has ended.
#[derive(Message, Debug, Clone)]
pub struct EncounterEndEvent;
