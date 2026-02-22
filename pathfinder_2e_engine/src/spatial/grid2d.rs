use bevy::prelude::*;
use std::collections::HashMap;

use super::position::{Direction, Position};
use super::terrain::Terrain;
use super::topology::Topology;

/// A 2D square grid — the default PF2e spatial topology.
/// Each cell is a 5-foot square. Exists as a Bevy Resource.
#[derive(Resource, Debug, Clone)]
pub struct Grid2D {
    pub width: u32,
    pub height: u32,
    terrain: HashMap<(i32, i32), Terrain>,
}

impl Grid2D {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            terrain: HashMap::new(),
        }
    }
}

impl Topology for Grid2D {
    fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.y >= 0 && (pos.x as u32) < self.width && (pos.y as u32) < self.height
    }

    fn terrain_at(&self, pos: Position) -> Terrain {
        self.terrain
            .get(&(pos.x, pos.y))
            .copied()
            .unwrap_or(Terrain::Normal)
    }

    fn set_terrain(&mut self, pos: Position, terrain: Terrain) {
        self.terrain.insert((pos.x, pos.y), terrain);
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
}
