use bevy::prelude::*;

/// A single entry in the initiative order.
#[derive(Debug, Clone)]
pub struct InitiativeEntry {
    pub entity: Entity,
    pub initiative: i32,
}

/// The initiative order for the current encounter.
/// Stored as a Bevy Resource — a property of the encounter, not of entities.
#[derive(Resource, Debug, Clone, Default)]
pub struct InitiativeOrder {
    entries: Vec<InitiativeEntry>,
}

impl InitiativeOrder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, entity: Entity, initiative: i32) {
        self.entries.push(InitiativeEntry { entity, initiative });
        // Sort descending by initiative (highest goes first).
        self.entries
            .sort_by(|a, b| b.initiative.cmp(&a.initiative));
    }

    pub fn order(&self) -> &[InitiativeEntry] {
        &self.entries
    }

    pub fn remove(&mut self, entity: Entity) {
        self.entries.retain(|e| e.entity != entity);
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Get the entity at position N in the order (0-indexed).
    pub fn at(&self, index: usize) -> Option<Entity> {
        self.entries.get(index).map(|e| e.entity)
    }
}

/// The phase of the encounter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EncounterPhase {
    #[default]
    NotStarted,
    RollingInitiative,
    InProgress,
    Ended,
}

/// Resource: the overall encounter state.
#[derive(Resource, Debug, Clone)]
pub struct EncounterState {
    pub phase: EncounterPhase,
    pub round: u32,
    pub current_turn_index: usize,
}

impl Default for EncounterState {
    fn default() -> Self {
        Self {
            phase: EncounterPhase::NotStarted,
            round: 0,
            current_turn_index: 0,
        }
    }
}

impl EncounterState {
    pub fn start(&mut self) {
        self.phase = EncounterPhase::InProgress;
        self.round = 1;
        self.current_turn_index = 0;
    }

    /// Advance to the next turn. Returns true if a new round started.
    pub fn advance_turn(&mut self, initiative_count: usize) -> bool {
        self.current_turn_index += 1;
        if self.current_turn_index >= initiative_count {
            self.current_turn_index = 0;
            self.round += 1;
            true
        } else {
            false
        }
    }

    pub fn end(&mut self) {
        self.phase = EncounterPhase::Ended;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::World;

    #[test]
    fn initiative_sorts_descending() {
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        let e3 = world.spawn_empty().id();

        let mut order = InitiativeOrder::new();
        order.add(e1, 10);
        order.add(e2, 20);
        order.add(e3, 15);

        assert_eq!(order.at(0), Some(e2)); // 20 goes first
        assert_eq!(order.at(1), Some(e3)); // 15 second
        assert_eq!(order.at(2), Some(e1)); // 10 third
    }

    #[test]
    fn encounter_turn_advancement() {
        let mut state = EncounterState::default();
        state.start();
        assert_eq!(state.round, 1);
        assert_eq!(state.current_turn_index, 0);

        let new_round = state.advance_turn(3);
        assert!(!new_round);
        assert_eq!(state.current_turn_index, 1);

        let new_round = state.advance_turn(3);
        assert!(!new_round);
        assert_eq!(state.current_turn_index, 2);

        let new_round = state.advance_turn(3);
        assert!(new_round); // wrapped around → new round
        assert_eq!(state.round, 2);
        assert_eq!(state.current_turn_index, 0);
    }
}
