use level_generator::{generate, GeneratorParams, GenerationMode, TileType};

fn main() {
    // Generate a level and analyze it
    let params = GeneratorParams {
        width: 80,
        height: 40,
        rooms: 15,
        mode: GenerationMode::Marble,
        enable_elevation: true,
        enable_obstacles: true,
        ..Default::default()
    };
    
    let level = generate(&params);
    
    // Analyze the generated level
    if let Some(marble_tiles) = &level.marble_tiles {
        let mut tile_counts = std::collections::HashMap::new();
        
        for row in marble_tiles {
            for tile in row {
                *tile_counts.entry(format!("{:?}", tile.tile_type)).or_insert(0) += 1;
            }
        }
        
        println!("Tile Distribution:");
        for (tile_type, count) in tile_counts {
            println!("  {}: {}", tile_type, count);
        }
        
        // Find slopes
        let slope_count: usize = marble_tiles
            .iter()
            .flat_map(|row| row.iter())
            .filter(|t| t.tile_type == TileType::Slope)
            .count();
        
        println!("\nSlopes for elevation changes: {}", slope_count);
    }
}
