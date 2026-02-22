//! # Spatial Layer (Layer 1 — "Geometry")
//!
//! Defines the topology of the space entities exist in. Like the geometry
//! of spacetime, this is a property of the field — entities are placed
//! within it, not the other way around.
//!
//! The core abstraction is the `Topology` trait: any spatial system
//! (2D grid, hex, 3D) implements it, and the rest of the engine
//! interacts through the trait. Swapping spatial implementations
//! doesn't touch any other layer.

pub mod grid2d;
pub mod movement;
pub mod position;
pub mod terrain;
pub mod topology;

use bevy::prelude::*;

use crate::EngineSet;

pub use grid2d::Grid2D;
pub use movement::{MovementRules, MovementType};
pub use position::{Direction, Position};
pub use terrain::{Terrain, TerrainEffect};
pub use topology::Topology;

pub struct SpatialPlugin;

impl Plugin for SpatialPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MoveRequestedEvent>()
            .add_message::<MoveResolvedEvent>()
            .add_systems(
                Update,
                resolve_movement_system.in_set(EngineSet::ResolveMechanics),
            );
    }
}

/// Message: something wants to move an entity.
#[derive(Message, Debug, Clone)]
pub struct MoveRequestedEvent {
    pub entity: Entity,
    pub from: Position,
    pub to: Position,
}

/// Message: a movement has been resolved by the spatial layer.
#[derive(Message, Debug, Clone)]
pub struct MoveResolvedEvent {
    pub entity: Entity,
    pub from: Position,
    pub to: Position,
    pub cost_in_feet: u32,
    pub blocked: bool,
    pub reason: Option<String>,
}

/// Component: an entity's position in the spatial field.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition(pub Position);

/// Bevy system: resolves movement requests against the current topology.
fn resolve_movement_system(
    topology: Option<Res<Grid2D>>,
    mut requests: MessageReader<MoveRequestedEvent>,
    mut results: MessageWriter<MoveResolvedEvent>,
) {
    let Some(grid) = topology else {
        for request in requests.read() {
            results.write(MoveResolvedEvent {
                entity: request.entity,
                from: request.from,
                to: request.to,
                cost_in_feet: 0,
                blocked: true,
                reason: Some("No spatial topology loaded".into()),
            });
        }
        return;
    };

    for request in requests.read() {
        if !grid.in_bounds(request.to) {
            results.write(MoveResolvedEvent {
                entity: request.entity,
                from: request.from,
                to: request.to,
                cost_in_feet: 0,
                blocked: true,
                reason: Some("Destination out of bounds".into()),
            });
            continue;
        }

        match grid.path_cost_in_feet(&[request.from, request.to]) {
            None => {
                results.write(MoveResolvedEvent {
                    entity: request.entity,
                    from: request.from,
                    to: request.to,
                    cost_in_feet: 0,
                    blocked: true,
                    reason: Some("Impassable terrain".into()),
                });
            }
            Some(cost) => {
                results.write(MoveResolvedEvent {
                    entity: request.entity,
                    from: request.from,
                    to: request.to,
                    cost_in_feet: cost,
                    blocked: false,
                    reason: None,
                });
            }
        }
    }
}
