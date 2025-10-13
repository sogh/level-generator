//! Isometric HTML/SVG visualization for marble levels.
//!
//! This module provides isometric rendering of marble tile levels,
//! showing elevation, walls, and different tile types in 3D perspective.

use crate::dungeon::Level;
use crate::tiles::{MarbleTile, TileType};

/// Tile dimensions for isometric projection
const TILE_WIDTH: f32 = 32.0;
const TILE_HEIGHT: f32 = 16.0;
const ELEVATION_HEIGHT: f32 = 12.0;
const WALL_HEIGHT: f32 = 20.0;

/// Convert 3D coordinates to isometric 2D screen coordinates
fn to_isometric(x: f32, y: f32, z: f32) -> (f32, f32) {
    let iso_x = (x - y) * TILE_WIDTH / 2.0;
    let iso_y = (x + y) * TILE_HEIGHT / 4.0 - z * ELEVATION_HEIGHT;
    (iso_x, iso_y)
}

/// Get color for a tile type
fn tile_color(tile_type: &TileType) -> &'static str {
    match tile_type {
        TileType::Empty => "#2b2b2b",
        TileType::Straight => "#5a9fd4",
        TileType::Curve90 => "#5aa4d4",
        TileType::TJunction => "#4c8fc7",
        TileType::YJunction => "#4c8fc7",
        TileType::CrossJunction => "#4080b8",
        TileType::Slope => "#e8a847",
        TileType::OpenPlatform => "#a6a6a6",
        TileType::Obstacle => "#8b4513",
        TileType::Merge => "#6b7fc7",
        TileType::OneWayGate => "#c74c8f",
        TileType::LoopDeLoop => "#c7478f",
        TileType::HalfPipe => "#8f47c7",
        TileType::LaunchPad => "#ff4444",
        TileType::Bridge => "#7fc76b",
        TileType::Tunnel => "#4c6bc7",
    }
}

/// Adjust color brightness based on elevation (lighter = higher)
fn adjust_color_for_elevation(base_color: &str, elevation: i32) -> String {
    // Parse hex color
    let r = u8::from_str_radix(&base_color[1..3], 16).unwrap_or(128);
    let g = u8::from_str_radix(&base_color[3..5], 16).unwrap_or(128);
    let b = u8::from_str_radix(&base_color[5..7], 16).unwrap_or(128);
    
    // Adjust brightness: +10% per elevation level
    let factor = 1.0 + (elevation as f32 * 0.1);
    let r = (r as f32 * factor).min(255.0) as u8;
    let g = (g as f32 * factor).min(255.0) as u8;
    let b = (b as f32 * factor).min(255.0) as u8;
    
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Render a single tile as SVG polygons
fn render_tile_svg(tile: &MarbleTile, x: usize, y: usize, svg: &mut String) {
    if tile.tile_type == TileType::Empty {
        return;
    }
    
    let fx = x as f32;
    let fy = y as f32;
    let fz = tile.elevation as f32;
    
    // Get base color and adjust for elevation
    let base_color = tile_color(&tile.tile_type);
    let color = adjust_color_for_elevation(base_color, tile.elevation);
    
    // Calculate corners of the tile top surface
    let (x0, y0) = to_isometric(fx, fy, fz);
    let (x1, y1) = to_isometric(fx + 1.0, fy, fz);
    let (x2, y2) = to_isometric(fx + 1.0, fy + 1.0, fz);
    let (x3, y3) = to_isometric(fx, fy + 1.0, fz);
    
    // Draw top surface
    let polygon_points = format!("{},{} {},{} {},{} {},{}", x0, y0, x1, y1, x2, y2, x3, y3);
    svg.push_str(&format!(
        "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#333\" stroke-width=\"1\"/>\n",
        polygon_points, color
    ));
    
    // Draw walls if the tile has walls
    if tile.has_walls {
        // Darken color for walls
        let wall_color = darken_color(&color, 0.7);
        
        // South wall (front-left face)
        let (bx3, by3) = to_isometric(fx, fy + 1.0, fz - WALL_HEIGHT / ELEVATION_HEIGHT);
        let (bx2, by2) = to_isometric(fx + 1.0, fy + 1.0, fz - WALL_HEIGHT / ELEVATION_HEIGHT);
        
        let wall_points = format!("{},{} {},{} {},{} {},{}", x3, y3, x2, y2, bx2, by2, bx3, by3);
        svg.push_str(&format!(
            "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#222\" stroke-width=\"0.5\" opacity=\"0.9\"/>\n",
            wall_points, wall_color
        ));
        
        // East wall (front-right face)
        let (bx1, by1) = to_isometric(fx + 1.0, fy, fz - WALL_HEIGHT / ELEVATION_HEIGHT);
        
        let wall_points2 = format!("{},{} {},{} {},{} {},{}", x1, y1, x2, y2, bx2, by2, bx1, by1);
        svg.push_str(&format!(
            "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#222\" stroke-width=\"0.5\" opacity=\"0.8\"/>\n",
            wall_points2, darken_color(&color, 0.6)
        ));
    }
    
    // Add tile type indicator for special tiles
    if matches!(tile.tile_type, TileType::Slope) {
        let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz);
        // Show a slope indicator (⛰ or ⇅)
        svg.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" font-size=\"16\" fill=\"#fff\" text-anchor=\"middle\" dominant-baseline=\"middle\">⛰</text>\n",
            cx, cy
        ));
    }
}

/// Darken a hex color by a factor (0.0 = black, 1.0 = original)
fn darken_color(hex: &str, factor: f32) -> String {
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(128);
    
    let r = (r as f32 * factor) as u8;
    let g = (g as f32 * factor) as u8;
    let b = (b as f32 * factor) as u8;
    
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Generate HTML with embedded SVG for isometric visualization
pub fn generate_html(level: &Level) -> String {
    let mut html = String::new();
    
    // HTML header
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str("  <title>Marble Level - Isometric View</title>\n");
    html.push_str("  <style>\n");
    html.push_str("    body { margin: 0; padding: 20px; background: #1a1a1a; font-family: Arial, sans-serif; }\n");
    html.push_str("    .container { max-width: 1400px; margin: 0 auto; }\n");
    html.push_str("    h1 { color: #fff; text-align: center; }\n");
    html.push_str("    .info { color: #aaa; text-align: center; margin: 10px 0; }\n");
    html.push_str("    svg { background: #0d0d0d; display: block; margin: 20px auto; border: 2px solid #333; }\n");
    html.push_str("    .legend { color: #fff; background: #2a2a2a; padding: 15px; border-radius: 5px; margin-top: 20px; }\n");
    html.push_str("    .legend-item { display: inline-block; margin: 5px 15px; }\n");
    html.push_str("    .legend-color { display: inline-block; width: 20px; height: 20px; margin-right: 5px; vertical-align: middle; border: 1px solid #555; }\n");
    html.push_str("  </style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("  <div class=\"container\">\n");
    html.push_str(&format!("    <h1>Marble Level Generator - Isometric View</h1>\n"));
    html.push_str(&format!("    <div class=\"info\">Seed: {} | Size: {}×{} | Rooms: {}</div>\n", 
        level.seed, level.width, level.height, level.rooms.len()));
    
    // Generate SVG
    if let Some(marble_tiles) = &level.marble_tiles {
        let height = marble_tiles.len();
        let width = if height > 0 { marble_tiles[0].len() } else { 0 };
        
        // Calculate SVG dimensions with padding
        let svg_width = (width as f32 + height as f32) * TILE_WIDTH / 2.0 + 200.0;
        let svg_height = (width as f32 + height as f32) * TILE_HEIGHT / 4.0 + 400.0;
        
        // Offset to center the view
        let offset_x = svg_width / 2.0;
        let offset_y = 150.0;
        
        html.push_str(&format!("    <svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
            svg_width, svg_height, svg_width, svg_height));
        html.push_str(&format!("      <g transform=\"translate({}, {})\">\n", offset_x, offset_y));
        
        // Render tiles from back to front (isometric painter's algorithm)
        // Sort by y + x to render in correct order
        for sum in 0..(width + height) {
            for y in 0..height {
                let x = sum.saturating_sub(y);
                if x < width {
                    render_tile_svg(&marble_tiles[y][x], x, y, &mut html);
                }
            }
        }
        
        html.push_str("      </g>\n");
        html.push_str("    </svg>\n");
    } else {
        html.push_str("    <p style=\"color: #fff; text-align: center;\">No marble tile data available. Use --mode marble to generate.</p>\n");
    }
    
    // Legend
    html.push_str("    <div class=\"legend\">\n");
    html.push_str("      <strong>Legend:</strong><br>\n");
    html.push_str(&format!("      <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Straight Path</div>\n", tile_color(&TileType::Straight)));
    html.push_str(&format!("      <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Curve</div>\n", tile_color(&TileType::Curve90)));
    html.push_str(&format!("      <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Junction</div>\n", tile_color(&TileType::TJunction)));
    html.push_str(&format!("      <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Slope ⛰</div>\n", tile_color(&TileType::Slope)));
    html.push_str(&format!("      <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Open Platform</div>\n", tile_color(&TileType::OpenPlatform)));
    html.push_str("      <div style=\"margin-top: 10px;\"><em>Note: Lighter shades indicate higher elevation</em></div>\n");
    html.push_str("    </div>\n");
    
    html.push_str("  </div>\n");
    html.push_str("</body>\n</html>");
    
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isometric_projection() {
        let (x, y) = to_isometric(0.0, 0.0, 0.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        
        let (x, y) = to_isometric(1.0, 0.0, 0.0);
        assert_eq!(x, TILE_WIDTH / 2.0);
        assert_eq!(y, TILE_HEIGHT / 4.0);
    }

    #[test]
    fn test_color_adjustment() {
        let base = "#808080";
        let elevated = adjust_color_for_elevation(base, 1);
        assert_ne!(base, elevated);
        // Higher elevation should be brighter
        assert!(elevated > base.to_string());
    }
}

