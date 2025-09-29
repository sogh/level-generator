use crate::dungeon::Level;

/// Convert a `Level` into a single ASCII string for preview.
pub fn to_ascii(level: &Level) -> String {
    level.tiles.join("\n")
}


