/// A position in abstract space. Z exists for future 3D support
/// but defaults to 0 for 2D grids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y, z: 0 }
    }

    pub fn new3d(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Chebyshev distance — basis of PF2e grid distance (king moves).
    pub fn chebyshev(&self, other: &Position) -> u32 {
        let dx = (self.x - other.x).unsigned_abs();
        let dy = (self.y - other.y).unsigned_abs();
        dx.max(dy)
    }

    /// How many diagonal steps are needed between two positions.
    pub fn diagonal_steps(&self, other: &Position) -> u32 {
        let dx = (self.x - other.x).unsigned_abs();
        let dy = (self.y - other.y).unsigned_abs();
        dx.min(dy)
    }

    /// How many cardinal (non-diagonal) steps remain after diagonals.
    pub fn cardinal_steps(&self, other: &Position) -> u32 {
        let dx = (self.x - other.x).unsigned_abs();
        let dy = (self.y - other.y).unsigned_abs();
        dx.abs_diff(dy)
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }
}

/// Cardinal and diagonal directions on a 2D grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Direction {
    pub fn offset(&self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::NorthWest => (-1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
        }
    }

    pub fn is_diagonal(&self) -> bool {
        matches!(
            self,
            Direction::NorthEast | Direction::NorthWest | Direction::SouthEast | Direction::SouthWest
        )
    }

    pub fn all() -> &'static [Direction; 8] {
        &[
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
            Direction::NorthEast,
            Direction::NorthWest,
            Direction::SouthEast,
            Direction::SouthWest,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chebyshev_distance() {
        let a = Position::new(0, 0);
        let b = Position::new(3, 4);
        assert_eq!(a.chebyshev(&b), 4);
    }

    #[test]
    fn diagonal_and_cardinal_steps() {
        let a = Position::new(0, 0);
        let b = Position::new(3, 5);
        assert_eq!(a.diagonal_steps(&b), 3);
        assert_eq!(a.cardinal_steps(&b), 2);
    }
}
