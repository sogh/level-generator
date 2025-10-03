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

    /// High-level generation mode
    pub mode: GenerationMode,

    /// Marble mode: channel width in tiles
    pub channel_width: u32,

    /// Marble mode: corner radius in tiles
    pub corner_radius: u32,
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
        return Level { width, height, seed, rooms: Vec::new(), tiles };
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

        let candidate = Room { x, y, w, h };

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


