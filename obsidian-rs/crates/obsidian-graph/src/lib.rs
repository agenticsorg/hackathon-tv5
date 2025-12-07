//! Knowledge graph and backlinks for Obsidian-rs
//!
//! This crate provides:
//! - Note linking graph
//! - Backlinks tracking
//! - Graph visualization data
//! - Link resolution

pub mod error;
pub mod graph;
pub mod link;
pub mod visualization;

pub use error::{GraphError, GraphResult};
pub use graph::KnowledgeGraph;
pub use link::{Link, LinkResolver, LinkType};
pub use visualization::{GraphData, GraphNode, GraphEdge};

/// Prelude for common imports
pub mod prelude {
    pub use crate::error::{GraphError, GraphResult};
    pub use crate::graph::KnowledgeGraph;
    pub use crate::link::{Link, LinkResolver, LinkType};
    pub use crate::visualization::{GraphData, GraphEdge, GraphNode};
}
