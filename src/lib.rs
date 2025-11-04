//! # Level Generator
//!
//! A procedural level generator for creating dungeons, marble rolling tracks, and mazes.
//!
//! ## Quick Start
//!
//! ```rust
//! use level_generator::{generate, GeneratorParams, GenerationMode};
//!
//! let params = GeneratorParams {
//!     width: 80,
//!     height: 40,
//!     rooms: 12,
//!     min_room: 4,
//!     max_room: 10,
//!     seed: Some(42),
//!     mode: GenerationMode::Marble,
//!     // Optional: bias generation in a specific direction
//!     trend_vector: Some((1.0, 0.0, 1.0)), 
//!     trend_strength: 0.7,
//!     // Optional: start generation from a specific point
//!     start_point: Some((10, 0, 10)),
//!     ..Default::default()
//! };
//!
//! let level = generate(&params);
//! println!("Generated level with {} rooms", level.rooms.len());
//! ```
//!
//! ## Generation Modes
//!
//! - **Classic**: Traditional roguelike dungeons with rooms and corridors
//! - **Marble**: Wide channels with curves, elevation, slopes, and obstacles for marble games
//! - **WFC**: Wave Function Collapse algorithm for connected mazes
//!
//! ## Features
//!
//! - Reproducible generation with seeds
//! - JSON export with detailed tile metadata
//! - Isometric HTML/SVG visualization
//! - 16+ tile types for complex marble tracks
//! - Elevation system with automatic slope generation

#[cfg(feature = "cli")]
pub mod cli;

pub mod dungeon;
pub mod isometric;
pub mod tiles;
pub mod visualize;

// Re-export commonly used types for convenience
pub use dungeon::{generate, GenerationMode, GeneratorParams, Level, Room};
pub use tiles::{Direction, MarbleTile, TileType};
pub use isometric::generate_html;
pub use visualize::to_ascii;


