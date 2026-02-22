//! Faction marking for the game.
//!
//! Each creature in an encounter belongs to a faction. The game layer uses
//! this to determine who is an ally and who is an enemy for targeting,
//! AI decisions, and victory conditions.

/// Faction marking for creatures in an encounter.
///
/// Player-controlled creatures belong to `Player`, while all hostile
/// NPCs and monsters belong to `Enemy`. This simple two-faction model
/// is sufficient for the core game loop; it can be extended later with
/// `Neutral` or `Ally` variants if needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Faction {
    /// A player-controlled creature.
    Player,
    /// A hostile NPC or monster.
    Enemy,
}

impl Faction {
    /// Returns `true` if this faction is hostile to `other`.
    pub fn is_hostile_to(&self, other: &Faction) -> bool {
        self != other
    }

    /// Returns `true` if this faction is friendly to `other`.
    pub fn is_friendly_to(&self, other: &Faction) -> bool {
        self == other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_hostile_to_enemy() {
        assert!(Faction::Player.is_hostile_to(&Faction::Enemy));
        assert!(Faction::Enemy.is_hostile_to(&Faction::Player));
    }

    #[test]
    fn same_faction_friendly() {
        assert!(Faction::Player.is_friendly_to(&Faction::Player));
        assert!(Faction::Enemy.is_friendly_to(&Faction::Enemy));
    }

    #[test]
    fn same_faction_not_hostile() {
        assert!(!Faction::Player.is_hostile_to(&Faction::Player));
        assert!(!Faction::Enemy.is_hostile_to(&Faction::Enemy));
    }
}
