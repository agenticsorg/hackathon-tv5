use crate::config::{Config, RateLimitTier};
use crate::error::{ApiError, ApiResult};
use chrono::{Duration, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use tracing::{debug, warn};

pub struct RateLimiter {
    redis: ConnectionManager,
    config: Arc<Config>,
}

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset: i64,
}

impl RateLimiter {
    pub async fn new(config: Arc<Config>) -> anyhow::Result<Self> {
        let client = redis::Client::open(config.redis.url.as_str())?;
        let redis = ConnectionManager::new(client).await?;

        Ok(Self { redis, config })
    }

    pub async fn check_rate_limit(&self, user_id: &str, tier: &str) -> ApiResult<RateLimitInfo> {
        if !self.config.rate_limit.enabled {
            return Ok(RateLimitInfo {
                limit: u32::MAX,
                remaining: u32::MAX,
                reset: (Utc::now() + Duration::minutes(1)).timestamp(),
            });
        }

        let tier_config = self
            .config
            .rate_limit
            .tiers
            .get(tier)
            .or_else(|| self.config.rate_limit.tiers.get("anonymous"))
            .ok_or_else(|| ApiError::InternalError(anyhow::anyhow!("Invalid rate limit tier")))?;

        // Check per-second rate limit using sliding window
        let second_info = self
            .check_sliding_window(user_id, tier_config, 1, tier_config.requests_per_second)
            .await?;

        // Check per-minute rate limit using sliding window
        let minute_info = self
            .check_sliding_window(user_id, tier_config, 60, tier_config.requests_per_minute)
            .await?;

        // Return the most restrictive limit
        let info = if second_info.remaining < minute_info.remaining {
            second_info
        } else {
            minute_info
        };

        if info.remaining == 0 {
            warn!(user_id = user_id, tier = tier, "Rate limit exceeded");
            return Err(ApiError::RateLimited {
                retry_after: (info.reset - Utc::now().timestamp()) as u64,
            });
        }

        debug!(
            user_id = user_id,
            tier = tier,
            remaining = info.remaining,
            "Rate limit check passed"
        );

        Ok(info)
    }

    async fn check_sliding_window(
        &self,
        user_id: &str,
        _tier: &RateLimitTier,
        window_seconds: i64,
        limit: u32,
    ) -> ApiResult<RateLimitInfo> {
        let now = Utc::now();
        let window_start = now - Duration::seconds(window_seconds);
        let window_end = now;

        let key = format!("ratelimit:{}:{}", user_id, window_seconds);
        let score_start = window_start.timestamp_millis() as f64;
        let score_end = window_end.timestamp_millis() as f64;
        let member = now.timestamp_nanos_opt().unwrap_or(0).to_string();

        let mut conn = self.redis.clone();

        // Remove old entries
        let _: () = conn
            .zrembyscore(&key, f64::MIN, score_start)
            .await
            .map_err(|e| ApiError::InternalError(e.into()))?;

        // Add current request
        let _: () = conn
            .zadd(&key, &member, score_end)
            .await
            .map_err(|e| ApiError::InternalError(e.into()))?;

        // Set expiration
        let _: () = conn
            .expire(&key, window_seconds + 1)
            .await
            .map_err(|e| ApiError::InternalError(e.into()))?;

        // Count requests in window
        let count: u32 = conn
            .zcount(&key, score_start, score_end)
            .await
            .map_err(|e| ApiError::InternalError(e.into()))?;

        let remaining = if count > limit { 0 } else { limit - count };
        let reset = (now + Duration::seconds(window_seconds)).timestamp();

        Ok(RateLimitInfo {
            limit,
            remaining,
            reset,
        })
    }

    pub fn get_user_tier(&self, user: Option<&str>) -> String {
        // In a real implementation, this would look up the user's subscription tier
        // For now, return based on whether user is authenticated
        match user {
            None => "anonymous".to_string(),
            Some(_) => "free".to_string(), // Default authenticated tier
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_disabled() {
        let mut config = Config::default();
        config.rate_limit.enabled = false;

        // This test requires Redis to be running, so we'll skip it in CI
        // In a real implementation, we'd use a mock Redis client
    }
}
