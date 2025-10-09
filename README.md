# level-generator

A procedural level generator written in Rust with multiple generation modes:
- **Classic**: Nethack-style dungeons with rectangular rooms and L-shaped tunnels
- **Marble**: Wide, rounded channels with elevation changes, slopes, obstacles, and junctions - designed for marble rolling games
- **WFC**: Wave Function Collapse algorithm for pipe-based mazes

Exports levels as ASCII, JSON (with detailed tile metadata), and isometric HTML/SVG visualizations.

## Overview

- Rooms are placed randomly (sizes in `[min_room, max_room]`) and rejected if overlapping an existing room expanded by a margin.
- Rooms are sorted by center `x` and connected pairwise with L-shaped tunnels.
- Tiles: `#` is wall, `.` is floor.
- Generation is reproducible via `--seed`.

## Install

Requires Rust and Cargo.

```bash
cargo build
```

## Usage

```bash
# Show ASCII preview only
cargo run -- --width 60 --height 25 --rooms 10

# Export JSON (also prints to stdout)
cargo run -- --width 60 --height 25 --rooms 10 --no-ascii --print-json --json-path out/level.json

# Reproducible output
cargo run -- --seed 42 --print-json --no-ascii

# Marble mode with rounded channels (width=3, radius=3)
cargo run -- --mode marble --channel-width 3 --corner-radius 3 --width 60 --height 25 --rooms 10 --print-json --no-ascii

# Marble mode with elevation changes and slopes
cargo run -- --mode marble --width 60 --height 30 --rooms 8 --enable-elevation --max-elevation 3 --html-path out/level.html

# Marble mode with obstacles and isometric visualization
cargo run -- --mode marble --width 80 --height 40 --rooms 10 --enable-obstacles --obstacle-density 0.5 --html-path out/level.html

# Complete marble mode with all features
cargo run -- --mode marble --enable-elevation --max-elevation 2 --enable-obstacles --obstacle-density 0.3 --channel-width 3 --html-path out/level.html --json-path out/level.json

# Wave Function Collapse (WFC) mode with a simple pipe tileset
cargo run -- --mode wfc --width 60 --height 25 --print-json --no-ascii
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
- `--enable-obstacles` place obstacles in large rooms
- `--obstacle-density` obstacle density 0.0-1.0 (default: 0.3)

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
- `SlopeUp` - Upward incline (+1 elevation)
- `SlopeDown` - Downward decline (-1 elevation)
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
- **Tile indicators** - arrows for slopes (↗ ↘)
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

## License

MIT
