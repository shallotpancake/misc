use super::position::Position;
use super::terrain::Terrain;

/// The topology trait — any spatial system implements this.
/// This abstraction is what lets us swap 2D grid for hex or 3D later.
pub trait Topology {
    /// Whether a position is within the bounds of this space.
    fn in_bounds(&self, pos: Position) -> bool;

    /// The terrain at a given position.
    fn terrain_at(&self, pos: Position) -> Terrain;

    /// Set terrain at a position.
    fn set_terrain(&mut self, pos: Position, terrain: Terrain);

    /// Distance in feet between two positions (PF2e uses 5-foot squares).
    fn distance_in_feet(&self, from: Position, to: Position) -> u32;

    /// All positions adjacent to the given position.
    fn neighbors(&self, pos: Position) -> Vec<Position>;

    /// Whether two positions are adjacent (within 5 feet).
    fn is_adjacent(&self, a: Position, b: Position) -> bool {
        self.neighbors(a).contains(&b)
    }

    /// Movement cost along a specific path, checking terrain per cell.
    /// Returns None if any cell in the path is impassable.
    fn path_cost_in_feet(&self, path: &[Position]) -> Option<u32> {
        if path.len() < 2 {
            return Some(0);
        }
        let mut total = 0u32;
        let mut diagonal_count = 0u32;
        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];
            let terrain = self.terrain_at(to);
            let multiplier = terrain.movement_cost()?; // None = impassable
            let dx = (to.x - from.x).unsigned_abs();
            let dy = (to.y - from.y).unsigned_abs();
            let is_diagonal = dx == 1 && dy == 1;
            let base_cost = if is_diagonal {
                diagonal_count += 1;
                if diagonal_count % 2 == 0 { 10 } else { 5 }
            } else {
                5
            };
            total += base_cost * multiplier;
        }
        Some(total)
    }
}
