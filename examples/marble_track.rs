use level_generator::{generate, GeneratorParams, GenerationMode};
use std::fs;

fn main() {
    // Marble track with elevation and obstacles
    let params = GeneratorParams {
        width: 60,
        height: 40,
        rooms: 10,
        mode: GenerationMode::Marble,
        channel_width: 3,
        corner_radius: 2,
        enable_elevation: true,
        max_elevation: 3,
        enable_obstacles: true,
        obstacle_density: 0.4,
        seed: Some(12345),
        ..Default::default()
    };
    
    let level = generate(&params);
    
    // Export to JSON
    let json = serde_json::to_string_pretty(&level).unwrap();
    fs::write("marble_level.json", json).expect("Failed to write JSON");
    
    // Generate HTML visualization
    let html = level_generator::generate_html(&level);
    fs::write("marble_level.html", html).expect("Failed to write HTML");
    
    println!("Generated marble track with {} rooms", level.rooms.len());
    println!("Outputs: marble_level.json, marble_level.html");
}
