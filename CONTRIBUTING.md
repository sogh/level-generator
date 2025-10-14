# Contributing to Level Generator

## Running Examples

```bash
# Run all examples
cargo run --example basic_usage
cargo run --example marble_track
cargo run --example custom_generation
```

## Testing

```bash
cargo test
```

## Documentation

Build and view the documentation:

```bash
cargo doc --open
```

## Code Structure

- `src/lib.rs` - Public API and documentation
- `src/dungeon.rs` - Core generation logic
- `src/tiles.rs` - Tile type definitions
- `src/isometric.rs` - HTML/SVG visualization
- `src/visualize.rs` - ASCII rendering
- `src/cli.rs` - Command-line interface (optional, feature-gated)
- `src/main.rs` - CLI binary entry point
