use redb::{Database, Error, TableDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Define the serializable snapshot of the graph data
#[derive(Serialize, Deserialize, Debug)]
pub struct PersistentGraphData {
    pub nodes: Vec<String>,
    pub edges: Vec<(u32, u32, f32)>, // (source_index, target_index, weight)
    pub node_indices: HashMap<String, u32>,
}

// Define the redb table schema
const TRAIL_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("trail_graph_store");

pub fn save_graph_to_redb(db_path: &str, data: &PersistentGraphData) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::create(db_path)?;
    let write_txn = db.begin_write()?;
    
    {
        let mut table = write_txn.open_table(TRAIL_TABLE)?;
        let encoded = bincode::serialize(data)?;
        table.insert("adam_gulch_network", encoded.as_slice())?;
    }
    
    write_txn.commit()?;
    Ok(())
}

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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_file = "trails_cache.redb";

    // Example data packet to save
    let sample_data = PersistentGraphData {
        nodes: vec![
            "Adams Gulch Trailhead (North)".to_string(),
            "Mid-Loop Intersection".to_string(),
        ],
        edges: vec![(0, 1, 3.2)],
        node_indices: HashMap::from([
            ("Adams Gulch Trailhead (North)".to_string(), 0),
            ("Mid-Loop Intersection".to_string(), 1),
        ]),
    };

    // Save to local redb store
    save_graph_to_redb(db_file, &sample_data)?;
    println!("Successfully persisted trail graph to local redb database.");

    // Load back for offline routing query
    if let Some(loaded_data) = load_graph_from_redb(db_file)? {
        println!("Loaded offline trail graph with {} nodes.", loaded_data.nodes.len());
    }

    Ok(())
}
