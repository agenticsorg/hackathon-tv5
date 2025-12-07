//! Graph commands

use crate::state::AppState;
use obsidian_graph::visualization::{GraphData, GraphEdge, GraphNode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::debug;

/// Graph data for frontend
#[derive(Debug, Serialize)]
pub struct GraphDataResponse {
    pub nodes: Vec<NodeResponse>,
    pub edges: Vec<EdgeResponse>,
}

/// Node for frontend
#[derive(Debug, Serialize)]
pub struct NodeResponse {
    pub id: String,
    pub name: String,
    pub exists: bool,
    pub tags: Vec<String>,
    pub link_count: usize,
    pub backlink_count: usize,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

/// Edge for frontend
#[derive(Debug, Serialize)]
pub struct EdgeResponse {
    pub source: String,
    pub target: String,
    pub link_type: String,
}

impl From<GraphData> for GraphDataResponse {
    fn from(data: GraphData) -> Self {
        Self {
            nodes: data
                .nodes
                .into_iter()
                .map(|n| NodeResponse {
                    id: n.id,
                    name: n.name,
                    exists: n.exists,
                    tags: n.tags,
                    link_count: n.link_count,
                    backlink_count: n.backlink_count,
                    x: n.x,
                    y: n.y,
                })
                .collect(),
            edges: data
                .edges
                .into_iter()
                .map(|e| EdgeResponse {
                    source: e.source,
                    target: e.target,
                    link_type: format!("{:?}", e.link_type).to_lowercase(),
                })
                .collect(),
        }
    }
}

/// Get full graph data
#[tauri::command]
pub async fn get_graph_data(
    state: State<'_, Arc<AppState>>,
) -> Result<GraphDataResponse, String> {
    debug!("Getting graph data");

    let graph = state.graph.read();
    let graph = graph
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    let mut data = graph.get_full_graph();
    data.calculate_sizes();
    data.apply_layout(50);

    Ok(GraphDataResponse::from(data))
}

/// Get local graph around a note
#[tauri::command]
pub async fn get_local_graph(
    path: String,
    depth: Option<usize>,
    state: State<'_, Arc<AppState>>,
) -> Result<GraphDataResponse, String> {
    debug!("Getting local graph for: {}", path);

    let graph = state.graph.read();
    let graph = graph
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    let depth = depth.unwrap_or(2);
    let mut data = graph.get_local_graph(&path, depth);
    data.calculate_sizes();
    data.apply_layout(30);

    Ok(GraphDataResponse::from(data))
}

/// Backlink information
#[derive(Debug, Serialize)]
pub struct BacklinkInfo {
    pub path: String,
    pub name: String,
    pub context: Option<String>,
}

/// Get backlinks for a note
#[tauri::command]
pub async fn get_backlinks(
    path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<BacklinkInfo>, String> {
    debug!("Getting backlinks for: {}", path);

    let graph = state.graph.read();
    let graph = graph
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    let backlinks = graph.get_backlinks(&path);

    Ok(backlinks
        .into_iter()
        .map(|bl| {
            let name = std::path::Path::new(&bl)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| bl.clone());

            BacklinkInfo {
                path: bl,
                name,
                context: None, // Would extract context in full implementation
            }
        })
        .collect())
}
