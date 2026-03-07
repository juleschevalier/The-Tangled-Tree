//! A position in the world grid.

/// A discrete (x, y) position in the world grid.
///
/// Origin (0, 0) is the top-left corner. X grows right, Y grows down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldPosition {
    pub x: u32,
    pub y: u32,
}

impl WorldPosition {
    /// Create a new world position.
    #[must_use]
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Manhattan distance to another position.
    #[must_use]
    pub fn manhattan_distance(self, other: Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    /// Get the 4-connected neighbors (N, S, E, W), respecting bounds.
    #[must_use]
    pub fn neighbors(self, width: u32, height: u32) -> Vec<WorldPosition> {
        let mut result = Vec::with_capacity(4);
        if self.x > 0 {
            result.push(Self::new(self.x - 1, self.y));
        }
        if self.x + 1 < width {
            result.push(Self::new(self.x + 1, self.y));
        }
        if self.y > 0 {
            result.push(Self::new(self.x, self.y - 1));
        }
        if self.y + 1 < height {
            result.push(Self::new(self.x, self.y + 1));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manhattan_distance_same_point() {
        let pos = WorldPosition::new(5, 5);
        assert_eq!(pos.manhattan_distance(pos), 0);
    }

    #[test]
    fn manhattan_distance_diagonal() {
        let a = WorldPosition::new(0, 0);
        let b = WorldPosition::new(3, 4);
        assert_eq!(a.manhattan_distance(b), 7);
    }

    #[test]
    fn neighbors_center() {
        let pos = WorldPosition::new(5, 5);
        let neighbors = pos.neighbors(10, 10);
        assert_eq!(neighbors.len(), 4);
    }

    #[test]
    fn neighbors_corner() {
        let pos = WorldPosition::new(0, 0);
        let neighbors = pos.neighbors(10, 10);
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&WorldPosition::new(1, 0)));
        assert!(neighbors.contains(&WorldPosition::new(0, 1)));
    }

    #[test]
    fn neighbors_edge() {
        let pos = WorldPosition::new(9, 5);
        let neighbors = pos.neighbors(10, 10);
        assert_eq!(neighbors.len(), 3); // no east neighbor
    }
}
