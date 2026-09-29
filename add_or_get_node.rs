use petgraph::algo::dijkstra;
use petgraph::graph::{NodeIndex, UnGraph};
use std::collections::HashMap;

// Define the trail network graph structure
pub struct TrailNetwork {
    // UnGraph<NodeName, EdgeWeight> where EdgeWeight is distance in miles
    pub graph: UnGraph<String, f32>,
    pub node_indices: HashMap<String, NodeIndex>,
}

impl TrailNetwork {
    pub fn new() -> Self {
        Self {
            graph: UnGraph::default(),
            node_indices: HashMap::new(),
        }
    }

    // Helper to add or retrieve a node index by name
    pub fn add_or_get_node(&mut self, name: &str) -> NodeIndex {
        if let Some(&idx) = self.node_indices.get(name) {
            idx
        } else {
            let idx = self.graph.add_node(name.to_string());
            self.node_indices.insert(name.to_string(), idx);
            idx
        }
    }

    // Add a trail segment between two junctions
    pub fn add_trail(&mut self, start: &str, end: &str, distance_miles: f32) {
        let u = self.add_or_get_node(start);
        let v = self.add_or_get_node(end);
        self.graph.update_edge(u, v, distance_miles);
    }

    // Compute shortest path using Dijkstra's algorithm
    pub fn find_shortest_path(&self, start_name: &str, end_name: &str) -> Option<(f32, Vec<String>)> {
        let start_idx = *self.node_indices.get(start_name)?;
        let end_idx = *self.node_indices.get(end_name)?;

        // run Dijkstra: returns a map of NodeIndex -> minimum cost (distance)
        // and parent pointers for path reconstruction
        let (distances, predecessors) = dijkstra(&self.graph, start_idx, Some(end_idx), |e| *e.weight());

        let target_distance = *distances.get(&end_idx)?;

        // Reconstruct path from predecessors
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

        Some((target_distance, path))
    }
}
