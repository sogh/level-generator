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

        let candidate = Room { x, y, w, h };

        // ensure no overlap with margin of 1 tile
        if rooms.iter().any(|r| intersects_with_margin(r, &candidate, 1)) {
            continue;
        }

        carve_room(&mut grid, &candidate);
        rooms.push(candidate);
    }

    // connect rooms in order of x-coordinate of center to ensure connectivity
    rooms.sort_by_key(|r| r.center().0);
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

    let tiles: Vec<String> = grid
        .into_iter()
        .map(|row| row.into_iter().collect())
        .collect();

    Level { width, height, seed, rooms, tiles }
}

/// Whether `a`, expanded by `margin` tiles on each side, intersects `b`.
fn intersects_with_margin(a: &Room, b: &Room, margin: i32) -> bool {
    let a_expanded = Room { x: a.x - margin, y: a.y - margin, w: a.w + 2*margin, h: a.h + 2*margin };
    a_expanded.intersects(b)
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


