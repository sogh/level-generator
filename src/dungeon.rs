//! Dungeon generation algorithm (overview)
//!
//! Stages:
//! 1) Initialize a 2D grid filled with wall tiles.
//! 2) Attempt to place rectangular rooms with random sizes within
//!    `[min_room, max_room]`, rejecting candidates that overlap existing
//!    rooms when expanded by a margin. Accepted rooms are carved as floors.
//! 3) Sort rooms by their center `x` coordinate, then connect neighboring
//!    rooms using simple L-shaped tunnels (horizontal-then-vertical or
//!    vertical-then-horizontal chosen randomly) carved as floors.
//! 4) Export the result: the grid is converted row-wise into `Vec<String>`.
//!    Walls are `'#'` and floors are `'.'`.
//!
//! The result is a connected dungeon suitable for roguelike prototypes.
//! The generator is seedable for reproducibility.
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use std::collections::VecDeque;
use crate::tiles::{MarbleTile, Direction};

/// 2D tile grid stored row-major as characters.
pub type Grid = Vec<Vec<char>>;

/// Wall tile character.
pub const TILE_WALL: char = '#';
/// Floor tile character.
pub const TILE_FLOOR: char = '.';

/// Minimum sensible map dimension to avoid degenerate results.
pub const MIN_MAP_DIM: u32 = 10;
/// Minimum sensible room dimension.
pub const MIN_ROOM_DIM: u32 = 3;

/// Axis-aligned rectangular room.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// Elevation level of this room (0 = ground level)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<i32>,
}

impl Room {
    /// Returns whether this room intersects another room.
    pub fn intersects(&self, other: &Room) -> bool {
        let left = self.x;
        let right = self.x + self.w;
        let top = self.y;
        let bottom = self.y + self.h;

		let oleft = other.x;
		let oright = other.x + other.w;
		let otop = other.y;
		let obottom = other.y + other.h;

        !(right <= oleft || oright <= left || bottom <= otop || obottom <= top)
    }

	/// Returns the integer center of the room (floor division).
	pub fn center(&self) -> (i32, i32) {
        (
            self.x + self.w / 2,
            self.y + self.h / 2,
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Level {
    /// Width of the level in tiles
    pub width: u32,
    /// Height of the level in tiles
    pub height: u32,
    /// RNG seed used to generate this level
    pub seed: u64,
    /// Rooms that were placed on the map
    pub rooms: Vec<Room>,
    /// ASCII tiles (row-major). `'#'` is wall, `'.'` is floor
    pub tiles: Vec<String>,
    /// Marble tile grid (optional, only for marble mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marble_tiles: Option<Vec<Vec<MarbleTile>>>,
    // legend: '#' = wall, '.' = floor
}

#[derive(Debug, Clone)]
pub struct GeneratorParams {
    /// Target width of the generated map (clamped to at least `MIN_MAP_DIM`)
    pub width: u32,
    /// Target height of the generated map (clamped to at least `MIN_MAP_DIM`)
    pub height: u32,
    /// Number of rooms to try to place
    pub rooms: u32,
    /// Minimum room side length (clamped to at least `MIN_ROOM_DIM`)
    pub min_room: u32,
    /// Maximum room side length (at least `min_room + 1`)
    pub max_room: u32,
    /// Optional RNG seed for reproducible results
    pub seed: Option<u64>,

    /// High-level generation mode
    pub mode: GenerationMode,

    /// Marble mode: channel width in tiles
    pub channel_width: u32,

    /// Marble mode: corner radius in tiles
    pub corner_radius: u32,

    /// Marble mode: enable elevation variation
    pub enable_elevation: bool,

    /// Marble mode: maximum elevation difference between rooms
    pub max_elevation: i32,

    /// Marble mode: enable obstacle placement in large rooms
    pub enable_obstacles: bool,

    /// Marble mode: obstacle density (0.0 to 1.0)
    pub obstacle_density: f32,
}

impl Default for GeneratorParams {
    fn default() -> Self {
        Self {
            width: 80,
            height: 25,
            rooms: 12,
            min_room: 4,
            max_room: 10,
            seed: None,
            mode: GenerationMode::Classic,
            channel_width: 2,
            corner_radius: 2,
            enable_elevation: false,
            max_elevation: 2,
            enable_obstacles: false,
            obstacle_density: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GenerationMode {
    Classic,
    Marble,
    Wfc,
}

/// Generate a new `Level` using basic room placement and corridor connectivity.
pub fn generate(params: &GeneratorParams) -> Level {
    let width = params.width.max(MIN_MAP_DIM);
    let height = params.height.max(MIN_MAP_DIM);
    let min_room = params.min_room.max(MIN_ROOM_DIM);
    let max_room = params.max_room.max(min_room + 1);

    let seed = params.seed.unwrap_or_else(|| {
        // derive a seed from thread_rng for reproducibility in output
        let mut tr = rand::rng();
        tr.random()
    });
    let mut rng = StdRng::seed_from_u64(seed);

    // Early exit for WFC mode: generate a tilemap entirely via WFC
    if matches!(params.mode, GenerationMode::Wfc) {
        let tiles = generate_wfc_tilemap(width as usize, height as usize, &mut rng);
        return Level { width, height, seed, rooms: Vec::new(), tiles, marble_tiles: None };
    }

    let mut grid: Grid = vec![vec![TILE_WALL; width as usize]; height as usize];
    let mut rooms: Vec<Room> = Vec::new();

    let attempts = (params.rooms * 10).max(100);
    for _ in 0..attempts {
        if rooms.len() as u32 >= params.rooms { break; }

        let w = rng.random_range(min_room as i32..=max_room as i32);
        let h = rng.random_range(min_room as i32..=max_room as i32);

        if w >= width as i32 - 4 || h >= height as i32 - 4 { continue; }

        let x = rng.random_range(1..=(width as i32 - w - 2));
        let y = rng.random_range(1..=(height as i32 - h - 2));

        // Assign elevation if enabled
        let elevation = if params.enable_elevation && matches!(params.mode, GenerationMode::Marble) {
            Some(rng.random_range(-params.max_elevation..=params.max_elevation))
        } else {
            None
        };

        let candidate = Room { x, y, w, h, elevation };

        // ensure no overlap with margin of 1 tile
        if rooms.iter().any(|r| intersects_with_margin(r, &candidate, 1)) {
            continue;
        }

        carve_room(&mut grid, &candidate);
        rooms.push(candidate);
    }

    // connect rooms depending on the chosen mode
    rooms.sort_by_key(|r| r.center().0);
    match params.mode {
        GenerationMode::Classic => {
            for i in 1..rooms.len() {
                let (x1, y1) = rooms[i - 1].center();
                let (x2, y2) = rooms[i].center();
                if rng.random_bool(0.5) {
                    carve_horizontal_tunnel(&mut grid, x1, x2, y1);
                    carve_vertical_tunnel(&mut grid, y1, y2, x2);
                } else {
                    carve_vertical_tunnel(&mut grid, y1, y2, x1);
                    carve_horizontal_tunnel(&mut grid, x1, x2, y2);
                }
            }
        }
        GenerationMode::Marble => {
            let w = params.channel_width.max(1) as i32;
            let r = params.corner_radius.max(0) as i32;
            for i in 1..rooms.len() {
                let (x1, y1) = rooms[i - 1].center();
                let (x2, y2) = rooms[i].center();
                if rng.random_bool(0.5) {
                    carve_wide_horizontal_with_rounded_turn(&mut grid, x1, x2, y1, w, r, true);
                    carve_wide_vertical(&mut grid, y1, y2, x2, w);
                } else {
                    carve_wide_vertical_with_rounded_turn(&mut grid, y1, y2, x1, w, r, true);
                    carve_wide_horizontal(&mut grid, x1, x2, y2, w);
                }
            }
        }
        GenerationMode::Wfc => unreachable!("handled earlier"),
    }

    let tiles: Vec<String> = grid
        .iter()
        .map(|row| row.iter().collect())
        .collect();

    // Generate marble tile grid for marble mode
    let marble_tiles = if matches!(params.mode, GenerationMode::Marble) {
        // Create elevation map for corridors if elevation is enabled
        let elevation_map = if params.enable_elevation {
            create_corridor_elevation_map(&grid, &rooms, width as usize, height as usize)
        } else {
            vec![vec![0; width as usize]; height as usize]
        };
        
        let mut tiles = grid_to_marble_tiles(&grid, &rooms, params.enable_elevation, &elevation_map);
        
        // Place obstacles in large rooms if enabled
        if params.enable_obstacles {
            place_obstacles_in_rooms(&mut tiles, &rooms, &mut rng, params.obstacle_density);
        }
        
        Some(tiles)
    } else {
        None
    };

    Level { width, height, seed, rooms, tiles, marble_tiles }
}

/// Whether `a`, expanded by `margin` tiles on each side, intersects `b`.
fn intersects_with_margin(a: &Room, b: &Room, margin: i32) -> bool {
    let a_expanded = Room { 
        x: a.x - margin, 
        y: a.y - margin, 
        w: a.w + 2*margin, 
        h: a.h + 2*margin,
        elevation: a.elevation,
    };
    a_expanded.intersects(b)
}

/// Create elevation map for corridors between rooms with different elevations
/// This creates smooth transitions with slope tiles where elevation changes
fn create_corridor_elevation_map(
    grid: &Grid,
    rooms: &[Room],
    width: usize,
    height: usize,
) -> Vec<Vec<i32>> {
    use std::collections::{VecDeque, HashMap};
    
    let mut elevation_map = vec![vec![0i32; width]; height];
    let mut distance_map = vec![vec![i32::MAX; width]; height];
    
    // First, assign elevations and distances to all room tiles
    for room in rooms {
        let room_elev = room.elevation.unwrap_or(0);
        for y in room.y..room.y + room.h {
            for x in room.x..room.x + room.w {
                if y >= 0 && (y as usize) < height && x >= 0 && (x as usize) < width {
                    elevation_map[y as usize][x as usize] = room_elev;
                    distance_map[y as usize][x as usize] = 0; // Room tiles have distance 0
                }
            }
        }
    }
    
    // Multi-source BFS to find nearest room for each corridor tile
    let mut queue: VecDeque<(usize, usize, i32, i32)> = VecDeque::new(); // (x, y, distance, elevation)
    
    // Start from all room tiles
    for room in rooms {
        let room_elev = room.elevation.unwrap_or(0);
        for y in room.y..room.y + room.h {
            for x in room.x..room.x + room.w {
                if y >= 0 && (y as usize) < height && x >= 0 && (x as usize) < width {
                    if grid[y as usize][x as usize] == TILE_FLOOR {
                        queue.push_back((x as usize, y as usize, 0, room_elev));
                    }
                }
            }
        }
    }
    
    // BFS to propagate elevations from rooms to corridors
    while let Some((x, y, dist, elev)) = queue.pop_front() {
        // Skip if we've already found a shorter path to this tile
        if dist > distance_map[y][x] {
            continue;
        }
        
        for (dx, dy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            
            if ny >= 0 && ny < height as i32 && nx >= 0 && nx < width as i32 {
                let nux = nx as usize;
                let nuy = ny as usize;
                
                if grid[nuy][nux] == TILE_FLOOR {
                    let new_dist = dist + 1;
                    if new_dist < distance_map[nuy][nux] {
                        distance_map[nuy][nux] = new_dist;
                        elevation_map[nuy][nux] = elev;
                        queue.push_back((nux, nuy, new_dist, elev));
                    }
                }
            }
        }
    }
    
    // Second pass: smooth out large elevation jumps iteratively
    // Keep smoothing until no tile has a neighbor with elevation difference > 1
    let max_iterations = 50;
    for _iter in 0..max_iterations {
        let mut changes_made = false;
        let mut new_elevations: HashMap<(usize, usize), i32> = HashMap::new();
        
        for y in 0..height {
            for x in 0..width {
                if grid[y][x] != TILE_FLOOR {
                    continue;
                }
                
                let current_elev = elevation_map[y][x];
                let current_dist = distance_map[y][x];
                
                // Check all neighbors for large jumps
                for (dx, dy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    
                    if ny >= 0 && (ny as usize) < height && nx >= 0 && (nx as usize) < width {
                        if grid[ny as usize][nx as usize] == TILE_FLOOR {
                            let neighbor_elev = elevation_map[ny as usize][nx as usize];
                            let neighbor_dist = distance_map[ny as usize][nx as usize];
                            let diff = neighbor_elev - current_elev;
                            
                            // If there's a jump > 1, we need to insert intermediate elevations
                            if diff.abs() > 1 {
                                // Adjust this tile if it's farther from a room OR same distance
                                if current_dist >= neighbor_dist {
                                    let dir = diff.signum();
                                    let new_elev = current_elev + dir;
                                    // Only update if we haven't already scheduled a change
                                    if !new_elevations.contains_key(&(x, y)) {
                                        new_elevations.insert((x, y), new_elev);
                                        changes_made = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Apply all changes
        for ((x, y), new_elev) in &new_elevations {
            elevation_map[*y][*x] = *new_elev;
        }
        
        if !changes_made {
            break; // No more large jumps, we're done
        }
    }
    
    elevation_map
}

/// Place obstacles in large rooms
fn place_obstacles_in_rooms(
    marble_grid: &mut [Vec<MarbleTile>],
    rooms: &[Room],
    rng: &mut StdRng,
    density: f32,
) {
    use crate::tiles::TileType;
    
    let height = marble_grid.len();
    let width = if height > 0 { marble_grid[0].len() } else { 0 };
    
    for room in rooms {
        let room_area = (room.w * room.h) as f32;
        
        // Only place obstacles in rooms larger than 30 tiles
        if room_area < 30.0 {
            continue;
        }
        
        // Number of obstacles based on room size and density
        let num_obstacles = ((room_area * density * 0.1) as i32).max(1);
        
        for _ in 0..num_obstacles {
            // Try to place obstacle in a random floor position within the room
            for _ in 0..20 {  // Max 20 attempts per obstacle
                let ox = rng.random_range(room.x + 1..room.x + room.w - 1);
                let oy = rng.random_range(room.y + 1..room.y + room.h - 1);
                
                if oy >= 0 && (oy as usize) < height && ox >= 0 && (ox as usize) < width {
                    let tile = &marble_grid[oy as usize][ox as usize];
                    
                    // Only place obstacle on passable tiles that aren't already obstacles
                    if tile.tile_type.is_passable() && tile.tile_type != TileType::Obstacle {
                        let elevation = tile.elevation;
                        marble_grid[oy as usize][ox as usize] = MarbleTile::with_params(
                            TileType::Obstacle,
                            elevation,
                            0,
                            false,
                        );
                        break;
                    }
                }
            }
        }
    }
}

/// Convert a character grid to a marble tile grid with intelligent tile type detection
fn grid_to_marble_tiles(
    grid: &Grid, 
    _rooms: &[Room], 
    enable_elevation: bool,
    elevation_map: &[Vec<i32>]
) -> Vec<Vec<MarbleTile>> {
    use crate::tiles::TileType;
    
    let height = grid.len();
    let width = if height > 0 { grid[0].len() } else { 0 };
    
    let mut marble_grid = vec![vec![MarbleTile::empty(); width]; height];
    
    // Helper to check if a position is a floor tile
    let is_floor = |x: i32, y: i32| -> bool {
        if y >= 0 && (y as usize) < height && x >= 0 && (x as usize) < width {
            grid[y as usize][x as usize] == TILE_FLOOR
        } else {
            false
        }
    };
    
    // Get elevation from the map
    let get_elevation = |x: i32, y: i32| -> i32 {
        if y >= 0 && (y as usize) < height && x >= 0 && (x as usize) < width {
            elevation_map[y as usize][x as usize]
        } else {
            0
        }
    };
    
    // First pass: detect tile types based on neighbors
    for y in 0..height {
        for x in 0..width {
            if grid[y][x] != TILE_FLOOR {
                continue;
            }
            
            let ix = x as i32;
            let iy = y as i32;
            
            // Check all four directions
            let north = is_floor(ix, iy - 1);
            let south = is_floor(ix, iy + 1);
            let east = is_floor(ix + 1, iy);
            let west = is_floor(ix - 1, iy);
            
            let connection_count = [north, south, east, west].iter().filter(|&&b| b).count();
            
            // Determine base elevation for this tile from the elevation map
            let base_elevation = get_elevation(ix, iy);
            
            let (tile_type, rotation) = match connection_count {
                0 | 1 => (TileType::OpenPlatform, 0), // Isolated or dead-end
                2 => {
                    // Straight or curve
                    if (north && south) || (east && west) {
                        // Straight path
                        let rot = if north && south { 0 } else { 1 };
                        (TileType::Straight, rot)
                    } else {
                        // 90-degree curve
                        let rot = if north && east {
                            0
                        } else if east && south {
                            1
                        } else if south && west {
                            2
                        } else {
                            3
                        };
                        (TileType::Curve90, rot)
                    }
                }
                3 => {
                    // T-junction
                    let rot = if !south {
                        0
                    } else if !west {
                        1
                    } else if !north {
                        2
                    } else {
                        3
                    };
                    (TileType::TJunction, rot)
                }
                4 => (TileType::CrossJunction, 0),
                _ => (TileType::Straight, 0),
            };
            
            marble_grid[y][x] = MarbleTile::with_params(tile_type, base_elevation, rotation, true);
        }
    }
    
    // Second pass: place advanced tiles in appropriate locations (before slope conversion)
    place_advanced_tiles(&mut marble_grid, grid, enable_elevation);
    
    // Third pass: detect and place slope tiles where elevation changes
    if enable_elevation {
        for y in 0..height {
            for x in 0..width {
                let tile = &marble_grid[y][x];
                if tile.tile_type == TileType::Empty {
                    continue;
                }
                
                let ix = x as i32;
                let iy = y as i32;
                let current_elev = tile.elevation;
                
                // Only convert simple tiles to slopes (not junctions, curves, or advanced tiles)
                if !matches!(tile.tile_type, TileType::Straight | TileType::OpenPlatform | TileType::CrossJunction) {
                    continue;
                }
                
                // Check each direction for elevation changes (±1)
                // North/South (vertical)
                if (is_floor(ix, iy - 1) && (get_elevation(ix, iy - 1) - current_elev).abs() == 1) ||
                   (is_floor(ix, iy + 1) && (get_elevation(ix, iy + 1) - current_elev).abs() == 1) {
                    marble_grid[y][x] = MarbleTile::with_params(
                        TileType::Slope,
                        current_elev,
                        0, // Vertical orientation
                        true
                    );
                    continue;
                }
                // East/West (horizontal)
                if (is_floor(ix + 1, iy) && (get_elevation(ix + 1, iy) - current_elev).abs() == 1) ||
                   (is_floor(ix - 1, iy) && (get_elevation(ix - 1, iy) - current_elev).abs() == 1) {
                    marble_grid[y][x] = MarbleTile::with_params(
                        TileType::Slope,
                        current_elev,
                        1, // Horizontal orientation
                        true
                    );
                }
            }
        }
    }
    
    marble_grid
}

/// Place advanced tiles in appropriate locations based on context
fn place_advanced_tiles(
    marble_grid: &mut Vec<Vec<MarbleTile>>,
    grid: &Grid,
    enable_elevation: bool,
) {
    use crate::tiles::TileType;
    
    let height = marble_grid.len();
    let width = if height > 0 { marble_grid[0].len() } else { 0 };
    
    // Helper to check if a position is a floor tile
    let is_floor = |x: i32, y: i32| -> bool {
        if y >= 0 && (y as usize) < height && x >= 0 && (x as usize) < width {
            grid[y as usize][x as usize] == TILE_FLOOR
        } else {
            false
        }
    };
    
    // Place Y-junctions where we have smooth 3-way connections
    for y in 1..height-1 {
        for x in 1..width-1 {
            let tile = &marble_grid[y][x];
            if tile.tile_type != TileType::TJunction {
                continue;
            }
            
            let ix = x as i32;
            let iy = y as i32;
            
            // Check if this T-junction could be a smooth Y-junction
            // Look for diagonal connections that suggest smooth curves
            let north = is_floor(ix, iy - 1);
            let south = is_floor(ix, iy + 1);
            let east = is_floor(ix + 1, iy);
            let west = is_floor(ix - 1, iy);
            
            // Check for diagonal patterns that suggest Y-junction
            let has_diagonal = (north && east && is_floor(ix + 1, iy - 1)) ||
                              (east && south && is_floor(ix + 1, iy + 1)) ||
                              (south && west && is_floor(ix - 1, iy + 1)) ||
                              (west && north && is_floor(ix - 1, iy - 1));
            
            if has_diagonal {
                marble_grid[y][x] = MarbleTile::with_params(
                    TileType::YJunction,
                    tile.elevation,
                    tile.rotation,
                    true
                );
            }
        }
    }
    
    // Place merge tiles where multiple paths converge to a single output
    for y in 1..height-1 {
        for x in 1..width-1 {
            let tile = &marble_grid[y][x];
            if tile.tile_type != TileType::CrossJunction {
                continue;
            }
            
            let ix = x as i32;
            let iy = y as i32;
            
            // Check if this cross junction has a clear "output" direction
            // (one direction with more connections downstream)
            let north_connections = count_connections_downstream(marble_grid, grid, ix, iy - 1, Direction::North);
            let south_connections = count_connections_downstream(marble_grid, grid, ix, iy + 1, Direction::South);
            let east_connections = count_connections_downstream(marble_grid, grid, ix + 1, iy, Direction::East);
            let west_connections = count_connections_downstream(marble_grid, grid, ix - 1, iy, Direction::West);
            
            let connections = [north_connections, south_connections, east_connections, west_connections];
            let max_connections = connections.iter().max().unwrap_or(&0);
            
            // If one direction has significantly more connections, it's likely a merge
            if *max_connections >= 3 && connections.iter().filter(|&&c| c > 0).count() >= 3 {
                // Determine the output direction (the one with most connections)
                let output_dir = if north_connections == *max_connections { 0 }
                                else if east_connections == *max_connections { 1 }
                                else if south_connections == *max_connections { 2 }
                                else { 3 };
                
                marble_grid[y][x] = MarbleTile::with_params(
                    TileType::Merge,
                    tile.elevation,
                    output_dir,
                    true
                );
            }
        }
    }
    
    // Place one-way gates in narrow passages (relaxed conditions)
    for y in 1..height-1 {
        for x in 1..width-1 {
            let tile = &marble_grid[y][x];
            if tile.tile_type != TileType::Straight {
                continue;
            }
            
            let ix = x as i32;
            let iy = y as i32;
            
            // Check if this is a narrow passage (straight line with walls on sides)
            // Relaxed: only need walls on one side, not both
            let is_narrow_passage = match tile.rotation {
                0 | 2 => { // Vertical passage
                    (!is_floor(ix - 1, iy) || !is_floor(ix + 1, iy)) &&
                    is_floor(ix, iy - 1) && is_floor(ix, iy + 1)
                },
                1 | 3 => { // Horizontal passage
                    (!is_floor(ix, iy - 1) || !is_floor(ix, iy + 1)) &&
                    is_floor(ix - 1, iy) && is_floor(ix + 1, iy)
                },
                _ => false,
            };
            
            if is_narrow_passage {
                marble_grid[y][x] = MarbleTile::with_params(
                    TileType::OneWayGate,
                    tile.elevation,
                    tile.rotation,
                    true
                );
            }
        }
    }
    
    // Place loop-de-loops where we have elevation changes of +2 or more
    if enable_elevation {
        for y in 1..height-1 {
            for x in 1..width-1 {
                let tile = &marble_grid[y][x];
                if tile.tile_type != TileType::Straight {
                    continue;
                }
                
                let ix = x as i32;
                let iy = y as i32;
                let current_elev = tile.elevation;
                
                // Check for large elevation changes that could support a loop
                let has_large_elevation_change = 
                    (is_floor(ix, iy - 1) && (get_elevation(marble_grid, ix, iy - 1) - current_elev).abs() >= 2) ||
                    (is_floor(ix, iy + 1) && (get_elevation(marble_grid, ix, iy + 1) - current_elev).abs() >= 2) ||
                    (is_floor(ix + 1, iy) && (get_elevation(marble_grid, ix + 1, iy) - current_elev).abs() >= 2) ||
                    (is_floor(ix - 1, iy) && (get_elevation(marble_grid, ix - 1, iy) - current_elev).abs() >= 2);
                
                if has_large_elevation_change {
                    marble_grid[y][x] = MarbleTile::with_params(
                        TileType::LoopDeLoop,
                        current_elev,
                        tile.rotation,
                        true
                    );
                }
            }
        }
    }
    
    // Place half-pipes in curved sections with elevation changes
    if enable_elevation {
        for y in 1..height-1 {
            for x in 1..width-1 {
                let tile = &marble_grid[y][x];
                if tile.tile_type != TileType::Curve90 {
                    continue;
                }
                
                let ix = x as i32;
                let iy = y as i32;
                let current_elev = tile.elevation;
                
                // Check if this curve has elevation changes
                let has_elevation_change = 
                    (is_floor(ix, iy - 1) && (get_elevation(marble_grid, ix, iy - 1) - current_elev).abs() == 1) ||
                    (is_floor(ix, iy + 1) && (get_elevation(marble_grid, ix, iy + 1) - current_elev).abs() == 1) ||
                    (is_floor(ix + 1, iy) && (get_elevation(marble_grid, ix + 1, iy) - current_elev).abs() == 1) ||
                    (is_floor(ix - 1, iy) && (get_elevation(marble_grid, ix - 1, iy) - current_elev).abs() == 1);
                
                if has_elevation_change {
                    marble_grid[y][x] = MarbleTile::with_params(
                        TileType::HalfPipe,
                        current_elev,
                        tile.rotation,
                        true
                    );
                }
            }
        }
    }
    
    // Place launch pads at the start of straight sections (relaxed conditions)
    for y in 1..height-1 {
        for x in 1..width-1 {
            let tile = &marble_grid[y][x];
            if tile.tile_type != TileType::Straight {
                continue;
            }
            
            let ix = x as i32;
            let iy = y as i32;
            
            // Check if this is the start of a straight section (relaxed: just need continuation)
            let is_launch_pad = match tile.rotation {
                0 | 2 => { // Vertical
                    !is_floor(ix, iy - 1) && is_floor(ix, iy + 1)
                },
                1 | 3 => { // Horizontal
                    !is_floor(ix - 1, iy) && is_floor(ix + 1, iy)
                },
                _ => false,
            };
            
            if is_launch_pad {
                marble_grid[y][x] = MarbleTile::with_params(
                    TileType::LaunchPad,
                    tile.elevation,
                    tile.rotation,
                    true
                );
            }
        }
    }
}

/// Helper function to count connections downstream from a position
fn count_connections_downstream(
    marble_grid: &Vec<Vec<MarbleTile>>,
    grid: &Grid,
    start_x: i32,
    start_y: i32,
    direction: Direction,
) -> usize {
    use crate::tiles::TileType;
    if start_y < 0 || (start_y as usize) >= marble_grid.len() ||
       start_x < 0 || (start_x as usize) >= marble_grid[0].len() {
        return 0;
    }
    
    let mut count = 0;
    let mut x = start_x;
    let mut y = start_y;
    
    // Follow the path in the given direction
    for _ in 0..10 { // Limit to prevent infinite loops
        let (dx, dy) = match direction {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        };
        
        x += dx;
        y += dy;
        
        if y < 0 || (y as usize) >= marble_grid.len() ||
           x < 0 || (x as usize) >= marble_grid[0].len() {
            break;
        }
        
        if grid[y as usize][x as usize] != TILE_FLOOR {
            break;
        }
        
        count += 1;
        
        // Stop if we hit a junction or dead end
        let tile = &marble_grid[y as usize][x as usize];
        if tile.tile_type == TileType::TJunction || 
           tile.tile_type == TileType::CrossJunction ||
           tile.tile_type == TileType::YJunction {
            break;
        }
    }
    
    count
}

/// Helper function to get elevation from marble grid
fn get_elevation(marble_grid: &Vec<Vec<MarbleTile>>, x: i32, y: i32) -> i32 {
    if y >= 0 && (y as usize) < marble_grid.len() &&
       x >= 0 && (x as usize) < marble_grid[0].len() {
        marble_grid[y as usize][x as usize].elevation
    } else {
        0
    }
}

/// Fill the rectangle defined by `room` with floor tiles.
fn carve_room(grid: &mut [Vec<char>], room: &Room) {
    for y in room.y..room.y + room.h {
        for x in room.x..room.x + room.w {
            set_floor(grid, x, y);
        }
    }
}

/// Carve a horizontal tunnel from `x1..=x2` at row `y`.
fn carve_horizontal_tunnel(grid: &mut [Vec<char>], x1: i32, x2: i32, y: i32) {
    let (start, end) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
    for x in start..=end {
        set_floor(grid, x, y);
    }
}

/// Carve a vertical tunnel from `y1..=y2` at column `x`.
fn carve_vertical_tunnel(grid: &mut [Vec<char>], y1: i32, y2: i32, x: i32) {
    let (start, end) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
    for y in start..=end {
        set_floor(grid, x, y);
    }
}

/// Safely set the tile at `(x, y)` to floor if within bounds.
fn set_floor(grid: &mut [Vec<char>], x: i32, y: i32) {
    if y >= 0 && (y as usize) < grid.len() {
        let row = &mut grid[y as usize];
        if x >= 0 && (x as usize) < row.len() {
            row[x as usize] = TILE_FLOOR;
        }
    }
}

// ========================= WFC IMPLEMENTATION ========================= //

#[derive(Clone, Copy)]
struct WfcTile {
    ch: char,
    // edges: [up, right, down, left]; true = connection, false = no connection
    edges: [bool; 4],
}

fn wfc_tileset() -> Vec<WfcTile> {
    vec![
        WfcTile { ch: ' ', edges: [false, false, false, false] },
        WfcTile { ch: '─', edges: [false, true,  false, true  ] },
        WfcTile { ch: '│', edges: [true,  false, true,  false ] },
        WfcTile { ch: '┌', edges: [false, true,  true,  false ] },
        WfcTile { ch: '┐', edges: [false, false, true,  true  ] },
        WfcTile { ch: '└', edges: [true,  true,  false, false ] },
        WfcTile { ch: '┘', edges: [true,  false, false, true  ] },
        WfcTile { ch: '├', edges: [true,  true,  true,  false ] },
        WfcTile { ch: '┤', edges: [true,  false, true,  true  ] },
        WfcTile { ch: '┬', edges: [false, true,  true,  true  ] },
        WfcTile { ch: '┴', edges: [true,  true,  false, true  ] },
        WfcTile { ch: '┼', edges: [true,  true,  true,  true  ] },
    ]
}

fn opposite(dir: usize) -> usize { (dir + 2) % 4 }

fn generate_wfc_tilemap(width: usize, height: usize, rng: &mut StdRng) -> Vec<String> {
    let tiles = wfc_tileset();
    let num_tiles = tiles.len();
    let all_mask: u32 = if num_tiles >= 32 { u32::MAX } else { (1u32 << num_tiles) - 1 };

    // Precompute compatibility: compat[t][dir] = bitmask of neighbor tiles allowed
    let mut compat: Vec<[u32; 4]> = vec![[0; 4]; num_tiles];
    for (i, t) in tiles.iter().enumerate() {
        for dir in 0..4 {
            let mut mask = 0u32;
            for (j, n) in tiles.iter().enumerate() {
                if t.edges[dir] == n.edges[opposite(dir)] {
                    mask |= 1u32 << j;
                }
            }
            compat[i][dir] = mask;
        }
    }

    let idx = |x: usize, y: usize| -> usize { y * width + x };

    let mut attempts = 0;
    while attempts < 10 {
        attempts += 1;
        let mut domains: Vec<u32> = vec![all_mask; width * height];

        // Border constraints: disallow tiles whose connections go off-grid
        for y in 0..height {
            for x in 0..width {
                let mut mask = all_mask;
                if y == 0 {
                    // up must be false
                    mask &= allowed_without_connection(&tiles, 0);
                }
                if x + 1 == width {
                    // right must be false
                    mask &= allowed_without_connection(&tiles, 1);
                }
                if y + 1 == height {
                    // down must be false
                    mask &= allowed_without_connection(&tiles, 2);
                }
                if x == 0 {
                    // left must be false
                    mask &= allowed_without_connection(&tiles, 3);
                }
                domains[idx(x, y)] &= mask;
            }
        }

        let mut queue: VecDeque<usize> = VecDeque::new();

        loop {
            // Pick cell with lowest entropy > 1
            let mut best_i = None;
            let mut best_count = usize::MAX;
            for i in 0..domains.len() {
                let d = domains[i];
                let c = d.count_ones() as usize;
                if c > 1 && c < best_count {
                    best_count = c;
                    best_i = Some(i);
                }
            }

            if let Some(i) = best_i {
                // Collapse: choose random tile from domain
                let d = domains[i];
                if d == 0 { break; }
                let mut options: Vec<usize> = Vec::new();
                for t in 0..num_tiles { if (d & (1u32 << t)) != 0 { options.push(t); } }
                let choice = options[rng.random_range(0..options.len())];
                domains[i] = 1u32 << choice;
                queue.push_back(i);
            } else {
                // No cells with entropy >1: finished or contradiction
                if domains.iter().any(|&d| d == 0) {
                    break;
                }
                // Success
                let mut out: Vec<String> = Vec::with_capacity(height);
                for y in 0..height {
                    let mut row = String::with_capacity(width);
                    for x in 0..width {
                        let d = domains[idx(x, y)];
                        let tile_id = (0..num_tiles).find(|t| (d & (1u32 << t)) != 0).unwrap_or(0);
                        row.push(tiles[tile_id].ch);
                    }
                    out.push(row);
                }
                return out;
            }

            // Propagate constraints
            while let Some(i0) = queue.pop_front() {
                let x0 = i0 % width;
                let y0 = i0 / width;
                let d0 = domains[i0];
                if d0 == 0 { break; }

                for dir in 0..4 {
                    let nx = match dir { 1 => x0 + 1, 3 => x0.wrapping_sub(1), _ => x0 };
                    let ny = match dir { 0 => y0.wrapping_sub(1), 2 => y0 + 1, _ => y0 };
                    if nx >= width || ny >= height { continue; }
                    let ni = idx(nx, ny);

                    // Allowed neighbor set from current domain
                    let mut allowed = 0u32;
                    for t in 0..num_tiles { if (d0 & (1u32 << t)) != 0 { allowed |= compat[t][dir]; } }

                    let before = domains[ni];
                    let after = before & allowed;
                    if after != before {
                        domains[ni] = after;
                        // Early contradiction; continue to allow restart
                        if after == 0 { break; }
                        queue.push_back(ni);
                    }
                }
            }
            // If any domain zeroed, restart
            if domains.iter().any(|&d| d == 0) { break; }
        }
        // restart on failure
    }

    // Fallback: empty grid if all attempts failed
    vec![" ".repeat(width); height]
}

fn allowed_without_connection(tiles: &[WfcTile], dir: usize) -> u32 {
    let mut mask = 0u32;
    for (i, t) in tiles.iter().enumerate() {
        if !t.edges[dir] { mask |= 1u32 << i; }
    }
    mask
}

/// Carve a horizontal channel of width `width_tiles` centered on `y`.
fn carve_wide_horizontal(grid: &mut [Vec<char>], x1: i32, x2: i32, y: i32, width_tiles: i32) {
    let (start, end) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
    let half = width_tiles / 2;
    for x in start..=end {
        for dy in -half..=half {
            set_floor(grid, x, y + dy);
        }
    }
}

/// Carve a vertical channel of width `width_tiles` centered on `x`.
fn carve_wide_vertical(grid: &mut [Vec<char>], y1: i32, y2: i32, x: i32, width_tiles: i32) {
    let (start, end) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
    let half = width_tiles / 2;
    for y in start..=end {
        for dx in -half..=half {
            set_floor(grid, x + dx, y);
        }
    }
}

/// Carve a rounded quarter-circle at the L-turn from horizontal to vertical.
/// If `turn_right` is true, the horizontal moves to the right before turning; otherwise to the left.
fn carve_wide_horizontal_with_rounded_turn(
    grid: &mut [Vec<char>], x1: i32, x2: i32, y: i32, width_tiles: i32, radius: i32, turn_down: bool,
) {
    carve_wide_horizontal(grid, x1, x2, y, width_tiles);
    // Draw a quarter disk at the corner (center near (x2, y))
    carve_quarter_disk(grid, x2, y, radius.max(width_tiles / 2), width_tiles, if turn_down { Quadrant::Down } else { Quadrant::Up });
}

/// Carve a rounded quarter-circle at the L-turn from vertical to horizontal.
fn carve_wide_vertical_with_rounded_turn(
    grid: &mut [Vec<char>], y1: i32, y2: i32, x: i32, width_tiles: i32, radius: i32, turn_right: bool,
) {
    carve_wide_vertical(grid, y1, y2, x, width_tiles);
    carve_quarter_disk(grid, x, y2, radius.max(width_tiles / 2), width_tiles, if turn_right { Quadrant::Right } else { Quadrant::Left });
}

#[derive(Clone, Copy)]
enum Quadrant { Up, Down, Left, Right }

/// Approximate a quarter disk for rounding corners, thickened by channel width.
fn carve_quarter_disk(grid: &mut [Vec<char>], cx: i32, cy: i32, radius: i32, width_tiles: i32, quad: Quadrant) {
    if radius <= 0 { return; }
    let inner = (radius - width_tiles / 2).max(0);
    let outer = radius + width_tiles / 2;
    match quad {
        Quadrant::Down => {
            for dy in 0..=outer {
                for dx in -outer..=outer {
                    let d2 = dx*dx + dy*dy;
                    if d2 <= outer*outer && d2 >= inner*inner {
                        set_floor(grid, cx + dx, cy + dy);
                    }
                }
            }
        }
        Quadrant::Up => {
            for dy in -outer..=0 {
                for dx in -outer..=outer {
                    let d2 = dx*dx + dy*dy;
                    if d2 <= outer*outer && d2 >= inner*inner {
                        set_floor(grid, cx + dx, cy + dy);
                    }
                }
            }
        }
        Quadrant::Right => {
            for dx in 0..=outer {
                for dy in -outer..=outer {
                    let d2 = dx*dx + dy*dy;
                    if d2 <= outer*outer && d2 >= inner*inner {
                        set_floor(grid, cx + dx, cy + dy);
                    }
                }
            }
        }
        Quadrant::Left => {
            for dx in -outer..=0 {
                for dy in -outer..=outer {
                    let d2 = dx*dx + dy*dy;
                    if d2 <= outer*outer && d2 >= inner*inner {
                        set_floor(grid, cx + dx, cy + dy);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params_base() -> GeneratorParams {
        GeneratorParams {
            width: 60,
            height: 25,
            rooms: 10,
            min_room: 4,
            max_room: 10,
            seed: Some(42),
            mode: GenerationMode::Classic,
            channel_width: 2,
            corner_radius: 2,
            enable_elevation: false,
            max_elevation: 2,
            enable_obstacles: false,
            obstacle_density: 0.3,
        }
    }

    fn count_chars(tiles: &[String], target: char) -> usize {
        tiles.iter().map(|row| row.chars().filter(|&c| c == target).count()).sum()
    }

    fn all_chars_in_set(tiles: &[String], allowed: &[char]) -> bool {
        let mut ok = true;
        for row in tiles {
            for ch in row.chars() {
                if !allowed.contains(&ch) { ok = false; break; }
            }
        }
        ok
    }

    #[test]
    fn classic_deterministic_with_seed() {
        let mut p = params_base();
        p.mode = GenerationMode::Classic;
        p.seed = Some(123);
        let a = generate(&p);
        let b = generate(&p);
        assert_eq!(a.tiles, b.tiles);
        assert!(all_chars_in_set(&a.tiles, &[TILE_WALL, TILE_FLOOR]));
    }

    #[test]
    fn marble_deterministic_with_seed() {
        let mut p = params_base();
        p.mode = GenerationMode::Marble;
        p.channel_width = 3;
        p.corner_radius = 3;
        p.seed = Some(999);
        let a = generate(&p);
        let b = generate(&p);
        assert_eq!(a.tiles, b.tiles);
        assert!(all_chars_in_set(&a.tiles, &[TILE_WALL, TILE_FLOOR]));
    }

    fn parse_grid(tiles: &[String]) -> Vec<Vec<char>> {
        tiles.iter().map(|r| r.chars().collect::<Vec<char>>()).collect::<Vec<_>>()
    }

    #[test]
    fn classic_connectivity_of_floors() {
        let mut p = params_base();
        p.mode = GenerationMode::Classic;
        p.seed = Some(7);
        let lvl = generate(&p);
        let grid = parse_grid(&lvl.tiles);
        let h = grid.len();
        let w = grid[0].len();
        // Find first floor
        let mut start: Option<(usize, usize)> = None;
        for y in 0..h {
            for x in 0..w {
                if grid[y][x] == TILE_FLOOR { start = Some((x, y)); break; }
            }
            if start.is_some() { break; }
        }
        if start.is_none() { return; }
        let (sx, sy) = start.unwrap();
        let mut visited = vec![vec![false; w]; h];
        let mut q = std::collections::VecDeque::new();
        visited[sy][sx] = true;
        q.push_back((sx, sy));
        let mut floors_seen = 1usize;
        while let Some((x, y)) = q.pop_front() {
            let dirs = [(1,0),(-1,0),(0,1),(0,-1)];
            for (dx, dy) in dirs {
                let nx = x as i32 + dx; let ny = y as i32 + dy;
                if nx>=0 && ny>=0 && (ny as usize) < h && (nx as usize) < w {
                    let ux = nx as usize; let uy = ny as usize;
                    if !visited[uy][ux] && grid[uy][ux] == TILE_FLOOR {
                        visited[uy][ux] = true; floors_seen += 1; q.push_back((ux, uy));
                    }
                }
            }
        }
        let total_floors = count_chars(&lvl.tiles, TILE_FLOOR);
        assert_eq!(floors_seen, total_floors);
    }

    #[test]
    fn wfc_deterministic_and_valid_adjacency() {
        let mut p = params_base();
        p.mode = GenerationMode::Wfc;
        p.width = 20; p.height = 10;
        p.seed = Some(2024);
        let a = generate(&p);
        let b = generate(&p);
        assert_eq!(a.tiles, b.tiles);

        // Build lookup from char to edges
        let ts = wfc_tileset();
        let mut edges_by_char: std::collections::HashMap<char, [bool;4]> = std::collections::HashMap::new();
        for t in &ts { edges_by_char.insert(t.ch, t.edges); }

        // Validate adjacency
        let h = a.tiles.len();
        let w = a.tiles[0].chars().count();
        for y in 0..h {
            let row: Vec<char> = a.tiles[y].chars().collect();
            for x in 0..w {
                let ch = row[x];
                let e = *edges_by_char.get(&ch).unwrap_or(&[false,false,false,false]);
                // up
                if y == 0 { assert!(!e[0]); } else {
                    let upch = a.tiles[y-1].chars().nth(x).unwrap();
                    let ue = *edges_by_char.get(&upch).unwrap_or(&[false,false,false,false]);
                    assert_eq!(e[0], ue[2]);
                }
                // right
                if x + 1 == w { assert!(!e[1]); } else {
                    let rch = a.tiles[y].chars().nth(x+1).unwrap();
                    let re = *edges_by_char.get(&rch).unwrap_or(&[false,false,false,false]);
                    assert_eq!(e[1], re[3]);
                }
                // down
                if y + 1 == h { assert!(!e[2]); } else {
                    let dch = a.tiles[y+1].chars().nth(x).unwrap();
                    let de = *edges_by_char.get(&dch).unwrap_or(&[false,false,false,false]);
                    assert_eq!(e[2], de[0]);
                }
                // left
                if x == 0 { assert!(!e[3]); } else {
                    let lch = a.tiles[y].chars().nth(x-1).unwrap();
                    let le = *edges_by_char.get(&lch).unwrap_or(&[false,false,false,false]);
                    assert_eq!(e[3], le[1]);
                }
            }
        }
    }
}
