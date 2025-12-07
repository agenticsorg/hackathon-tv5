//! # Media Gateway MCP Server
//!
//! Model Context Protocol (MCP) server for AI-assisted content discovery.
//!
//! This server implements the MCP protocol to enable AI assistants to interact
//! with the Media Gateway platform for content discovery, recommendations,
//! and watchlist management.
//!
//! ## Features
//!
//! - JSON-RPC 2.0 protocol implementation
//! - Semantic search tools
//! - Personalized recommendations
//! - Content availability checking
//! - Resource access for content and user preferences
//! - Discovery prompts for AI assistants
//!
//! ## Protocol Support
//!
//! - Transport: HTTP/SSE (Server-Sent Events) and STDIO
//! - Protocol Version: MCP 1.0
//! - JSON-RPC Version: 2.0
//!
//! ## Transport Modes
//!
//! ### HTTP Transport (Default)
//! Standard HTTP server with Server-Sent Events for real-time updates.
//! Suitable for web-based integrations and testing.
//!
//! ### STDIO Transport
//! Line-delimited JSON-RPC over standard input/output.
//! Required for Claude Desktop integration and command-line MCP clients.

use sqlx::PgPool;

pub mod handlers;
pub mod protocol;
pub mod resources;
pub mod tools;
pub mod transport;

/// MCP Server state shared across handlers
pub struct McpServerState {
    /// Database connection pool
    pub db_pool: PgPool,
    /// Resource manager
    pub resource_manager: resources::ResourceManager,
}

impl McpServerState {
    /// Create new server state
    pub fn new(db_pool: PgPool) -> Self {
        let resource_manager = resources::ResourceManager::new(db_pool.clone());
        Self {
            db_pool,
            resource_manager,
        }
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Database URL
    pub database_url: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string()),
        }
    }
}

impl McpServerConfig {
    /// Load configuration from environment
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("MCP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("MCP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        }
    }

    /// Get server address
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = McpServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_address() {
        let config = McpServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_url: "postgresql://localhost/test".to_string(),
        };
        assert_eq!(config.address(), "127.0.0.1:8080");
    }
}
