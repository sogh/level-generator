# level-generator

A simple Nethack-style dungeon generator written in Rust. It places non-overlapping rectangular rooms and connects them with L-shaped tunnels, exporting the result as ASCII and JSON. It now also supports a "marble" mode for wide, rounded channels suitable for marble runs.

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
```

### Options

- `--width, -w` map width (tiles)
- `--height, -H` map height (tiles)
- `--rooms, -r` target number of rooms to place
- `--min-room, -m` minimum room side length
- `--max-room, -M` maximum room side length
- `--seed, -s` RNG seed for reproducibility
- `--mode` generation mode: `classic` (default) or `marble`
- `--channel-width` marble mode: channel width in tiles
- `--corner-radius` marble mode: corner radius in tiles
- `--no-ascii` disable ASCII preview
- `--print-json` also print JSON to stdout
- `--json-path, -o` path to write JSON

## JSON Schema (informal)

```json
{
  "width": 60,
  "height": 25,
  "seed": 13051300863100127324,
  "rooms": [
    { "x": 4, "y": 9, "w": 9, "h": 10 }
  ],
  "tiles": [
    "#########...",
    "##......#..."
  ]
}
```

## Algorithm Details

1. Initialize a `width × height` grid with all walls.
2. Try placing up to `rooms` non-overlapping rectangles; each accepted rectangle is carved to floor.
3. Sort rooms by center `x` and connect each to the previous with a horizontal-then-vertical or vertical-then-horizontal tunnel (random choice).
4. Convert the character grid into `Vec<String>` for JSON export and ASCII preview.

## License

MIT
