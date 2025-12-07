//! Knowledge graph using petgraph

use crate::error::{GraphError, GraphResult};
use crate::link::{Link, LinkResolver, LinkType};
use crate::visualization::{GraphData, GraphEdge, GraphNode};
use indexmap::IndexMap;
use parking_lot::RwLock;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Node data in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteNode {
    /// Note path
    pub path: String,
    /// Note title
    pub title: String,
    /// Tags
    pub tags: Vec<String>,
    /// Whether the note exists (resolved)
    pub exists: bool,
}

impl NoteNode {
    /// Create a new note node
    pub fn new(path: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            title: title.into(),
            tags: Vec::new(),
            exists: true,
        }
    }

    /// Create an unresolved note node
    pub fn unresolved(target: impl Into<String>) -> Self {
        let target = target.into();
        Self {
            path: target.clone(),
            title: target,
            tags: Vec::new(),
            exists: false,
        }
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Edge data in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEdge {
    /// Link type
    pub link_type: LinkType,
    /// Display text
    pub display: Option<String>,
    /// Line number in source
    pub line: Option<usize>,
}

impl LinkEdge {
    /// Create a new link edge
    pub fn new(link_type: LinkType) -> Self {
        Self {
            link_type,
            display: None,
            line: None,
        }
    }

    /// Set display text
    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display = Some(display.into());
        self
    }

    /// Set line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

/// Knowledge graph for notes
pub struct KnowledgeGraph {
    /// The graph
    graph: RwLock<DiGraph<NoteNode, LinkEdge>>,
    /// Path to node index mapping
    node_indices: RwLock<HashMap<String, NodeIndex>>,
    /// Link resolver
    resolver: LinkResolver,
}

impl KnowledgeGraph {
    /// Create a new knowledge graph
    pub fn new(vault_path: impl Into<PathBuf>) -> Self {
        Self {
            graph: RwLock::new(DiGraph::new()),
            node_indices: RwLock::new(HashMap::new()),
            resolver: LinkResolver::new(vault_path),
        }
    }

    /// Get the link resolver
    pub fn resolver(&self) -> &LinkResolver {
        &self.resolver
    }

    /// Add a note to the graph
    pub fn add_note(&self, node: NoteNode) -> NodeIndex {
        let path = node.path.clone();
        let path_lower = path.to_lowercase();

        // Check if already exists
        {
            let indices = self.node_indices.read();
            if let Some(&idx) = indices.get(&path_lower) {
                // Update existing node
                let mut graph = self.graph.write();
                if let Some(existing) = graph.node_weight_mut(idx) {
                    *existing = node;
                }
                return idx;
            }
        }

        // Add new node
        let idx = {
            let mut graph = self.graph.write();
            graph.add_node(node)
        };

        // Register in index
        {
            let mut indices = self.node_indices.write();
            indices.insert(path_lower, idx);
        }

        idx
    }

    /// Remove a note from the graph
    pub fn remove_note(&self, path: &str) -> GraphResult<()> {
        let path_lower = path.to_lowercase();

        let idx = {
            let indices = self.node_indices.read();
            indices.get(&path_lower).copied()
        };

        if let Some(idx) = idx {
            // Remove all edges connected to this node
            {
                let mut graph = self.graph.write();
                let edges_to_remove: Vec<_> = graph
                    .edges_directed(idx, Direction::Incoming)
                    .chain(graph.edges_directed(idx, Direction::Outgoing))
                    .map(|e| e.id())
                    .collect();

                for edge in edges_to_remove {
                    graph.remove_edge(edge);
                }

                // Remove the node
                graph.remove_node(idx);
            }

            // Remove from index
            {
                let mut indices = self.node_indices.write();
                indices.remove(&path_lower);
            }

            Ok(())
        } else {
            Err(GraphError::NodeNotFound(path.to_string()))
        }
    }

    /// Add a link to the graph
    pub fn add_link(&self, link: &Link) -> GraphResult<()> {
        let source_lower = link.source.to_lowercase();
        let target_lower = link.target.to_lowercase();

        // Get or create source node
        let source_idx = {
            let indices = self.node_indices.read();
            if let Some(&idx) = indices.get(&source_lower) {
                idx
            } else {
                drop(indices);
                // Create node for source
                let node = NoteNode::new(&link.source, &link.source);
                self.add_note(node)
            }
        };

        // Get or create target node
        let target_idx = {
            let indices = self.node_indices.read();
            if let Some(&idx) = indices.get(&target_lower) {
                idx
            } else {
                drop(indices);
                // Create unresolved node for target
                let node = NoteNode::unresolved(&link.target);
                self.add_note(node)
            }
        };

        // Add edge
        let edge = LinkEdge::new(link.link_type);
        let edge = if let Some(ref display) = link.display {
            edge.with_display(display)
        } else {
            edge
        };
        let edge = if let Some(line) = link.line {
            edge.with_line(line)
        } else {
            edge
        };

        {
            let mut graph = self.graph.write();
            // Check if edge already exists
            let exists = graph
                .edges_connecting(source_idx, target_idx)
                .any(|_| true);

            if !exists {
                graph.add_edge(source_idx, target_idx, edge);
            }
        }

        Ok(())
    }

    /// Remove all outgoing links from a note
    pub fn remove_links_from(&self, source: &str) -> GraphResult<()> {
        let source_lower = source.to_lowercase();

        let source_idx = {
            let indices = self.node_indices.read();
            indices.get(&source_lower).copied()
        };

        if let Some(idx) = source_idx {
            let mut graph = self.graph.write();
            let edges_to_remove: Vec<_> = graph
                .edges_directed(idx, Direction::Outgoing)
                .map(|e| e.id())
                .collect();

            for edge in edges_to_remove {
                graph.remove_edge(edge);
            }

            Ok(())
        } else {
            Ok(()) // No error if source doesn't exist
        }
    }

    /// Get all outgoing links from a note
    pub fn get_links(&self, source: &str) -> Vec<String> {
        let source_lower = source.to_lowercase();
        let graph = self.graph.read();
        let indices = self.node_indices.read();

        if let Some(&idx) = indices.get(&source_lower) {
            graph
                .edges_directed(idx, Direction::Outgoing)
                .filter_map(|edge| {
                    graph
                        .node_weight(edge.target())
                        .map(|node| node.path.clone())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all backlinks to a note
    pub fn get_backlinks(&self, target: &str) -> Vec<String> {
        let target_lower = target.to_lowercase();
        let graph = self.graph.read();
        let indices = self.node_indices.read();

        if let Some(&idx) = indices.get(&target_lower) {
            graph
                .edges_directed(idx, Direction::Incoming)
                .filter_map(|edge| {
                    graph
                        .node_weight(edge.source())
                        .map(|node| node.path.clone())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all unresolved links
    pub fn get_unresolved(&self) -> Vec<String> {
        let graph = self.graph.read();

        graph
            .node_weights()
            .filter(|node| !node.exists)
            .map(|node| node.path.clone())
            .collect()
    }

    /// Get notes with a specific tag
    pub fn get_notes_with_tag(&self, tag: &str) -> Vec<String> {
        let tag_lower = tag.to_lowercase();
        let graph = self.graph.read();

        graph
            .node_weights()
            .filter(|node| {
                node.tags
                    .iter()
                    .any(|t| t.to_lowercase() == tag_lower)
            })
            .map(|node| node.path.clone())
            .collect()
    }

    /// Get all tags in the graph
    pub fn get_all_tags(&self) -> Vec<String> {
        let graph = self.graph.read();
        let mut tags: HashSet<String> = HashSet::new();

        for node in graph.node_weights() {
            for tag in &node.tags {
                tags.insert(tag.clone());
            }
        }

        tags.into_iter().collect()
    }

    /// Get the local graph around a note
    pub fn get_local_graph(&self, center: &str, depth: usize) -> GraphData {
        let center_lower = center.to_lowercase();
        let graph = self.graph.read();
        let indices = self.node_indices.read();

        let mut nodes: IndexMap<String, GraphNode> = IndexMap::new();
        let mut edges: Vec<GraphEdge> = Vec::new();
        let mut visited: HashSet<NodeIndex> = HashSet::new();

        if let Some(&center_idx) = indices.get(&center_lower) {
            self.collect_neighbors(&graph, center_idx, depth, &mut visited, &mut nodes, &mut edges);
        }

        GraphData::new(nodes.into_values().collect(), edges)
    }

    /// Collect neighbors recursively
    fn collect_neighbors(
        &self,
        graph: &DiGraph<NoteNode, LinkEdge>,
        idx: NodeIndex,
        depth: usize,
        visited: &mut HashSet<NodeIndex>,
        nodes: &mut IndexMap<String, GraphNode>,
        edges: &mut Vec<GraphEdge>,
    ) {
        if visited.contains(&idx) || depth == 0 {
            return;
        }
        visited.insert(idx);

        // Add current node
        if let Some(node) = graph.node_weight(idx) {
            let graph_node = GraphNode::new(&node.path, &node.title)
                .with_exists(node.exists)
                .with_tags(node.tags.clone());

            nodes.insert(node.path.clone(), graph_node);
        }

        // Collect outgoing edges
        for edge in graph.edges_directed(idx, Direction::Outgoing) {
            if let (Some(source), Some(target)) = (
                graph.node_weight(idx),
                graph.node_weight(edge.target()),
            ) {
                edges.push(GraphEdge::new(
                    &source.path,
                    &target.path,
                    edge.weight().link_type,
                ));

                if depth > 1 {
                    self.collect_neighbors(graph, edge.target(), depth - 1, visited, nodes, edges);
                }
            }
        }

        // Collect incoming edges
        for edge in graph.edges_directed(idx, Direction::Incoming) {
            if let (Some(source), Some(target)) = (
                graph.node_weight(edge.source()),
                graph.node_weight(idx),
            ) {
                edges.push(GraphEdge::new(
                    &source.path,
                    &target.path,
                    edge.weight().link_type,
                ));

                if depth > 1 {
                    self.collect_neighbors(graph, edge.source(), depth - 1, visited, nodes, edges);
                }
            }
        }
    }

    /// Get the full graph data
    pub fn get_full_graph(&self) -> GraphData {
        let graph = self.graph.read();

        let nodes: Vec<GraphNode> = graph
            .node_weights()
            .map(|node| {
                GraphNode::new(&node.path, &node.title)
                    .with_exists(node.exists)
                    .with_tags(node.tags.clone())
            })
            .collect();

        let indices = self.node_indices.read();
        let path_to_idx: HashMap<&str, NodeIndex> = indices
            .iter()
            .map(|(k, &v)| (k.as_str(), v))
            .collect();

        let edges: Vec<GraphEdge> = graph
            .edge_references()
            .filter_map(|edge| {
                let source = graph.node_weight(edge.source())?;
                let target = graph.node_weight(edge.target())?;
                Some(GraphEdge::new(
                    &source.path,
                    &target.path,
                    edge.weight().link_type,
                ))
            })
            .collect();

        GraphData::new(nodes, edges)
    }

    /// Get statistics about the graph
    pub fn stats(&self) -> GraphStats {
        let graph = self.graph.read();

        let node_count = graph.node_count();
        let edge_count = graph.edge_count();
        let resolved_count = graph.node_weights().filter(|n| n.exists).count();
        let unresolved_count = node_count - resolved_count;

        GraphStats {
            node_count,
            edge_count,
            resolved_count,
            unresolved_count,
        }
    }
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    /// Number of nodes
    pub node_count: usize,
    /// Number of edges
    pub edge_count: usize,
    /// Number of resolved notes
    pub resolved_count: usize,
    /// Number of unresolved links
    pub unresolved_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_add_note() {
        let dir = tempdir().unwrap();
        let graph = KnowledgeGraph::new(dir.path());

        let node = NoteNode::new("test.md", "Test Note");
        graph.add_note(node);

        let stats = graph.stats();
        assert_eq!(stats.node_count, 1);
    }

    #[test]
    fn test_add_link() {
        let dir = tempdir().unwrap();
        let graph = KnowledgeGraph::new(dir.path());

        graph.add_note(NoteNode::new("source.md", "Source"));
        graph.add_note(NoteNode::new("target.md", "Target"));

        let link = Link::new("source.md", "target.md", LinkType::Wiki);
        graph.add_link(&link).unwrap();

        let links = graph.get_links("source.md");
        assert_eq!(links.len(), 1);
        assert!(links.contains(&"target.md".to_string()));
    }

    #[test]
    fn test_backlinks() {
        let dir = tempdir().unwrap();
        let graph = KnowledgeGraph::new(dir.path());

        graph.add_note(NoteNode::new("a.md", "A"));
        graph.add_note(NoteNode::new("b.md", "B"));
        graph.add_note(NoteNode::new("c.md", "C"));

        graph.add_link(&Link::new("a.md", "c.md", LinkType::Wiki)).unwrap();
        graph.add_link(&Link::new("b.md", "c.md", LinkType::Wiki)).unwrap();

        let backlinks = graph.get_backlinks("c.md");
        assert_eq!(backlinks.len(), 2);
        assert!(backlinks.contains(&"a.md".to_string()));
        assert!(backlinks.contains(&"b.md".to_string()));
    }

    #[test]
    fn test_unresolved_links() {
        let dir = tempdir().unwrap();
        let graph = KnowledgeGraph::new(dir.path());

        graph.add_note(NoteNode::new("source.md", "Source"));

        // Link to non-existent note
        let link = Link::new("source.md", "nonexistent.md", LinkType::Wiki);
        graph.add_link(&link).unwrap();

        let unresolved = graph.get_unresolved();
        assert_eq!(unresolved.len(), 1);
        assert!(unresolved.contains(&"nonexistent.md".to_string()));
    }

    #[test]
    fn test_local_graph() {
        let dir = tempdir().unwrap();
        let graph = KnowledgeGraph::new(dir.path());

        graph.add_note(NoteNode::new("a.md", "A"));
        graph.add_note(NoteNode::new("b.md", "B"));
        graph.add_note(NoteNode::new("c.md", "C"));
        graph.add_note(NoteNode::new("d.md", "D"));

        graph.add_link(&Link::new("a.md", "b.md", LinkType::Wiki)).unwrap();
        graph.add_link(&Link::new("b.md", "c.md", LinkType::Wiki)).unwrap();
        graph.add_link(&Link::new("c.md", "d.md", LinkType::Wiki)).unwrap();

        // Local graph with depth 2 around B
        let local = graph.get_local_graph("b.md", 2);
        assert!(local.nodes.len() >= 3); // A, B, C at minimum
    }

    #[test]
    fn test_tags() {
        let dir = tempdir().unwrap();
        let graph = KnowledgeGraph::new(dir.path());

        graph.add_note(NoteNode::new("a.md", "A").with_tags(vec!["rust".to_string(), "programming".to_string()]));
        graph.add_note(NoteNode::new("b.md", "B").with_tags(vec!["rust".to_string()]));
        graph.add_note(NoteNode::new("c.md", "C").with_tags(vec!["python".to_string()]));

        let rust_notes = graph.get_notes_with_tag("rust");
        assert_eq!(rust_notes.len(), 2);

        let all_tags = graph.get_all_tags();
        assert!(all_tags.contains(&"rust".to_string()));
        assert!(all_tags.contains(&"programming".to_string()));
        assert!(all_tags.contains(&"python".to_string()));
    }
}
