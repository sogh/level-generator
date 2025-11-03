# level-generator

A procedural level generator written in Rust with multiple generation modes:
- **Classic**: Nethack-style dungeons with rectangular rooms and L-shaped tunnels
- **Marble**: Wide, rounded channels with elevation changes, slopes, obstacles, and junctions - designed for marble rolling games
- **WFC**: Wave Function Collapse algorithm for pipe-based mazes

Exports levels as ASCII, JSON (with detailed tile metadata), and isometric HTML/SVG visualizations.

## Prerequisites

- **Rust 1.70+** and Cargo (install from [rustup.rs](https://rustup.rs/))
- **Git** (for cloning the repository)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/sogh/level-generator.git
cd level-generator

# Build the project
cargo build --release

# Generate a simple dungeon
cargo run -- --width 40 --height 20 --rooms 8

# Generate a marble track with visualization
cargo run -- --mode marble --width 60 --height 30 --rooms 10 --html-path level.html
```

## Install

```bash
# Build the project
cargo build --release

# Or install globally (optional)
cargo install --path .
```

## Overview

- Rooms are placed randomly (sizes in `[min_room, max_room]`) and rejected if overlapping an existing room expanded by a margin.
- Rooms are sorted by center `x` and connected pairwise with L-shaped tunnels.
- Tiles: `#` is wall, `.` is floor.
- Generation is reproducible via `--seed`.

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
level-generator = "0.1.0"
```

### Basic Example

```rust
use level_generator::{generate, GeneratorParams, GenerationMode};

let params = GeneratorParams {
    width: 60,
    height: 30,
    rooms: 10,
    mode: GenerationMode::Classic,
    seed: Some(42),
    ..Default::default()
};

let level = generate(&params);
```

### Marble Track with Elevation

```rust
use level_generator::{generate, GeneratorParams, GenerationMode};

let params = GeneratorParams {
    width: 80,
    height: 40,
    rooms: 12,
    mode: GenerationMode::Marble,
    enable_elevation: true,
    max_elevation: 3,
    enable_obstacles: true,
    obstacle_density: 0.5,
    ..Default::default()
};

let level = generate(&params);

// Access marble tiles
if let Some(tiles) = &level.marble_tiles {
    for (y, row) in tiles.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            println!("Tile at ({}, {}): {:?} at elevation {}", 
                     x, y, tile.tile_type, tile.elevation);
        }
    }
}
```

### Directional Generation Example

```rust
use level_generator::{generate, GeneratorParams, GenerationMode};

let params = GeneratorParams {
    width: 60,
    height: 30,
    rooms: 10,
    mode: GenerationMode::Marble,
    enable_elevation: true,
    max_elevation: 2,
    max_elevation_change: 1, // Constrain elevation changes between rooms
    // Trend vector: extend northeast with upward bias
    trend_vector: Some((1.0, 0.5, 1.0)),
    trend_strength: 0.7,
    // Optional starting point in world coordinates
    start_point: Some((10, 0, 10)),
    ..Default::default()
};

let level = generate(&params);
```

### Generate Visualization

```rust
use level_generator::{generate, generate_html, GeneratorParams};
use std::fs;

let level = generate(&GeneratorParams::default());
let html = generate_html(&level);
fs::write("level.html", html)?;
```

See the `examples/` directory for more complete examples.

### Running Examples

```bash
# Basic dungeon generation
cargo run --example basic_usage

# Custom marble track generation
cargo run --example custom_generation

# Complete marble track with HTML export
cargo run --example marble_track
```

## CLI Usage

### Basic Examples

```bash
# Generate a simple dungeon (ASCII preview)
cargo run -- --width 60 --height 25 --rooms 10

# Generate with specific seed for reproducibility
cargo run -- --seed 42 --width 40 --height 20 --rooms 8

# Export to JSON file
cargo run -- --width 60 --height 25 --rooms 10 --json-path dungeon.json

# Generate HTML visualization
cargo run -- --width 60 --height 25 --rooms 10 --html-path dungeon.html
```

### Marble Track Examples

```bash
# Basic marble track
cargo run -- --mode marble --width 60 --height 30 --rooms 10 --html-path track.html

# Marble track with elevation changes
cargo run -- --mode marble --width 60 --height 30 --rooms 8 --enable-elevation --max-elevation 3 --html-path track.html

# Marble track with constrained elevation changes (smooth transitions)
cargo run -- --mode marble --width 60 --height 30 --rooms 8 --enable-elevation --max-elevation 3 --max-elevation-change 1 --html-path smooth-track.html

# Marble track with obstacles
cargo run -- --mode marble --width 80 --height 40 --rooms 10 --enable-obstacles --obstacle-density 0.5 --html-path track.html

# Complete marble track with all features
cargo run -- --mode marble --enable-elevation --max-elevation 2 --enable-obstacles --obstacle-density 0.3 --channel-width 3 --html-path track.html --json-path track.json
```

### WFC Maze Examples

```bash
# Generate a WFC maze
cargo run -- --mode wfc --width 60 --height 25 --html-path maze.html

# WFC maze with JSON export
cargo run -- --mode wfc --width 60 --height 25 --json-path maze.json --html-path maze.html
```

### Directional Generation Examples

```bash
# Generate level extending northeast (trend vector: 1, 0, 1)
cargo run -- --width 60 --height 30 --rooms 10 --trend-x 1.0 --trend-y 0.0 --trend-z 1.0 --html-path northeast.html

# Generate level with upward elevation trend
cargo run -- --mode marble --width 60 --height 30 --rooms 8 --enable-elevation --trend-x 0.0 --trend-y 1.0 --trend-z 1.0 --trend-strength 0.7 --html-path upward.html

# Generate level from a specific starting point extending east
cargo run -- --width 80 --height 40 --rooms 12 --trend-x 1.0 --trend-y 0.0 --trend-z 0.0 --start-x 10 --start-y 0 --start-z 20 --html-path from-start.html

# Classic dungeon with strong directional bias
cargo run -- --mode classic --width 60 --height 25 --rooms 10 --trend-x 1.0 --trend-y 0.0 --trend-z 0.5 --trend-strength 0.8 --html-path directional.html
```

### Advanced Usage

```bash
# Custom room sizes
cargo run -- --width 80 --height 40 --rooms 15 --min-room 6 --max-room 12

# Wide marble channels
cargo run -- --mode marble --channel-width 4 --corner-radius 3 --width 60 --height 30 --rooms 8

# Export only HTML (skip ASCII and JSON)
cargo run -- --mode marble --html-only --html-path track.html

# Print JSON to stdout
cargo run -- --width 40 --height 20 --rooms 8 --print-json --no-ascii
```

### Options

#### General
- `--width, -w` map width (tiles)
- `--height, -H` map height (tiles)
- `--rooms, -r` target number of rooms to place
- `--min-room, -m` minimum room side length
- `--max-room, -M` maximum room side length
- `--seed, -s` RNG seed for reproducibility
- `--mode` generation mode: `classic` (default), `marble`, or `wfc`

#### Marble Mode
- `--channel-width` channel width in tiles (default: 2)
- `--corner-radius` corner radius for rounded turns (default: 2)
- `--enable-elevation` enable elevation variation between rooms
- `--max-elevation` maximum elevation difference (default: 2)
- `--max-elevation-change` maximum elevation change between adjacent rooms (default: 1)
- `--enable-obstacles` place obstacles in large rooms
- `--obstacle-density` obstacle density 0.0-1.0 (default: 0.3)

#### Directional Generation
- `--trend-x <f32>` X component of trend vector (horizontal direction)
- `--trend-y <f32>` Y component of trend vector (vertical/elevation direction)
- `--trend-z <f32>` Z component of trend vector (horizontal direction)
- `--trend-strength <f32>` bias strength for trend vector, 0.0-1.0 (default: 0.5)
- `--start-x <i32>` starting point X coordinate in world space
- `--start-y <i32>` starting point Y coordinate (elevation) in world space
- `--start-z <i32>` starting point Z coordinate in world space

The trend vector provides a general direction in which the level should extend. Room placement and connections are biased toward this direction with configurable strength. The trend vector uses 3D world coordinates where:
- X and Z components control horizontal direction (map to grid x and y)
- Y component influences elevation bias when elevation is enabled

All trend vector components must be provided together for the feature to activate. The starting point is optional - if not provided, the generator uses the grid center or last placed room as reference.

#### Output
- `--no-ascii` disable ASCII preview
- `--print-json` print JSON to stdout
- `--json-path, -o` path to write JSON file
- `--html-path` path to write isometric HTML visualization
- `--html-only` only generate HTML (skip ASCII/JSON)

## JSON Schema (informal)

```json
{
  "width": 60,
  "height": 25,
  "seed": 13051300863100127324,
  "rooms": [
    { "x": 4, "y": 9, "w": 9, "h": 10, "elevation": 0 }
  ],
  "tiles": [
    "#########...",
    "##......#..."
  ],
  "marble_tiles": [
    [
      {
        "tile_type": "Straight",
        "elevation": 0,
        "rotation": 0,
        "has_walls": true,
        "metadata": ""
      },
      {
        "tile_type": "Curve90",
        "elevation": 1,
        "rotation": 2,
        "has_walls": true,
        "metadata": ""
      }
    ]
  ]
}
```

### Tile Types

The marble mode supports the following tile types:
- `Empty` - Wall/void
- `Straight` - Straight path segment
- `Curve90` - 90-degree curved turn
- `TJunction` - T-shaped 3-way junction
- `YJunction` - Y-shaped smooth 3-way split
- `CrossJunction` - 4-way intersection
- `Slope` - Incline connecting two elevations (±1 level difference)
- `OpenPlatform` - Open area with no walls
- `Obstacle` - Static obstacle (pillar, bumper)
- `Merge` - Multiple inputs converge to one output
- `OneWayGate` - Directional flow control
- `LoopDeLoop` - Vertical loop section
- `HalfPipe` - U-shaped channel
- `LaunchPad` - Catapult/jump section
- `Bridge` - Path crosses over another
- `Tunnel` - Path goes under another

## Isometric Visualization

The `--html-path` option generates an interactive isometric HTML/SVG visualization showing:
- **3D perspective** of the level with proper depth sorting
- **Elevation shading** - lighter tiles are higher
- **Wall rendering** - vertical faces show enclosed paths
- **Tile indicators** - slope icon (⛰) for inclines
- **Color coding** by tile type (straight paths, curves, junctions, slopes, obstacles)
- **Legend** explaining tile types and visual elements

Perfect for previewing marble levels before importing into a game engine!

## Algorithm Details

### Classic Mode
1. Initialize a `width × height` grid with all walls.
2. Try placing up to `rooms` non-overlapping rectangles; each accepted rectangle is carved to floor.
3. Sort rooms by center `x` and connect each to the previous with a horizontal-then-vertical or vertical-then-horizontal tunnel (random choice).
4. Convert the character grid into `Vec<String>` for JSON export and ASCII preview.

### Marble Mode
1. Generate rooms with optional elevation values.
2. Connect rooms with wide channels (using `channel_width` and `corner_radius`).
3. Detect tile types based on connectivity (straight, curve, T-junction, cross).
4. Insert slope tiles where elevation changes occur.
5. Place obstacles randomly in large rooms based on `obstacle_density`.
6. Export as both ASCII and detailed tile grid with metadata.

## Troubleshooting

### Common Issues

**Build fails with "feature 'cli' not found"**
```bash
# Make sure you're building with the cli feature
cargo build --features cli
# Or use the default features
cargo build
```

**HTML visualization doesn't display properly**
- Ensure you're opening the HTML file in a modern web browser
- Check that the file path is correct and the file was created successfully
- Try a different browser if the visualization appears broken

**No rooms generated**
- Try increasing the `--rooms` parameter
- Reduce `--min-room` and `--max-room` sizes
- Increase the map size with `--width` and `--height`

**Marble mode produces empty levels**
- Ensure you're using `--mode marble` (not `--mode classic`)
- Try different `--channel-width` and `--corner-radius` values
- Check that room count and sizes are reasonable for the map size

**JSON output is too large**
- Use `--no-ascii` to skip ASCII preview
- Use `--html-only` to skip JSON output entirely
- Consider smaller map dimensions

### Performance Tips

- For large maps (100x100+), consider using `--html-only` to skip ASCII generation
- Marble mode with elevation and obstacles is more computationally intensive
- Use specific seeds (`--seed`) for reproducible results during development

### Getting Help

- Check the examples in the `examples/` directory
- Run `cargo run -- --help` to see all available options
- Open an issue on GitHub if you encounter bugs

## License

MIT
