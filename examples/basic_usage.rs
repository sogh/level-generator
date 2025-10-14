use level_generator::{generate, GeneratorParams, to_ascii};

fn main() {
    // Classic dungeon with default settings
    let params = GeneratorParams {
        seed: Some(42),
        ..Default::default()
    };
    
    let level = generate(&params);
    println!("Classic Dungeon ({}x{}):", level.width, level.height);
    println!("{}", to_ascii(&level));
    println!("\nRooms: {}", level.rooms.len());
}
