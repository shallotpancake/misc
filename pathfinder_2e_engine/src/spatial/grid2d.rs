use bevy::prelude::*;

use super::position::{Direction, Position};
use super::terrain::Terrain;
use super::topology::Topology;

/// A 2D square grid — the default PF2e spatial topology.
/// Each cell is a 5-foot square. Exists as a Bevy Resource.
#[derive(Resource, Debug, Clone)]
pub struct Grid2D {
    pub width: u32,
    pub height: u32,
    terrain: Vec<Terrain>,
}

impl Grid2D {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            terrain: vec![Terrain::Normal; (width * height) as usize],
        }
    }

    fn index(&self, pos: Position) -> Option<usize> {
        if self.in_bounds(pos) {
            Some((pos.y as u32 * self.width + pos.x as u32) as usize)
        } else {
            None
        }
    }
}

impl Topology for Grid2D {
    fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.y >= 0 && (pos.x as u32) < self.width && (pos.y as u32) < self.height
    }

    fn terrain_at(&self, pos: Position) -> Terrain {
        match self.index(pos) {
            Some(idx) => self.terrain[idx],
            None => Terrain::Normal,
        }
    }

    fn set_terrain(&mut self, pos: Position, terrain: Terrain) {
        if let Some(idx) = self.index(pos) {
            self.terrain[idx] = terrain;
        }
    }

    /// PF2e diagonal movement: every second diagonal costs 10 feet instead of 5.
    /// Simplified here as: cardinal steps * 5 + diagonal steps * 5, with
    /// every second diagonal adding an extra 5 feet.
    fn distance_in_feet(&self, from: Position, to: Position) -> u32 {
        let diag = from.diagonal_steps(&to);
        let card = from.cardinal_steps(&to);
        let extra_diag_cost = diag / 2; // every second diagonal costs double
        (card + diag + extra_diag_cost) * 5
    }

    fn neighbors(&self, pos: Position) -> Vec<Position> {
        Direction::all()
            .iter()
            .map(|d| {
                let (dx, dy) = d.offset();
                pos.offset(dx, dy)
            })
            .filter(|p| self.in_bounds(*p))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_bounds() {
        let grid = Grid2D::new(10, 10);
        assert!(grid.in_bounds(Position::new(0, 0)));
        assert!(grid.in_bounds(Position::new(9, 9)));
        assert!(!grid.in_bounds(Position::new(10, 0)));
        assert!(!grid.in_bounds(Position::new(-1, 0)));
    }

    #[test]
    fn default_terrain_is_normal() {
        let grid = Grid2D::new(10, 10);
        assert_eq!(grid.terrain_at(Position::new(5, 5)), Terrain::Normal);
    }

    #[test]
    fn set_and_get_terrain() {
        let mut grid = Grid2D::new(10, 10);
        grid.set_terrain(Position::new(3, 3), Terrain::Difficult);
        assert_eq!(grid.terrain_at(Position::new(3, 3)), Terrain::Difficult);
    }

    #[test]
    fn cardinal_distance() {
        let grid = Grid2D::new(20, 20);
        let a = Position::new(0, 0);
        let b = Position::new(4, 0);
        assert_eq!(grid.distance_in_feet(a, b), 20); // 4 squares * 5 feet
    }

    #[test]
    fn diagonal_distance_alternating() {
        let grid = Grid2D::new(20, 20);
        let a = Position::new(0, 0);
        let b = Position::new(4, 4);
        // 4 diagonal steps: 5 + 10 + 5 + 10 = 30
        // Formula: (4 + 4/2) * 5 = (4 + 2) * 5 = 30
        assert_eq!(grid.distance_in_feet(a, b), 30);
    }

    #[test]
    fn neighbors_corner() {
        let grid = Grid2D::new(10, 10);
        let neighbors = grid.neighbors(Position::new(0, 0));
        assert_eq!(neighbors.len(), 3); // East, South, SouthEast
    }

    #[test]
    fn neighbors_center() {
        let grid = Grid2D::new(10, 10);
        let neighbors = grid.neighbors(Position::new(5, 5));
        assert_eq!(neighbors.len(), 8);
    }

    #[test]
    fn path_cost_mixed_terrain() {
        let mut grid = Grid2D::new(10, 10);
        // Path: (0,0) -> (1,0) -> (2,0) -> (3,0)
        // (1,0) normal, (2,0) difficult, (3,0) normal
        grid.set_terrain(Position::new(2, 0), Terrain::Difficult);
        let path = vec![
            Position::new(0, 0),
            Position::new(1, 0),
            Position::new(2, 0),
            Position::new(3, 0),
        ];
        // Step 1: normal terrain, cardinal = 5 * 1 = 5
        // Step 2: difficult terrain, cardinal = 5 * 2 = 10
        // Step 3: normal terrain, cardinal = 5 * 1 = 5
        assert_eq!(grid.path_cost_in_feet(&path), Some(20));
    }

    #[test]
    fn path_cost_blocked_by_impassable() {
        let mut grid = Grid2D::new(10, 10);
        grid.set_terrain(Position::new(2, 0), Terrain::Impassable);
        let path = vec![
            Position::new(0, 0),
            Position::new(1, 0),
            Position::new(2, 0),
            Position::new(3, 0),
        ];
        assert_eq!(grid.path_cost_in_feet(&path), None);
    }

    #[test]
    fn path_cost_diagonal_alternation() {
        let grid = Grid2D::new(10, 10);
        // 4 diagonal steps: costs should alternate 5, 10, 5, 10
        let path = vec![
            Position::new(0, 0),
            Position::new(1, 1),
            Position::new(2, 2),
            Position::new(3, 3),
            Position::new(4, 4),
        ];
        // Step 1: diagonal #1 (odd) = 5
        // Step 2: diagonal #2 (even) = 10
        // Step 3: diagonal #3 (odd) = 5
        // Step 4: diagonal #4 (even) = 10
        // Total = 30
        assert_eq!(grid.path_cost_in_feet(&path), Some(30));
    }
}
