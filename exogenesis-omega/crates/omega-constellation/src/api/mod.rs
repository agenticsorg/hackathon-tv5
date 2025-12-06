//! API module - gRPC and REST endpoints

pub mod grpc;
pub mod rest;

pub use grpc::SyncService;
pub use rest::{create_rest_router, RestState};
