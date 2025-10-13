//! Tile type definitions and logic for marble level generation.
//!
//! This module defines the various tile types that can be placed in a marble
//! level, including straight paths, curves, junctions, slopes, and obstacles.

use serde::Serialize;

/// Core tile types for marble level generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TileType {
    /// Empty space / wall / void
    Empty,
    /// Straight path segment
    Straight,
    /// 90-degree curved turn
    Curve90,
    /// T-shaped junction (3-way)
    TJunction,
    /// Y-shaped junction (3-way, smooth angles)
    YJunction,
    /// Cross-shaped junction (4-way)
    CrossJunction,
    /// Slope (connects two elevations differing by 1)
    Slope,
    /// Open platform with no walls
    OpenPlatform,
    /// Static obstacle (pillar, bumper)
    Obstacle,
    /// Merge tile (multiple inputs converge to one output)
    Merge,
    /// One-way gate (directional flow)
    OneWayGate,
    /// Loop-de-loop section
    LoopDeLoop,
    /// Half-pipe section
    HalfPipe,
    /// Launch pad / catapult
    LaunchPad,
    /// Bridge (path goes over another)
    Bridge,
    /// Tunnel (path goes under another)
    Tunnel,
}

impl TileType {
    /// Returns true if this tile type is passable (not a wall)
    pub fn is_passable(&self) -> bool {
        !matches!(self, TileType::Empty | TileType::Obstacle)
    }

    /// Returns true if this tile type has walls by default
    pub fn has_default_walls(&self) -> bool {
        matches!(
            self,
            TileType::Straight
                | TileType::Curve90
                | TileType::TJunction
                | TileType::YJunction
                | TileType::CrossJunction
                | TileType::Slope
                | TileType::Merge
                | TileType::LoopDeLoop
        )
    }

    /// Returns the ASCII character representation for this tile type
    pub fn to_ascii(&self, has_walls: bool) -> char {
        match (self, has_walls) {
            (TileType::Empty, _) => '#',
            (TileType::Obstacle, _) => 'O',
            (_, true) => '.',
            (_, false) => '·',
        }
    }
}

/// Connection directions for tile compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Direction {
    /// Returns the opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    /// Rotate direction clockwise by given number of 90° steps
    pub fn rotate(&self, steps: u8) -> Direction {
        let idx = (*self as u8 + steps) % 4;
        match idx {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            _ => unreachable!(),
        }
    }
}

/// A marble tile with type, elevation, rotation, and wall information
#[derive(Debug, Clone, Serialize)]
pub struct MarbleTile {
    /// The type of tile
    pub tile_type: TileType,
    /// Elevation level (0 = ground level, can be negative)
    pub elevation: i32,
    /// Rotation in 90° increments (0-3)
    pub rotation: u8,
    /// Whether this tile has walls
    pub has_walls: bool,
    /// Additional metadata for game engines (JSON string)
    pub metadata: String,
}

impl MarbleTile {
    /// Create a new empty tile (wall)
    pub fn empty() -> Self {
        Self {
            tile_type: TileType::Empty,
            elevation: 0,
            rotation: 0,
            has_walls: false,
            metadata: String::new(),
        }
    }

    /// Create a new tile with the given type at ground level
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            elevation: 0,
            rotation: 0,
            has_walls: tile_type.has_default_walls(),
            metadata: String::new(),
        }
    }

    /// Create a tile with specific parameters
    pub fn with_params(
        tile_type: TileType,
        elevation: i32,
        rotation: u8,
        has_walls: bool,
    ) -> Self {
        Self {
            tile_type,
            elevation,
            rotation: rotation % 4,
            has_walls,
            metadata: String::new(),
        }
    }

    /// Set metadata for this tile
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = metadata;
        self
    }

    /// Get the connections this tile has (based on type and rotation)
    pub fn connections(&self) -> Vec<Direction> {
        let base_connections = match self.tile_type {
            TileType::Empty | TileType::Obstacle => vec![],
            TileType::Straight => vec![Direction::North, Direction::South],
            TileType::Curve90 => vec![Direction::North, Direction::East],
            TileType::TJunction => vec![Direction::North, Direction::East, Direction::South],
            TileType::YJunction => vec![Direction::North, Direction::East, Direction::South],
            TileType::CrossJunction => vec![
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ],
            TileType::Slope => vec![Direction::North, Direction::South],
            TileType::OpenPlatform => vec![
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ],
            TileType::Merge => vec![Direction::North, Direction::East, Direction::West],
            TileType::OneWayGate => vec![Direction::North, Direction::South],
            TileType::LoopDeLoop => vec![Direction::North, Direction::South],
            TileType::HalfPipe => vec![Direction::North, Direction::South],
            TileType::LaunchPad => vec![Direction::North],
            TileType::Bridge => vec![Direction::North, Direction::South],
            TileType::Tunnel => vec![Direction::North, Direction::South],
        };

        // Rotate connections based on tile rotation
        base_connections
            .into_iter()
            .map(|d| d.rotate(self.rotation))
            .collect()
    }

    /// Check if this tile connects in a given direction
    pub fn connects(&self, direction: Direction) -> bool {
        self.connections().contains(&direction)
    }

    /// Check if this tile is compatible with another tile in a given direction
    pub fn compatible_with(&self, other: &MarbleTile, direction: Direction) -> bool {
        // Check if this tile connects in that direction
        if !self.connects(direction) {
            return false;
        }
        // Check if the other tile connects back
        if !other.connects(direction.opposite()) {
            return false;
        }
        // For slopes, check elevation compatibility (diff of ±1)
        match (&self.tile_type, &other.tile_type) {
            (TileType::Slope, _) | (_, TileType::Slope) => {
                (self.elevation - other.elevation).abs() <= 1
            }
            _ => self.elevation == other.elevation,
        }
    }

    /// Convert to ASCII character for legacy output
    pub fn to_ascii(&self) -> char {
        self.tile_type.to_ascii(self.has_walls)
    }
}

impl Default for MarbleTile {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_creation() {
        let tile = MarbleTile::new(TileType::Straight);
        assert_eq!(tile.tile_type, TileType::Straight);
        assert_eq!(tile.elevation, 0);
        assert_eq!(tile.rotation, 0);
        assert!(tile.has_walls);
    }

    #[test]
    fn test_tile_connections() {
        let straight = MarbleTile::new(TileType::Straight);
        assert_eq!(straight.connections().len(), 2);
        assert!(straight.connects(Direction::North));
        assert!(straight.connects(Direction::South));
        assert!(!straight.connects(Direction::East));
        assert!(!straight.connects(Direction::West));
    }

    #[test]
    fn test_tile_rotation() {
        let mut curve = MarbleTile::new(TileType::Curve90);
        assert!(curve.connects(Direction::North));
        assert!(curve.connects(Direction::East));

        curve.rotation = 1; // Rotate 90° clockwise
        assert!(curve.connects(Direction::East));
        assert!(curve.connects(Direction::South));
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
    }

    #[test]
    fn test_slope_compatibility() {
        let ground = MarbleTile::with_params(TileType::Straight, 0, 0, true);
        let slope = MarbleTile::with_params(TileType::Slope, 0, 0, true);
        let elevated = MarbleTile::with_params(TileType::Straight, 1, 0, true);

        // Slope at elevation 0 should connect to both ground (0) and elevated (1)
        assert!(slope.compatible_with(&ground, Direction::North));
        assert!(slope.compatible_with(&elevated, Direction::North));
    }
}


