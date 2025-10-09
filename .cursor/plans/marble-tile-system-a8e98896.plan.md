<!-- a8e98896-c31f-49d3-a592-cc86de9e3bf0 e7539057-988f-421b-af78-f0af7062f05a -->
# Marble Tile System with Elevation and Isometric Visualization

## Phase 1: Core Tile Type System

### 1.1 Create tile type infrastructure

- Add `src/tiles.rs` module with:
  - `TileType` enum (Straight, Curve90, TJunction, YJunction, CrossJunction, SlopeUp, SlopeDown, OpenPlatform, Obstacle, etc.)
  - `MarbleTile` struct with fields: `tile_type`, `elevation: i32`, `has_walls: bool`, `rotation: u8`, `width: u32`
  - Connection point system (north/south/east/west) for tile compatibility
- Update `dungeon.rs` to use tile-based grid instead of simple char grid
- Extend `Level` struct to include `Vec<Vec<MarbleTile>>` alongside the legacy ASCII tiles

### 1.2 Update JSON export format

- Add tile metadata to JSON output:
  - Tile type, elevation, rotation, wall info
  - Connection information for game engines
  - Maintain backward compatibility with ASCII representation

## Phase 2: Tile Generation Logic

### 2.1 Basic path tiles (enclosed)

- Implement straight channel generation with tile types
- Implement 90° curves with proper rotation handling
- Implement T-junctions and cross-junctions
- Modify marble mode to use `MarbleTile` grid and place appropriate tile types at corridors and turns

### 2.2 Elevation and slopes

- Add elevation tracking to room placement (each room can be at different height)
- Implement slope tiles (SlopeUp/SlopeDown) to connect rooms at different elevations
- Add validation: slopes can only change 1 level per tile
- Generate slope transitions automatically when connecting rooms at different heights

### 2.3 Open areas and obstacles

- Implement open platform tiles (no walls, can fall off edges)
- Add obstacle placement in large rooms (pillars, bumpers)
- Create "ridge" tiles for narrow exposed paths

## Phase 3: Isometric HTML/SVG Visualization

### 3.1 Create visualization module

- Add `src/isometric.rs` with SVG generation
- Implement isometric projection math (convert x, y, elevation to screen coordinates)
- Define visual styles for each tile type:
  - Different colors/patterns for walls, floors, slopes
  - Elevation shading (darker = lower, lighter = higher)
  - Wall height rendering based on `has_walls` flag

### 3.2 SVG rendering

- Generate HTML file with embedded SVG
- Render tiles layer-by-layer (bottom to top elevation)
- Draw walls as vertical faces on tile edges
- Add basic lighting/shading for depth perception
- Include legend showing tile types and elevation scale

### 3.3 CLI integration

- Add `--html-path` flag to export HTML visualization
- Add `--html-only` flag to skip ASCII output
- Make isometric view the default for marble mode with elevation

## Phase 4: Advanced Tile Types

### 4.1 Special junction tiles

- Y-junction (smooth 3-way split)
- Merge tiles (multiple inputs, one output)
- One-way gates (directional flow)

### 4.2 Interactive/dynamic tiles

- Loop-de-loop (requires marking start/end with elevation +2)
- Half-pipe tiles
- Launch pad tiles
- Speed modifier zones (metadata only, visual indicator)

### 4.3 Multi-level features

- Bridge tiles (path crosses over another)
- Tunnel tiles (path goes under another)
- Add overlap detection to allow multi-level crossings

## Implementation Details

### Key Files to Modify/Create

- **NEW** `src/tiles.rs` - tile type definitions and logic
- **NEW** `src/isometric.rs` - SVG/HTML isometric renderer
- **MODIFY** `src/dungeon.rs` - replace char grid with tile grid, add elevation generation
- **MODIFY** `src/cli.rs` - add HTML export flags
- **MODIFY** `src/main.rs` - wire up new visualization output
- **MODIFY** `Cargo.toml` - add dependencies if needed (likely none for basic SVG)

### Data Structures

```rust
// tiles.rs
pub enum TileType {
    Empty,           // Wall/void
    Straight,        // Straight path
    Curve90,         // 90-degree turn
    TJunction,       // T-shaped junction
    CrossJunction,   // 4-way intersection
    SlopeUp,         // Incline +1 elevation
    SlopeDown,       // Decline -1 elevation
    OpenPlatform,    // No walls, can fall
    Obstacle,        // Static obstacle in open area
    // ... more types in later phases
}

pub struct MarbleTile {
    pub tile_type: TileType,
    pub elevation: i32,      // 0 = ground level
    pub rotation: u8,        // 0-3 for 90° rotations
    pub has_walls: bool,
    pub metadata: String,    // JSON string for game engine
}
```

### JSON Export Example

```json
{
  "width": 60,
  "height": 25,
  "seed": 12345,
  "tiles": [...],  // Legacy ASCII
  "marble_tiles": [
    {
      "x": 5, "y": 10,
      "type": "Curve90",
      "elevation": 0,
      "rotation": 1,
      "has_walls": true
    }
  ]
}
```

### SVG Isometric Projection

```rust
fn to_isometric(x: f32, y: f32, z: f32) -> (f32, f32) {
    let iso_x = (x - y) * TILE_WIDTH / 2.0;
    let iso_y = (x + y) * TILE_HEIGHT / 4.0 - z * ELEVATION_HEIGHT;
    (iso_x, iso_y)
}
```

## Testing Strategy

- Unit tests for tile connection validation
- Regression tests for ASCII output (ensure backward compatibility)
- Visual inspection of generated HTML files
- Validate JSON schema for game engine consumption

### To-dos

- [ ] Create tiles.rs with TileType enum and MarbleTile struct
- [ ] Replace char grid with MarbleTile grid in dungeon.rs
- [ ] Extend JSON export with tile metadata
- [ ] Implement basic path tile generation (straight, curves, junctions)
- [ ] Add elevation tracking and slope tile generation
- [ ] Implement open platform and obstacle tiles
- [ ] Create isometric.rs with SVG generation and projection math
- [ ] Implement full SVG tile rendering with walls and elevation shading
- [ ] Add HTML export flags to CLI and wire up in main.rs
- [ ] Implement Y-junction, merge, and one-way tiles
- [ ] Add loop-de-loop, half-pipe, and speed modifier tiles
- [ ] Implement bridge/tunnel tiles with overlap detection