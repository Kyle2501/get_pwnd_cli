use clap::{Parser, Subcommand};
use petgraph::algo::dijkstra;
use petgraph::graph::{NodeIndex, UnGraph};
use redb::{Database, TableDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- 1. Data Structures ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistentGraphData {
    pub nodes: Vec<String>,
    pub edges: Vec<(u32, u32, f32)>, // (source_index, target_index, weight)
    pub node_indices: HashMap<String, u32>,
}

pub struct TrailNetwork {
    pub graph: UnGraph<String, f32>,
    pub node_indices: HashMap<String, NodeIndex>,
}

impl From<PersistentGraphData> for TrailNetwork {
    fn from(data: PersistentGraphData) -> Self {
        let mut graph = UnGraph::default();
        let mut node_indices = HashMap::new();

        // Recreate nodes
        let mut indices = Vec::new();
        for name in data.nodes {
            let idx = graph.add_node(name.clone());
            node_indices.insert(name, idx);
            indices.push(idx);
        }

        // Recreate edges
        for (u, v, weight) in data.edges {
            let u_idx = indices[u as usize];
            let v_idx = indices[v as usize];
            graph.update_edge(u_idx, v_idx, weight);
        }

        Self { graph, node_indices }
    }
}

impl TrailNetwork {
    pub fn find_shortest_path(&self, start_name: &str, end_name: &str) -> Option<(f32, Vec<String>)> {
        let start_idx = *self.node_indices.get(start_name)?;
        let end_idx = *self.node_indices.get(end_name)?;

        let (distances, predecessors) = dijkstra(&self.graph, start_idx, Some(end_idx), |e| *e.weight());
        let _target_distance = *distances.get(&end_idx)?;

        let mut path = Vec::new();
        let mut curr = end_idx;

        while curr != start_idx {
            path.push(self.graph[curr].clone());
            if let Some(&prev) = predecessors.get(&curr) {
                curr = prev;
            } else {
                break;
            }
        }
        path.push(self.graph[start_idx].clone());
        path.reverse();

        Some((distances[&end_idx], path))
    }
}

// --- 2. Database Layer ---

const TRAIL_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("trail_graph_store");

pub fn load_graph_from_redb(db_path: &str) -> Result<Option<PersistentGraphData>, Box<dyn std::error::Error>> {
    let db = Database::open(db_path)?;
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TRAIL_TABLE)?;

    if let Some(bytes) = table.get("adam_gulch_network")? {
        let decoded: PersistentGraphData = bincode::deserialize(bytes.value())?;
        Ok(Some(decoded))
    } else {
        Ok(None)
    }
}

// --- 3. CLI Definitions ---

#[derive(Parser, Debug)]
#[command(name = "ejected-trail-cli", version = "6.5", about = "Offline trail router")]
pub struct Cli {
    #[arg(short, long, default_value = "trails_cache.redb")]
    pub db: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Search { query: String },
    Route { start: String, end: String, loop_mode: bool },
    Stats,
}

// --- 4. Main Entrypoint & Wiring ---

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load persistent data from redb cache
    let persistent_data = match load_graph_from_redb(&cli.db) {
        Ok(Some(data)) => data,
        Ok(None) => {
            eprintln!("Error: No trail network found in database '{}'. Please initialize the cache.", cli.db);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            std::process::exit(1);
        }
    };

    // Convert persistent snapshot into live petgraph network
    let network = TrailNetwork::from(persistent_data);

    match cli.command {
        Commands::Search { query } => {
            println!("Searching offline database for: '{}'", query);
            let matches: Vec<&String> = network.node_indices.keys()
                .filter(|name| name.to_lowercase().contains(&query.to_lowercase()))
                .collect();
            
            if matches.is_empty() {
                println!("No matching trail junctions found.");
            } else {
                println!("Found matches:");
                for m in matches {
                    println!("  - {}", m);
                }
            }
        }
        Commands::Route { start, end, loop_mode } => {
            println!("Calculating path from '{}' to '{}' (Loop Mode: {})", start, end, loop_mode);
            
            match network.find_shortest_path(&start, &end) {
                Some((distance, path)) => {
                    println!("\nRoute Found!");
                    println!("Total Distance: {:.2} miles", distance);
                    println!("Path Junctions:");
                    for (i, step) in path.iter().enumerate() {
                        println!("  {}. {}", i + 1, step);
                    }
                }
                None => {
                    println!("Could not find a valid path between '{}' and '{}'.", start, end);
                }
            }
        }
        Commands::Stats => {
            println!("--- Network Statistics ---");
            println!("Total Junctions / Nodes: {}", network.graph.node_count());
            println!("Total Trail Segments / Edges: {}", network.graph.edge_count());
        }
    }

    Ok(())
}
