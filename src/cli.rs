use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum ModeArg {
    Classic,
    Marble,
    Wfc,
}

impl std::str::FromStr for ModeArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "classic" | "dungeon" => Ok(ModeArg::Classic),
            "marble" | "marbles" => Ok(ModeArg::Marble),
            "wfc" | "wave" => Ok(ModeArg::Wfc),
            other => Err(format!("invalid mode: {} (expected classic|marble)", other)),
        }
    }
}

/// Command-line arguments for the level generator.
#[derive(Debug, Parser, Clone)]
#[command(name = "level-generator", version, about = "Roguelike dungeon level generator")] 
pub struct Args {
    /// Overall map width in tiles
    #[arg(long = "width", short = 'w', default_value_t = 80, help = "Overall map width in tiles")] 
    pub width: u32,

    // Note: avoid -h because it's reserved for help
    /// Overall map height in tiles
    #[arg(long = "height", short = 'H', default_value_t = 25, help = "Overall map height in tiles")] 
    pub height: u32,

    /// Target number of rooms to attempt placing
    #[arg(long = "rooms", short = 'r', default_value_t = 12, help = "Target number of rooms")] 
    pub rooms: u32,

    /// Minimum room dimension (width and height)
    #[arg(long = "min-room", short = 'm', default_value_t = 4, help = "Minimum room dimension")] 
    pub min_room: u32,

    /// Maximum room dimension (width and height)
    #[arg(long = "max-room", short = 'M', default_value_t = 10, help = "Maximum room dimension")] 
    pub max_room: u32,

    /// RNG seed for reproducible dungeons
    #[arg(long = "seed", short = 's', help = "RNG seed for reproducible dungeons")] 
    pub seed: Option<u64>,

    /// Generation mode: classic (rooms+tunnels) or marble (rounded channels)
    #[arg(long = "mode", default_value = "classic", help = "Generation mode: classic|marble")] 
    pub mode: ModeArg,

    /// Marble: channel width in tiles (ignored for classic)
    #[arg(long = "channel-width", default_value_t = 2, help = "Marble: channel width in tiles")] 
    pub channel_width: u32,

    /// Marble: corner radius in tiles for rounded turns (ignored for classic)
    #[arg(long = "corner-radius", default_value_t = 2, help = "Marble: corner radius in tiles")] 
    pub corner_radius: u32,

    /// File path to write the generated level as JSON
    #[arg(long = "json-path", short = 'o', help = "Write level to JSON file path")] 
    pub json_path: Option<PathBuf>,

    /// Also print JSON to stdout
    #[arg(long = "print-json", default_value_t = false, help = "Print JSON to stdout")] 
    pub print_json: bool,

    /// Disable ASCII preview in stdout
    #[arg(long = "no-ascii", default_value_t = false, help = "Disable ASCII preview")] 
    pub no_ascii: bool,
}


