use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "ejected-trail-cli",
    author = "Ejected Media",
    version = "6.5",
    about = "Offline trail router and telemetry query engine for local-first systems"
)]
pub struct Cli {
    #[arg(short, long, default_value = "trails_cache.redb")]
    pub db: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for a trail by name or keyword
    Search {
        #[arg(short, long)]
        query: String,
    },
    /// Calculate a route or loop between two points
    Route {
        #[arg(short, long)]
        start: String,
        #[arg(short, long)]
        end: String,
        #[arg(short, long, default_value_t = false)]
        loop_mode: bool,
    },
    /// Display aggregate statistics for the loaded trail network
    Stats,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("Using database cache at: {}", cli.db);

    match cli.command {
        Commands::Search { query } => {
            println!("Searching offline database for trail matching: '{}'", query);
            // TODO: Query redb store for matching trail segments
        }
        Commands::Route { start, end, loop_mode } => {
            println!("Calculating path from '{}' to '{}' (Loop mode: {})", start, end, loop_mode);
            // TODO: Execute petgraph Dijkstra routing function from previous step
        }
        Commands::Stats => {
            println!("Fetching network summary statistics from local redb cache...");
            // TODO: Display total miles, node count, and loop classifications
        }
    }

    Ok(())
}
