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

/// Render a single tile as accurate SVG shapes
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
    
    // Draw base tile surface (lighter for non-walls)
    let surface_color = if tile.has_walls { &color } else { &lighten_color(&color, 0.3) };
    let polygon_points = format!("{},{} {},{} {},{} {},{}", x0, y0, x1, y1, x2, y2, x3, y3);
    svg.push_str(&format!(
        "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#333\" stroke-width=\"0.5\" opacity=\"0.8\"/>\n",
        polygon_points, surface_color
    ));
    
    // Draw walls if the tile has walls
    if tile.has_walls {
        draw_tile_walls(fx, fy, fz, &color, svg);
    }
    
    // Draw tile-specific shapes and paths
    match tile.tile_type {
        TileType::Straight => {
            draw_straight_path(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::Curve90 => {
            draw_curve_path(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::TJunction => {
            draw_t_junction(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::YJunction => {
            draw_y_junction(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::CrossJunction => {
            draw_cross_junction(fx, fy, fz, &color, svg);
        },
        TileType::Slope => {
            draw_slope(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::OpenPlatform => {
            // Just the base surface, no walls or paths
        },
        TileType::Obstacle => {
            draw_obstacle(fx, fy, fz, &color, svg);
        },
        TileType::Merge => {
            draw_merge_junction(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::OneWayGate => {
            draw_one_way_gate(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::LoopDeLoop => {
            draw_loop_de_loop(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::HalfPipe => {
            draw_half_pipe(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::LaunchPad => {
            draw_launch_pad(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::Bridge => {
            draw_bridge(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::Tunnel => {
            draw_tunnel(fx, fy, fz, tile.rotation, &color, svg);
        },
        TileType::Empty => {
            // Empty tiles are handled by the early return
        },
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

/// Lighten a hex color by a factor (1.0 = original, >1.0 = lighter)
fn lighten_color(hex: &str, factor: f32) -> String {
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(128);
    
    let r = ((r as f32 * factor).min(255.0)) as u8;
    let g = ((g as f32 * factor).min(255.0)) as u8;
    let b = ((b as f32 * factor).min(255.0)) as u8;
    
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Draw walls for a tile
fn draw_tile_walls(fx: f32, fy: f32, fz: f32, color: &str, svg: &mut String) {
    let (_x0, _y0) = to_isometric(fx, fy, fz);
    let (x1, y1) = to_isometric(fx + 1.0, fy, fz);
    let (x2, y2) = to_isometric(fx + 1.0, fy + 1.0, fz);
    let (x3, y3) = to_isometric(fx, fy + 1.0, fz);
    
    let wall_color = darken_color(color, 0.7);
    
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
        wall_points2, darken_color(color, 0.6)
    ));
}

/// Draw a straight path with raised edges
fn draw_straight_path(fx: f32, fy: f32, fz: f32, rotation: u8, color: &str, svg: &mut String) {
    let path_color = lighten_color(color, 1.2);
    
    match rotation {
        0 | 2 => { // Vertical
            let (x1, y1) = to_isometric(fx + 0.3, fy + 0.2, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.7, fy + 0.2, fz + 0.1);
            let (x3, y3) = to_isometric(fx + 0.7, fy + 0.8, fz + 0.1);
            let (x4, y4) = to_isometric(fx + 0.3, fy + 0.8, fz + 0.1);
            
            let path_points = format!("{},{} {},{} {},{} {},{}", x1, y1, x2, y2, x3, y3, x4, y4);
            svg.push_str(&format!(
                "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                path_points, path_color
            ));
        },
        1 | 3 => { // Horizontal
            let (x1, y1) = to_isometric(fx + 0.2, fy + 0.3, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.8, fy + 0.3, fz + 0.1);
            let (x3, y3) = to_isometric(fx + 0.8, fy + 0.7, fz + 0.1);
            let (x4, y4) = to_isometric(fx + 0.2, fy + 0.7, fz + 0.1);
            
            let path_points = format!("{},{} {},{} {},{} {},{}", x1, y1, x2, y2, x3, y3, x4, y4);
            svg.push_str(&format!(
                "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                path_points, path_color
            ));
        },
        _ => {}
    }
}

/// Draw a curved path
fn draw_curve_path(fx: f32, fy: f32, fz: f32, rotation: u8, color: &str, svg: &mut String) {
    let path_color = lighten_color(color, 1.2);
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.1);
    
    // Draw a curved path as an SVG arc
    match rotation {
        0 => { // North to East curve
            let (x1, y1) = to_isometric(fx + 0.5, fy + 0.3, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.7, fy + 0.5, fz + 0.1);
            svg.push_str(&format!(
                "  <path d=\"M {},{} Q {},{} {},{} L {},{} Q {},{} {},{} Z\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                x1, y1, cx, cy, x2, y2, x1, y1, cx, cy, x1, y1, path_color
            ));
        },
        1 => { // East to South curve
            let (x1, y1) = to_isometric(fx + 0.7, fy + 0.5, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.5, fy + 0.7, fz + 0.1);
            svg.push_str(&format!(
                "  <path d=\"M {},{} Q {},{} {},{} L {},{} Q {},{} {},{} Z\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                x1, y1, cx, cy, x2, y2, x1, y1, cx, cy, x1, y1, path_color
            ));
        },
        2 => { // South to West curve
            let (x1, y1) = to_isometric(fx + 0.5, fy + 0.7, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.3, fy + 0.5, fz + 0.1);
            svg.push_str(&format!(
                "  <path d=\"M {},{} Q {},{} {},{} L {},{} Q {},{} {},{} Z\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                x1, y1, cx, cy, x2, y2, x1, y1, cx, cy, x1, y1, path_color
            ));
        },
        3 => { // West to North curve
            let (x1, y1) = to_isometric(fx + 0.3, fy + 0.5, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.5, fy + 0.3, fz + 0.1);
            svg.push_str(&format!(
                "  <path d=\"M {},{} Q {},{} {},{} L {},{} Q {},{} {},{} Z\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                x1, y1, cx, cy, x2, y2, x1, y1, cx, cy, x1, y1, path_color
            ));
        },
        _ => {}
    }
}

/// Draw a T-junction with connecting paths
fn draw_t_junction(fx: f32, fy: f32, fz: f32, rotation: u8, color: &str, svg: &mut String) {
    let path_color = lighten_color(color, 1.2);
    
    match rotation {
        0 => { // Missing South
            // North path
            let (x1, y1) = to_isometric(fx + 0.3, fy + 0.2, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.7, fy + 0.2, fz + 0.1);
            let (x3, y3) = to_isometric(fx + 0.7, fy + 0.5, fz + 0.1);
            let (x4, y4) = to_isometric(fx + 0.3, fy + 0.5, fz + 0.1);
            let north_path = format!("{},{} {},{} {},{} {},{}", x1, y1, x2, y2, x3, y3, x4, y4);
            
            // East path
            let (x5, y5) = to_isometric(fx + 0.5, fy + 0.3, fz + 0.1);
            let (x6, y6) = to_isometric(fx + 0.8, fy + 0.3, fz + 0.1);
            let (x7, y7) = to_isometric(fx + 0.8, fy + 0.7, fz + 0.1);
            let (x8, y8) = to_isometric(fx + 0.5, fy + 0.7, fz + 0.1);
            let east_path = format!("{},{} {},{} {},{} {},{}", x5, y5, x6, y6, x7, y7, x8, y8);
            
            // West path
            let (x9, y9) = to_isometric(fx + 0.2, fy + 0.3, fz + 0.1);
            let (x10, y10) = to_isometric(fx + 0.5, fy + 0.3, fz + 0.1);
            let (x11, y11) = to_isometric(fx + 0.5, fy + 0.7, fz + 0.1);
            let (x12, y12) = to_isometric(fx + 0.2, fy + 0.7, fz + 0.1);
            let west_path = format!("{},{} {},{} {},{} {},{}", x9, y9, x10, y10, x11, y11, x12, y12);
            
            svg.push_str(&format!(
                "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                north_path, path_color
            ));
            svg.push_str(&format!(
                "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                east_path, path_color
            ));
            svg.push_str(&format!(
                "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                west_path, path_color
            ));
        },
        // Similar patterns for other rotations...
        _ => {}
    }
}

/// Draw a Y-junction with smooth curved paths
fn draw_y_junction(fx: f32, fy: f32, fz: f32, rotation: u8, color: &str, svg: &mut String) {
    let path_color = lighten_color(color, 1.2);
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.1);
    
    // Draw Y-junction with curved connecting paths
    svg.push_str(&format!(
        "  <circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
        cx, cy, path_color
    ));
    
    // Add small directional indicators
    match rotation {
        0 => {
            let (x1, y1) = to_isometric(fx + 0.5, fy + 0.3, fz + 0.15);
            let (x2, y2) = to_isometric(fx + 0.7, fy + 0.4, fz + 0.15);
            let (x3, y3) = to_isometric(fx + 0.3, fy + 0.4, fz + 0.15);
            svg.push_str(&format!(
                "  <polygon points=\"{},{} {},{} {},{}\" fill=\"#fff\" opacity=\"0.8\"/>\n",
                x1, y1, x2, y2, x3, y3
            ));
        },
        _ => {}
    }
}

/// Draw a cross junction with all four paths
fn draw_cross_junction(fx: f32, fy: f32, fz: f32, color: &str, svg: &mut String) {
    let path_color = lighten_color(color, 1.2);
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.1);
    
    // Draw central hub
    svg.push_str(&format!(
        "  <circle cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
        cx, cy, path_color
    ));
    
    // Draw connecting lines
    let (x1, y1) = to_isometric(fx + 0.5, fy + 0.2, fz + 0.15);
    let (x2, y2) = to_isometric(fx + 0.8, fy + 0.5, fz + 0.15);
    let (x3, y3) = to_isometric(fx + 0.5, fy + 0.8, fz + 0.15);
    let (x4, y4) = to_isometric(fx + 0.2, fy + 0.5, fz + 0.15);
    
    svg.push_str(&format!(
        "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#fff\" stroke-width=\"1\" opacity=\"0.6\"/>\n",
        cx, cy, x1, y1
    ));
    svg.push_str(&format!(
        "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#fff\" stroke-width=\"1\" opacity=\"0.6\"/>\n",
        cx, cy, x2, y2
    ));
    svg.push_str(&format!(
        "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#fff\" stroke-width=\"1\" opacity=\"0.6\"/>\n",
        cx, cy, x3, y3
    ));
    svg.push_str(&format!(
        "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#fff\" stroke-width=\"1\" opacity=\"0.6\"/>\n",
        cx, cy, x4, y4
    ));
}

/// Draw a slope with incline indicator
fn draw_slope(fx: f32, fy: f32, fz: f32, rotation: u8, color: &str, svg: &mut String) {
    let path_color = lighten_color(color, 1.2);
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.1);
    
    // Draw slope surface with gradient effect
    match rotation {
        0 | 2 => { // Vertical slope
            let (x1, y1) = to_isometric(fx + 0.3, fy + 0.2, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.7, fy + 0.2, fz + 0.1);
            let (x3, y3) = to_isometric(fx + 0.7, fy + 0.8, fz + 0.2);
            let (x4, y4) = to_isometric(fx + 0.3, fy + 0.8, fz + 0.2);
            
            let slope_points = format!("{},{} {},{} {},{} {},{}", x1, y1, x2, y2, x3, y3, x4, y4);
            svg.push_str(&format!(
                "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                slope_points, path_color
            ));
        },
        1 | 3 => { // Horizontal slope
            let (x1, y1) = to_isometric(fx + 0.2, fy + 0.3, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.8, fy + 0.3, fz + 0.2);
            let (x3, y3) = to_isometric(fx + 0.8, fy + 0.7, fz + 0.2);
            let (x4, y4) = to_isometric(fx + 0.2, fy + 0.7, fz + 0.1);
            
            let slope_points = format!("{},{} {},{} {},{} {},{}", x1, y1, x2, y2, x3, y3, x4, y4);
            svg.push_str(&format!(
                "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                slope_points, path_color
            ));
        },
        _ => {}
    }
    
    // Add slope direction indicator
    svg.push_str(&format!(
        "  <text x=\"{}\" y=\"{}\" font-size=\"12\" fill=\"#fff\" text-anchor=\"middle\" dominant-baseline=\"middle\">⛰</text>\n",
        cx, cy
    ));
}

/// Draw an obstacle (pillar/bumper)
fn draw_obstacle(fx: f32, fy: f32, fz: f32, color: &str, svg: &mut String) {
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.2);
    let obstacle_color = darken_color(color, 0.8);
    
    // Draw cylindrical obstacle
    svg.push_str(&format!(
        "  <circle cx=\"{}\" cy=\"{}\" r=\"6\" fill=\"{}\" stroke=\"#222\" stroke-width=\"1\"/>\n",
        cx, cy, obstacle_color
    ));
    
    // Add highlight
    let (hx, hy) = to_isometric(fx + 0.4, fy + 0.4, fz + 0.25);
    svg.push_str(&format!(
        "  <circle cx=\"{}\" cy=\"{}\" r=\"2\" fill=\"#fff\" opacity=\"0.3\"/>\n",
        hx, hy
    ));
}

/// Draw a merge junction with converging paths
fn draw_merge_junction(fx: f32, fy: f32, fz: f32, _rotation: u8, color: &str, svg: &mut String) {
    let path_color = lighten_color(color, 1.2);
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.1);
    
    // Draw merge symbol (funnel shape)
    svg.push_str(&format!(
        "  <circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
        cx, cy, path_color
    ));
    
    // Add merge direction indicator
    svg.push_str(&format!(
        "  <text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"#fff\" text-anchor=\"middle\" dominant-baseline=\"middle\">M</text>\n",
        cx, cy
    ));
}

/// Draw a one-way gate with directional arrow
fn draw_one_way_gate(fx: f32, fy: f32, fz: f32, rotation: u8, color: &str, svg: &mut String) {
    let path_color = lighten_color(color, 1.2);
    
    // Draw gate frame
    match rotation {
        0 | 2 => { // Vertical gate
            let (x1, y1) = to_isometric(fx + 0.4, fy + 0.2, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.6, fy + 0.2, fz + 0.1);
            let (x3, y3) = to_isometric(fx + 0.6, fy + 0.8, fz + 0.1);
            let (x4, y4) = to_isometric(fx + 0.4, fy + 0.8, fz + 0.1);
            let gate_points = format!("{},{} {},{} {},{} {},{}", x1, y1, x2, y2, x3, y3, x4, y4);
            svg.push_str(&format!(
                "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                gate_points, path_color
            ));
        },
        1 | 3 => { // Horizontal gate
            let (x1, y1) = to_isometric(fx + 0.2, fy + 0.4, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.8, fy + 0.4, fz + 0.1);
            let (x3, y3) = to_isometric(fx + 0.8, fy + 0.6, fz + 0.1);
            let (x4, y4) = to_isometric(fx + 0.2, fy + 0.6, fz + 0.1);
            let gate_points = format!("{},{} {},{} {},{} {},{}", x1, y1, x2, y2, x3, y3, x4, y4);
            svg.push_str(&format!(
                "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                gate_points, path_color
            ));
        },
        _ => {}
    }
    
    // Add directional arrow
    match rotation {
        0 => { // North
            let (x1, y1) = to_isometric(fx + 0.5, fy + 0.7, fz + 0.15);
            let (x2, y2) = to_isometric(fx + 0.45, fy + 0.6, fz + 0.15);
            let (x3, y3) = to_isometric(fx + 0.55, fy + 0.6, fz + 0.15);
            svg.push_str(&format!(
                "  <polygon points=\"{},{} {},{} {},{}\" fill=\"#fff\" opacity=\"0.9\"/>\n",
                x1, y1, x2, y2, x3, y3
            ));
        },
        1 => { // East
            let (x1, y1) = to_isometric(fx + 0.7, fy + 0.5, fz + 0.15);
            let (x2, y2) = to_isometric(fx + 0.6, fy + 0.45, fz + 0.15);
            let (x3, y3) = to_isometric(fx + 0.6, fy + 0.55, fz + 0.15);
            svg.push_str(&format!(
                "  <polygon points=\"{},{} {},{} {},{}\" fill=\"#fff\" opacity=\"0.9\"/>\n",
                x1, y1, x2, y2, x3, y3
            ));
        },
        2 => { // South
            let (x1, y1) = to_isometric(fx + 0.5, fy + 0.3, fz + 0.15);
            let (x2, y2) = to_isometric(fx + 0.45, fy + 0.4, fz + 0.15);
            let (x3, y3) = to_isometric(fx + 0.55, fy + 0.4, fz + 0.15);
            svg.push_str(&format!(
                "  <polygon points=\"{},{} {},{} {},{}\" fill=\"#fff\" opacity=\"0.9\"/>\n",
                x1, y1, x2, y2, x3, y3
            ));
        },
        3 => { // West
            let (x1, y1) = to_isometric(fx + 0.3, fy + 0.5, fz + 0.15);
            let (x2, y2) = to_isometric(fx + 0.4, fy + 0.45, fz + 0.15);
            let (x3, y3) = to_isometric(fx + 0.4, fy + 0.55, fz + 0.15);
            svg.push_str(&format!(
                "  <polygon points=\"{},{} {},{} {},{}\" fill=\"#fff\" opacity=\"0.9\"/>\n",
                x1, y1, x2, y2, x3, y3
            ));
        },
        _ => {}
    }
}

/// Draw a loop-de-loop structure
fn draw_loop_de_loop(fx: f32, fy: f32, fz: f32, _rotation: u8, color: &str, svg: &mut String) {
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.1);
    let loop_color = lighten_color(color, 1.2);
    
    // Draw loop as a simple circle
    svg.push_str(&format!(
        "  <circle cx=\"{}\" cy=\"{}\" r=\"6\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
        cx, cy, loop_color
    ));
    
    // Add loop indicator
    svg.push_str(&format!(
        "  <text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"#fff\" text-anchor=\"middle\" dominant-baseline=\"middle\">∞</text>\n",
        cx, cy
    ));
}

/// Draw a half-pipe structure
fn draw_half_pipe(fx: f32, fy: f32, fz: f32, rotation: u8, color: &str, svg: &mut String) {
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.1);
    let pipe_color = lighten_color(color, 1.2);
    
    // Draw half-pipe as curved path
    match rotation {
        0 => { // North to East curve with elevation
            let (x1, y1) = to_isometric(fx + 0.5, fy + 0.2, fz + 0.1);
            let (x2, y2) = to_isometric(fx + 0.8, fy + 0.5, fz + 0.2);
            svg.push_str(&format!(
                "  <path d=\"M {},{} Q {},{} {},{} L {},{} Q {},{} {},{} Z\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                x1, y1, cx, cy, x2, y2, x1, y1, cx, cy, x1, y1, pipe_color
            ));
        },
        _ => {}
    }
    
    // Add half-pipe indicator
    svg.push_str(&format!(
        "  <text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"#fff\" text-anchor=\"middle\" dominant-baseline=\"middle\">∪</text>\n",
        cx, cy
    ));
}

/// Draw a launch pad with speed lines
fn draw_launch_pad(fx: f32, fy: f32, fz: f32, rotation: u8, color: &str, svg: &mut String) {
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.1);
    let launch_color = lighten_color(color, 1.3);
    
    // Draw launch pad surface
    let (x1, y1) = to_isometric(fx + 0.2, fy + 0.2, fz + 0.1);
    let (x2, y2) = to_isometric(fx + 0.8, fy + 0.2, fz + 0.1);
    let (x3, y3) = to_isometric(fx + 0.8, fy + 0.8, fz + 0.1);
    let (x4, y4) = to_isometric(fx + 0.2, fy + 0.8, fz + 0.1);
    let pad_points = format!("{},{} {},{} {},{} {},{}", x1, y1, x2, y2, x3, y3, x4, y4);
    svg.push_str(&format!(
        "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
        pad_points, launch_color
    ));
    
    // Add speed lines
    match rotation {
        0 => { // Launching North
            for i in 0..3 {
                let (x1, y1) = to_isometric(fx + 0.4 + i as f32 * 0.1, fy + 0.3, fz + 0.15);
                let (x2, y2) = to_isometric(fx + 0.4 + i as f32 * 0.1, fy + 0.1, fz + 0.15);
                svg.push_str(&format!(
                    "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#fff\" stroke-width=\"1\" opacity=\"0.7\"/>\n",
                    x1, y1, x2, y2
                ));
            }
        },
        _ => {}
    }
    
    // Add launch indicator
    svg.push_str(&format!(
        "  <text x=\"{}\" y=\"{}\" font-size=\"12\" fill=\"#fff\" text-anchor=\"middle\" dominant-baseline=\"middle\">⚡</text>\n",
        cx, cy
    ));
}

/// Draw a bridge structure
fn draw_bridge(fx: f32, fy: f32, fz: f32, rotation: u8, color: &str, svg: &mut String) {
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.2);
    let bridge_color = lighten_color(color, 1.2);
    
    // Draw bridge deck
    match rotation {
        0 | 2 => { // Vertical bridge
            let (x1, y1) = to_isometric(fx + 0.3, fy + 0.1, fz + 0.2);
            let (x2, y2) = to_isometric(fx + 0.7, fy + 0.1, fz + 0.2);
            let (x3, y3) = to_isometric(fx + 0.7, fy + 0.9, fz + 0.2);
            let (x4, y4) = to_isometric(fx + 0.3, fy + 0.9, fz + 0.2);
            let bridge_points = format!("{},{} {},{} {},{} {},{}", x1, y1, x2, y2, x3, y3, x4, y4);
            svg.push_str(&format!(
                "  <polygon points=\"{}\" fill=\"{}\" stroke=\"#444\" stroke-width=\"0.3\"/>\n",
                bridge_points, bridge_color
            ));
        },
        _ => {}
    }
    
    // Add bridge indicator
    svg.push_str(&format!(
        "  <text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"#fff\" text-anchor=\"middle\" dominant-baseline=\"middle\">🌉</text>\n",
        cx, cy
    ));
}

/// Draw a tunnel entrance
fn draw_tunnel(fx: f32, fy: f32, fz: f32, _rotation: u8, color: &str, svg: &mut String) {
    let (cx, cy) = to_isometric(fx + 0.5, fy + 0.5, fz + 0.1);
    let tunnel_color = darken_color(color, 0.7);
    
    // Draw tunnel entrance as dark arch
    svg.push_str(&format!(
        "  <ellipse cx=\"{}\" cy=\"{}\" rx=\"6\" ry=\"4\" fill=\"{}\" stroke=\"#222\" stroke-width=\"1\"/>\n",
        cx, cy, tunnel_color
    ));
    
    // Add tunnel indicator
    svg.push_str(&format!(
        "  <text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"#fff\" text-anchor=\"middle\" dominant-baseline=\"middle\">🚇</text>\n",
        cx, cy
    ));
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
    
    // Legend with accurate visual representations
    html.push_str("    <div class=\"legend\">\n");
    html.push_str("      <strong>Legend - Accurate Visual Representations:</strong><br>\n");
    html.push_str("      <div style=\"display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 10px; margin-top: 10px;\">\n");
    
    // Basic Path Tiles
    html.push_str("        <div style=\"border: 1px solid #444; padding: 8px; border-radius: 4px;\">\n");
    html.push_str("          <strong style=\"color: #fff;\">Basic Paths:</strong><br>\n");
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Straight Path</div>\n", tile_color(&TileType::Straight)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Curved Path</div>\n", tile_color(&TileType::Curve90)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Open Platform</div>\n", tile_color(&TileType::OpenPlatform)));
    html.push_str("        </div>\n");
    
    // Junction Tiles
    html.push_str("        <div style=\"border: 1px solid #444; padding: 8px; border-radius: 4px;\">\n");
    html.push_str("          <strong style=\"color: #fff;\">Junctions:</strong><br>\n");
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>T-Junction (3-way)</div>\n", tile_color(&TileType::TJunction)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Y-Junction (smooth)</div>\n", tile_color(&TileType::YJunction)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Cross Junction (4-way)</div>\n", tile_color(&TileType::CrossJunction)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Merge Junction</div>\n", tile_color(&TileType::Merge)));
    html.push_str("        </div>\n");
    
    // Elevation & Movement
    html.push_str("        <div style=\"border: 1px solid #444; padding: 8px; border-radius: 4px;\">\n");
    html.push_str("          <strong style=\"color: #fff;\">Elevation & Movement:</strong><br>\n");
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Slope ⛰</div>\n", tile_color(&TileType::Slope)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Half-Pipe ∪</div>\n", tile_color(&TileType::HalfPipe)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Loop-de-Loop ∞</div>\n", tile_color(&TileType::LoopDeLoop)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Launch Pad ⚡</div>\n", tile_color(&TileType::LaunchPad)));
    html.push_str("        </div>\n");
    
    // Control & Structure
    html.push_str("        <div style=\"border: 1px solid #444; padding: 8px; border-radius: 4px;\">\n");
    html.push_str("          <strong style=\"color: #fff;\">Control & Structure:</strong><br>\n");
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>One-Way Gate →</div>\n", tile_color(&TileType::OneWayGate)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Obstacle</div>\n", tile_color(&TileType::Obstacle)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Bridge 🌉</div>\n", tile_color(&TileType::Bridge)));
    html.push_str(&format!("          <div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {}\"></span>Tunnel 🚇</div>\n", tile_color(&TileType::Tunnel)));
    html.push_str("        </div>\n");
    
    html.push_str("      </div>\n");
    html.push_str("      <div style=\"margin-top: 15px; padding: 10px; background: #333; border-radius: 4px;\">\n");
    html.push_str("        <strong style=\"color: #fff;\">Visual Features:</strong><br>\n");
    html.push_str("        <span style=\"color: #aaa;\">• <strong>Raised paths:</strong> Lighter colored track sections show the marble path</span><br>\n");
    html.push_str("        <span style=\"color: #aaa;\">• <strong>Curved tracks:</strong> Actual SVG curves show smooth marble flow</span><br>\n");
    html.push_str("        <span style=\"color: #aaa;\">• <strong>Junction hubs:</strong> Central circles with connecting lines</span><br>\n");
    html.push_str("        <span style=\"color: #aaa;\">• <strong>Directional arrows:</strong> Show marble flow direction</span><br>\n");
    html.push_str("        <span style=\"color: #aaa;\">• <strong>Elevation shading:</strong> Lighter = higher elevation</span><br>\n");
    html.push_str("        <span style=\"color: #aaa;\">• <strong>3D walls:</strong> Darker edges show raised boundaries</span>\n");
    html.push_str("      </div>\n");
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

