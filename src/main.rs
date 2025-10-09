use clap::Parser;
use std::fs;
use std::path::Path;

use level_generator::cli::Args;
use level_generator::cli::ModeArg;
use level_generator::dungeon::{generate, GenerationMode, GeneratorParams};
use level_generator::isometric;
use level_generator::visualize::to_ascii;

fn main() {
    let args = Args::parse();

    let params = GeneratorParams {
        width: args.width,
        height: args.height,
        rooms: args.rooms,
        min_room: args.min_room,
        max_room: args.max_room,
        seed: args.seed,
        mode: match args.mode {
            ModeArg::Classic => GenerationMode::Classic,
            ModeArg::Marble => GenerationMode::Marble,
            ModeArg::Wfc => GenerationMode::Wfc,
        },
        channel_width: args.channel_width,
        corner_radius: args.corner_radius,
        enable_elevation: args.enable_elevation,
        max_elevation: args.max_elevation,
        enable_obstacles: args.enable_obstacles,
        obstacle_density: args.obstacle_density,
    };

    let level = generate(&params);

    // ASCII output
    if !args.no_ascii && !args.html_only {
        let ascii = to_ascii(&level);
        println!("{}", ascii);
    }

    // JSON output
    if !args.html_only {
        let json = serde_json::to_string_pretty(&level).expect("serialize level");
        if args.print_json {
            println!("{}", json);
        }
        if let Some(path) = args.json_path.as_ref() {
            let p: &Path = path.as_path();
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = fs::create_dir_all(parent);
                }
            }
            fs::write(p, json).expect("write json file");
        }
    }

    // HTML isometric visualization
    if let Some(html_path) = args.html_path.as_ref() {
        let html = isometric::generate_html(&level);
        let p: &Path = html_path.as_path();
        if let Some(parent) = p.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = fs::create_dir_all(parent);
            }
        }
        fs::write(p, html).expect("write html file");
        println!("Isometric visualization written to: {}", html_path.display());
    }
}
