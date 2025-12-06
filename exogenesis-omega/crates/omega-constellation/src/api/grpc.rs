//! gRPC sync service for device synchronization

use crate::{PatternDelta, ShardManager};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info};

/// gRPC sync service
#[derive(Clone)]
pub struct SyncService {
    shard: Arc<ShardManager>,
}

impl SyncService {
    pub fn new(shard: Arc<ShardManager>) -> Self {
        info!("Initializing gRPC sync service");
        Self { shard }
    }

    /// Handle sync request from TV device
    pub async fn handle_push_patterns(
        &self,
        device_id: uuid::Uuid,
        delta: PatternDelta,
    ) -> Result<crate::GlobalPatterns, Status> {
        match self.shard.handle_sync(device_id, delta).await {
            Ok(global) => {
                info!("Sync successful for device {}", device_id);
                Ok(global)
            }
            Err(e) => {
                error!("Sync failed for device {}: {}", device_id, e);
                Err(Status::internal(format!("Sync failed: {}", e)))
            }
        }
    }
}

// TODO: Implement actual gRPC service with protobuf definitions
// This requires omega-protocol to define the proto messages
/*
use omega_protocol::proto::sync::{
    sync_service_server::{SyncService as ProtoSyncService, SyncServiceServer},
    PushRequest, PushResponse, PullRequest, PullResponse,
};

#[tonic::async_trait]
impl ProtoSyncService for SyncService {
    async fn push_patterns(
        &self,
        request: Request<PushRequest>,
    ) -> Result<Response<PushResponse>, Status> {
        let req = request.into_inner();
        let device_id = uuid::Uuid::parse_str(&req.device_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid device_id: {}", e)))?;

        // Decompress and deserialize delta
        let delta: PatternDelta = bincode::deserialize(&req.patterns_delta)
            .map_err(|e| Status::invalid_argument(format!("Invalid delta: {}", e)))?;

        let global = self.handle_push_patterns(device_id, delta).await?;

        Ok(Response::new(PushResponse {
            global_patterns: bincode::serialize(&global).unwrap(),
            global_version: global.global_version,
        }))
    }

    async fn pull_patterns(
        &self,
        request: Request<PullRequest>,
    ) -> Result<Response<PullResponse>, Status> {
        let req = request.into_inner();
        let device_id = uuid::Uuid::parse_str(&req.device_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid device_id: {}", e)))?;

        // Get global patterns for device
        let global = self.shard.get_global_patterns(&device_id).await
            .map_err(|e| Status::internal(format!("Failed to get patterns: {}", e)))?;

        Ok(Response::new(PullResponse {
            global_patterns: bincode::serialize(&global).unwrap(),
            global_version: global.global_version,
            trends: global.trending.into_iter().map(|t| /* convert to proto */).collect(),
        }))
    }
}

pub fn create_grpc_service(shard: Arc<ShardManager>) -> SyncServiceServer<SyncService> {
    SyncServiceServer::new(SyncService::new(shard))
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{storage::InMemoryStorage, ShardConfig};

    #[tokio::test]
    async fn test_sync_service_creation() {
        let storage = Arc::new(InMemoryStorage::new());
        let shard = Arc::new(
            ShardManager::new(ShardConfig::default(), storage)
                .await
                .unwrap(),
        );
        let _service = SyncService::new(shard);
    }
}
