//! Graph visualization data structures

use crate::link::LinkType;
use serde::{Deserialize, Serialize};

/// A node in the visualization graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique ID (path)
    pub id: String,
    /// Display name
    pub name: String,
    /// Whether the node exists
    pub exists: bool,
    /// Tags
    pub tags: Vec<String>,
    /// Number of incoming links
    pub backlink_count: usize,
    /// Number of outgoing links
    pub link_count: usize,
    /// X position (for pre-computed layouts)
    pub x: Option<f64>,
    /// Y position (for pre-computed layouts)
    pub y: Option<f64>,
    /// Node color
    pub color: Option<String>,
    /// Node size
    pub size: Option<f64>,
}

impl GraphNode {
    /// Create a new graph node
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            exists: true,
            tags: Vec::new(),
            backlink_count: 0,
            link_count: 0,
            x: None,
            y: None,
            color: None,
            size: None,
        }
    }

    /// Set whether node exists
    pub fn with_exists(mut self, exists: bool) -> Self {
        self.exists = exists;
        self
    }

    /// Set tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set position
    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    /// Set color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set size
    pub fn with_size(mut self, size: f64) -> Self {
        self.size = Some(size);
        self
    }

    /// Set link counts
    pub fn with_counts(mut self, link_count: usize, backlink_count: usize) -> Self {
        self.link_count = link_count;
        self.backlink_count = backlink_count;
        self
    }
}

/// An edge in the visualization graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Link type
    pub link_type: LinkType,
    /// Edge color
    pub color: Option<String>,
    /// Edge weight/width
    pub weight: Option<f64>,
}

impl GraphEdge {
    /// Create a new graph edge
    pub fn new(source: impl Into<String>, target: impl Into<String>, link_type: LinkType) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            link_type,
            color: None,
            weight: None,
        }
    }

    /// Set color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }
}

/// Complete graph data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    /// All nodes
    pub nodes: Vec<GraphNode>,
    /// All edges
    pub edges: Vec<GraphEdge>,
    /// Graph metadata
    pub metadata: GraphMetadata,
}

impl GraphData {
    /// Create new graph data
    pub fn new(nodes: Vec<GraphNode>, edges: Vec<GraphEdge>) -> Self {
        let metadata = GraphMetadata {
            node_count: nodes.len(),
            edge_count: edges.len(),
            ..Default::default()
        };

        Self {
            nodes,
            edges,
            metadata,
        }
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: GraphMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Calculate node sizes based on link counts
    pub fn calculate_sizes(&mut self) {
        // Count links for each node
        let mut link_counts: std::collections::HashMap<String, (usize, usize)> =
            std::collections::HashMap::new();

        for edge in &self.edges {
            link_counts.entry(edge.source.clone()).or_insert((0, 0)).0 += 1;
            link_counts.entry(edge.target.clone()).or_insert((0, 0)).1 += 1;
        }

        // Set sizes based on total connections
        for node in &mut self.nodes {
            let (links, backlinks) = link_counts.get(&node.id).copied().unwrap_or((0, 0));
            node.link_count = links;
            node.backlink_count = backlinks;

            let total = links + backlinks;
            // Size between 5 and 30 based on connections
            node.size = Some(5.0 + (total as f64).sqrt() * 3.0);
        }
    }

    /// Apply a simple force-directed layout
    pub fn apply_layout(&mut self, iterations: usize) {
        if self.nodes.is_empty() {
            return;
        }

        // Initialize positions randomly
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let angle = (i as f64) * 2.0 * std::f64::consts::PI / (self.nodes.len() as f64);
            node.x = Some(angle.cos() * 100.0);
            node.y = Some(angle.sin() * 100.0);
        }

        // Build adjacency for quick lookup
        let node_idx: std::collections::HashMap<String, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.clone(), i))
            .collect();

        // Force-directed iterations
        for _ in 0..iterations {
            let positions: Vec<(f64, f64)> = self
                .nodes
                .iter()
                .map(|n| (n.x.unwrap_or(0.0), n.y.unwrap_or(0.0)))
                .collect();

            let mut forces: Vec<(f64, f64)> = vec![(0.0, 0.0); self.nodes.len()];

            // Repulsive forces between all nodes
            let repulsion = 1000.0;
            for i in 0..self.nodes.len() {
                for j in (i + 1)..self.nodes.len() {
                    let dx = positions[j].0 - positions[i].0;
                    let dy = positions[j].1 - positions[i].1;
                    let dist_sq = dx * dx + dy * dy + 0.1;
                    let dist = dist_sq.sqrt();

                    let force = repulsion / dist_sq;
                    let fx = force * dx / dist;
                    let fy = force * dy / dist;

                    forces[i].0 -= fx;
                    forces[i].1 -= fy;
                    forces[j].0 += fx;
                    forces[j].1 += fy;
                }
            }

            // Attractive forces for edges
            let attraction = 0.01;
            for edge in &self.edges {
                if let (Some(&i), Some(&j)) = (node_idx.get(&edge.source), node_idx.get(&edge.target)) {
                    let dx = positions[j].0 - positions[i].0;
                    let dy = positions[j].1 - positions[i].1;
                    let dist = (dx * dx + dy * dy).sqrt() + 0.1;

                    let force = attraction * dist;
                    let fx = force * dx / dist;
                    let fy = force * dy / dist;

                    forces[i].0 += fx;
                    forces[i].1 += fy;
                    forces[j].0 -= fx;
                    forces[j].1 -= fy;
                }
            }

            // Apply forces with damping
            let damping = 0.8;
            for (i, node) in self.nodes.iter_mut().enumerate() {
                let x = node.x.unwrap_or(0.0) + forces[i].0 * damping;
                let y = node.y.unwrap_or(0.0) + forces[i].1 * damping;
                node.x = Some(x);
                node.y = Some(y);
            }
        }
    }

    /// Get nodes filtered by a predicate
    pub fn filter_nodes<F>(&self, predicate: F) -> Vec<&GraphNode>
    where
        F: Fn(&GraphNode) -> bool,
    {
        self.nodes.iter().filter(|n| predicate(n)).collect()
    }

    /// Get orphan nodes (no connections)
    pub fn get_orphans(&self) -> Vec<&GraphNode> {
        let connected: std::collections::HashSet<&String> = self
            .edges
            .iter()
            .flat_map(|e| vec![&e.source, &e.target])
            .collect();

        self.nodes
            .iter()
            .filter(|n| !connected.contains(&n.id))
            .collect()
    }
}

/// Graph metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Total node count
    pub node_count: usize,
    /// Total edge count
    pub edge_count: usize,
    /// Graph title
    pub title: Option<String>,
    /// Center node (for local graphs)
    pub center: Option<String>,
    /// Depth (for local graphs)
    pub depth: Option<usize>,
}

impl GraphMetadata {
    /// Create new metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Set title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set center
    pub fn with_center(mut self, center: impl Into<String>) -> Self {
        self.center = Some(center.into());
        self
    }

    /// Set depth
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_node() {
        let node = GraphNode::new("test.md", "Test")
            .with_exists(true)
            .with_tags(vec!["tag1".to_string()])
            .with_position(10.0, 20.0)
            .with_color("#ff0000")
            .with_size(15.0);

        assert_eq!(node.id, "test.md");
        assert_eq!(node.x, Some(10.0));
        assert_eq!(node.color, Some("#ff0000".to_string()));
    }

    #[test]
    fn test_graph_data_calculate_sizes() {
        let nodes = vec![
            GraphNode::new("a.md", "A"),
            GraphNode::new("b.md", "B"),
            GraphNode::new("c.md", "C"),
        ];

        let edges = vec![
            GraphEdge::new("a.md", "b.md", LinkType::Wiki),
            GraphEdge::new("a.md", "c.md", LinkType::Wiki),
            GraphEdge::new("b.md", "c.md", LinkType::Wiki),
        ];

        let mut data = GraphData::new(nodes, edges);
        data.calculate_sizes();

        // Node A has 2 outgoing links
        let node_a = data.nodes.iter().find(|n| n.id == "a.md").unwrap();
        assert_eq!(node_a.link_count, 2);
        assert_eq!(node_a.backlink_count, 0);

        // Node C has 2 incoming links
        let node_c = data.nodes.iter().find(|n| n.id == "c.md").unwrap();
        assert_eq!(node_c.link_count, 0);
        assert_eq!(node_c.backlink_count, 2);
    }

    #[test]
    fn test_get_orphans() {
        let nodes = vec![
            GraphNode::new("a.md", "A"),
            GraphNode::new("b.md", "B"),
            GraphNode::new("orphan.md", "Orphan"),
        ];

        let edges = vec![GraphEdge::new("a.md", "b.md", LinkType::Wiki)];

        let data = GraphData::new(nodes, edges);
        let orphans = data.get_orphans();

        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].id, "orphan.md");
    }

    #[test]
    fn test_apply_layout() {
        let nodes = vec![
            GraphNode::new("a.md", "A"),
            GraphNode::new("b.md", "B"),
            GraphNode::new("c.md", "C"),
        ];

        let edges = vec![
            GraphEdge::new("a.md", "b.md", LinkType::Wiki),
            GraphEdge::new("b.md", "c.md", LinkType::Wiki),
        ];

        let mut data = GraphData::new(nodes, edges);
        data.apply_layout(10);

        // Check all nodes have positions
        for node in &data.nodes {
            assert!(node.x.is_some());
            assert!(node.y.is_some());
        }
    }
}
