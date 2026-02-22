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
}
