//! Simple enemy AI for the game loop.
//!
//! This module provides a basic decision-making function for enemy creatures.
//! The AI follows a straightforward priority system:
//! 1. If adjacent to the player, attack (Strike).
//! 2. If not adjacent, move toward the player.
//! 3. If no useful action is available, end the turn.
//!
//! This is intentionally simple. More sophisticated AI (flanking, ability
//! usage, retreat logic) can be layered on top later.

use std::collections::HashSet;

use crate::spatial::grid2d::Grid2D;
use crate::spatial::position::{Direction, Position};
use crate::spatial::terrain::Terrain;
use crate::spatial::topology::Topology;

/// An action chosen by the AI for an enemy creature's turn.
///
/// The game loop receives one of these decisions and translates it into
/// the appropriate game state changes (moving the creature, resolving
/// a Strike, or advancing to the next turn).
#[derive(Debug, Clone)]
pub enum AiAction {
    /// Move to an adjacent position (one step on the grid).
    MoveTo(Position),
    /// Attack with the strike at the given index in the creature's attack list.
    Strike {
        /// Index into the creature's `EnemyAbilities.strikes` vector.
        attack_index: usize,
    },
    /// End the turn because no useful action is available.
    EndTurn,
}

/// Choose the best action for an enemy creature.
///
/// Implements a simple priority-based AI:
/// - If adjacent to the player, attack using the first melee strike (index 0).
/// - If not adjacent and the creature has actions remaining, move toward the
///   player by finding the neighboring cell that is closest to the player,
///   not occupied, and not impassable.
/// - If neither action is possible, end the turn.
///
/// # Arguments
/// * `my_pos` - The enemy creature's current position on the grid
/// * `my_speed` - The creature's movement speed in feet (unused in this simple
///   version, but available for future enhancement)
/// * `actions_remaining` - How many actions the creature has left this turn
/// * `player_pos` - The player creature's current position
/// * `grid` - The game grid, used for adjacency checks and terrain queries
/// * `occupied` - Set of positions currently occupied by other creatures
pub fn choose_enemy_action(
    my_pos: Position,
    _my_speed: u32,
    actions_remaining: u32,
    player_pos: Position,
    grid: &Grid2D,
    occupied: &HashSet<Position>,
) -> AiAction {
    // No actions left: end turn
    if actions_remaining == 0 {
        return AiAction::EndTurn;
    }

    // If adjacent to the player, attack with the first strike
    if grid.is_adjacent(my_pos, player_pos) {
        return AiAction::Strike { attack_index: 0 };
    }

    // Not adjacent: try to move closer to the player
    if let Some(move_target) = find_move_toward(my_pos, player_pos, grid, occupied) {
        return AiAction::MoveTo(move_target);
    }

    // Can't attack and can't move: give up
    AiAction::EndTurn
}

/// Find the best adjacent position to move to that gets closer to the target.
///
/// Examines all 8 neighbors of `from`, filters out positions that are:
/// - Out of bounds
/// - Occupied by another creature
/// - Impassable terrain
///
/// Of the remaining candidates, returns the one with the smallest Chebyshev
/// distance to `target`. Ties are broken arbitrarily (by iteration order).
///
/// Returns `None` if no valid move exists.
fn find_move_toward(
    from: Position,
    target: Position,
    grid: &Grid2D,
    occupied: &HashSet<Position>,
) -> Option<Position> {
    let current_distance = from.chebyshev(&target);
    let mut best: Option<(Position, u32)> = None;

    for dir in Direction::all() {
        let (dx, dy) = dir.offset();
        let candidate = from.offset(dx, dy);

        // Skip out-of-bounds positions
        if !grid.in_bounds(candidate) {
            continue;
        }

        // Skip occupied positions
        if occupied.contains(&candidate) {
            continue;
        }

        // Skip impassable terrain
        if grid.terrain_at(candidate) == Terrain::Impassable {
            continue;
        }

        let dist = candidate.chebyshev(&target);

        // Only consider positions that are actually closer to the target
        if dist >= current_distance {
            continue;
        }

        match best {
            None => best = Some((candidate, dist)),
            Some((_, best_dist)) if dist < best_dist => {
                best = Some((candidate, dist));
            }
            _ => {}
        }
    }

    best.map(|(pos, _)| pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid(w: u32, h: u32) -> Grid2D {
        Grid2D::new(w, h)
    }

    #[test]
    fn attack_when_adjacent() {
        let grid = make_grid(10, 10);
        let my_pos = Position::new(3, 3);
        let player_pos = Position::new(3, 4); // directly south, adjacent
        let occupied = HashSet::new();

        let action = choose_enemy_action(my_pos, 25, 3, player_pos, &grid, &occupied);

        match action {
            AiAction::Strike { attack_index } => assert_eq!(attack_index, 0),
            other => panic!("Expected Strike, got {:?}", other),
        }
    }

    #[test]
    fn move_toward_player_when_not_adjacent() {
        let grid = make_grid(10, 10);
        let my_pos = Position::new(1, 1);
        let player_pos = Position::new(5, 5);
        let occupied = HashSet::new();

        let action = choose_enemy_action(my_pos, 25, 3, player_pos, &grid, &occupied);

        match action {
            AiAction::MoveTo(pos) => {
                // The chosen position should be closer to the player than our current position
                assert!(
                    pos.chebyshev(&player_pos) < my_pos.chebyshev(&player_pos),
                    "Expected to move closer; moved to {:?} (dist {}), was at {:?} (dist {})",
                    pos,
                    pos.chebyshev(&player_pos),
                    my_pos,
                    my_pos.chebyshev(&player_pos),
                );
            }
            other => panic!("Expected MoveTo, got {:?}", other),
        }
    }

    #[test]
    fn end_turn_when_no_actions() {
        let grid = make_grid(10, 10);
        let my_pos = Position::new(3, 3);
        let player_pos = Position::new(3, 4);
        let occupied = HashSet::new();

        let action = choose_enemy_action(my_pos, 25, 0, player_pos, &grid, &occupied);

        match action {
            AiAction::EndTurn => {} // expected
            other => panic!("Expected EndTurn, got {:?}", other),
        }
    }

    #[test]
    fn end_turn_when_surrounded() {
        let grid = make_grid(10, 10);
        let my_pos = Position::new(5, 5);
        let player_pos = Position::new(8, 8); // far away

        // Fill all 8 neighbors with occupied positions
        let mut occupied = HashSet::new();
        for dir in Direction::all() {
            let (dx, dy) = dir.offset();
            occupied.insert(my_pos.offset(dx, dy));
        }

        let action = choose_enemy_action(my_pos, 25, 3, player_pos, &grid, &occupied);

        match action {
            AiAction::EndTurn => {} // expected, can't move and not adjacent
            other => panic!("Expected EndTurn, got {:?}", other),
        }
    }

    #[test]
    fn avoids_impassable_terrain() {
        let mut grid = make_grid(10, 10);
        let my_pos = Position::new(3, 3);
        let player_pos = Position::new(5, 3); // east of us

        // Make the direct path impassable
        grid.set_terrain(Position::new(4, 3), Terrain::Impassable);
        grid.set_terrain(Position::new(4, 2), Terrain::Impassable);
        grid.set_terrain(Position::new(4, 4), Terrain::Impassable);

        let occupied = HashSet::new();

        let action = choose_enemy_action(my_pos, 25, 3, player_pos, &grid, &occupied);

        match action {
            AiAction::MoveTo(pos) => {
                // Should not move to impassable terrain
                assert_ne!(grid.terrain_at(pos), Terrain::Impassable);
            }
            AiAction::EndTurn => {
                // Acceptable if all closer cells are blocked
            }
            other => panic!("Expected MoveTo or EndTurn, got {:?}", other),
        }
    }

    #[test]
    fn find_move_toward_picks_closest() {
        let grid = make_grid(10, 10);
        let from = Position::new(2, 2);
        let target = Position::new(5, 5);
        let occupied = HashSet::new();

        let result = find_move_toward(from, target, &grid, &occupied);
        assert!(result.is_some());

        let chosen = result.unwrap();
        // Diagonal move toward (5,5) from (2,2) should be (3,3)
        assert_eq!(chosen, Position::new(3, 3));
    }

    #[test]
    fn find_move_toward_avoids_occupied() {
        let grid = make_grid(10, 10);
        // From (2,2) targeting (5,2): cardinal east. Multiple neighbors get closer.
        // Closer neighbors: (3,2) dist=2, (3,1) dist=2, (3,3) dist=2.
        // Block (3,2) and verify the AI picks one of the other two.
        let from = Position::new(2, 2);
        let target = Position::new(5, 2);
        let mut occupied = HashSet::new();
        occupied.insert(Position::new(3, 2)); // block the direct east path

        let result = find_move_toward(from, target, &grid, &occupied);
        assert!(result.is_some());

        let chosen = result.unwrap();
        assert_ne!(chosen, Position::new(3, 2));
        // Should still be closer to target than our starting position
        assert!(chosen.chebyshev(&target) < from.chebyshev(&target));
    }

    #[test]
    fn find_move_toward_returns_none_when_stuck() {
        let grid = make_grid(10, 10);
        let from = Position::new(5, 5);
        let target = Position::new(8, 8);

        // Occupy all neighbors
        let mut occupied = HashSet::new();
        for dir in Direction::all() {
            let (dx, dy) = dir.offset();
            occupied.insert(from.offset(dx, dy));
        }

        let result = find_move_toward(from, target, &grid, &occupied);
        assert!(result.is_none());
    }
}
