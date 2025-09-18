use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use crate::errors::{ApiError, ApiResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_limit: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_limit: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTier {
    pub name: String,
    pub rate_limit: RateLimitConfig,
}

impl UserTier {
    pub fn free() -> Self {
        Self {
            name: "free".to_string(),
            rate_limit: RateLimitConfig {
                requests_per_minute: 30,
                requests_per_hour: 500,
                requests_per_day: 2000,
                burst_limit: 5,
            },
        }
    }
    
    pub fn pro() -> Self {
        Self {
            name: "pro".to_string(),
            rate_limit: RateLimitConfig {
                requests_per_minute: 120,
                requests_per_hour: 5000,
                requests_per_day: 50000,
                burst_limit: 20,
            },
        }
    }
    
    pub fn enterprise() -> Self {
        Self {
            name: "enterprise".to_string(),
            rate_limit: RateLimitConfig {
                requests_per_minute: 600,
                requests_per_hour: 20000,
                requests_per_day: 200000,
                burst_limit: 50,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_time: u64,
    pub retry_after: Option<u64>,
}

#[derive(Debug)]
pub struct RateLimiter {
    redis_client: redis::Client,
    default_config: RateLimitConfig,
    // Fallback in-memory rate limiter for when Redis is unavailable
    memory_store: Arc<Mutex<HashMap<String, Vec<u64>>>>,
}

impl RateLimiter {
    pub fn new(redis_client: redis::Client) -> Self {
        Self {
            redis_client,
            default_config: RateLimitConfig::default(),
            memory_store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn check_rate_limit(
        &self,
        identifier: &str,
        config: &RateLimitConfig,
    ) -> ApiResult<RateLimitInfo> {
        // Try Redis first, fallback to memory store
        match self.check_redis_rate_limit(identifier, config).await {
            Ok(info) => Ok(info),
            Err(_) => self.check_memory_rate_limit(identifier, config),
        }
    }
    
    async fn check_redis_rate_limit(
        &self,
        identifier: &str,
        config: &RateLimitConfig,
    ) -> ApiResult<RateLimitInfo> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await
            .map_err(|e| ApiError::Internal(format!("Redis connection failed: {}", e)))?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        // Check minute window
        let minute_key = format!("rate_limit:{}:minute:{}", identifier, now / 60);
        let minute_count: u32 = conn.incr(&minute_key, 1u32).await
            .map_err(|e| ApiError::Internal(format!("Redis incr failed: {}", e)))?;
        
        if minute_count == 1 {
            let _: () = conn.expire(&minute_key, 60).await
                .map_err(|e| ApiError::Internal(format!("Redis expire failed: {}", e)))?;
        }
        
        if minute_count > config.requests_per_minute {
            return Ok(RateLimitInfo {
                limit: config.requests_per_minute,
                remaining: 0,
                reset_time: (now / 60 + 1) * 60,
                retry_after: Some(60 - (now % 60)),
            });
        }
        
        // Check hour window
        let hour_key = format!("rate_limit:{}:hour:{}", identifier, now / 3600);
        let hour_count: u32 = conn.incr(&hour_key, 1u32).await
            .map_err(|e| ApiError::Internal(format!("Redis incr failed: {}", e)))?;
        
        if hour_count == 1 {
            let _: () = conn.expire(&hour_key, 3600).await
                .map_err(|e| ApiError::Internal(format!("Redis expire failed: {}", e)))?;
        }
        
        if hour_count > config.requests_per_hour {
            return Ok(RateLimitInfo {
                limit: config.requests_per_hour,
                remaining: 0,
                reset_time: (now / 3600 + 1) * 3600,
                retry_after: Some(3600 - (now % 3600)),
            });
        }
        
        // Check day window
        let day_key = format!("rate_limit:{}:day:{}", identifier, now / 86400);
        let day_count: u32 = conn.incr(&day_key, 1u32).await
            .map_err(|e| ApiError::Internal(format!("Redis incr failed: {}", e)))?;
        
        if day_count == 1 {
            let _: () = conn.expire(&day_key, 86400).await
                .map_err(|e| ApiError::Internal(format!("Redis expire failed: {}", e)))?;
        }
        
        if day_count > config.requests_per_day {
            return Ok(RateLimitInfo {
                limit: config.requests_per_day,
                remaining: 0,
                reset_time: (now / 86400 + 1) * 86400,
                retry_after: Some(86400 - (now % 86400)),
            });
        }
        
        // Check burst limit using sliding window
        let burst_key = format!("rate_limit:{}:burst", identifier);
        let burst_window = 60; // 1 minute window for burst
        
        // Add current timestamp to sorted set
        let _: () = conn.zadd(&burst_key, now, now).await
            .map_err(|e| ApiError::Internal(format!("Redis zadd failed: {}", e)))?;
        
        // Remove old entries (older than burst window)
        let _: () = conn.zremrangebyscore(&burst_key, 0, now - burst_window).await
            .map_err(|e| ApiError::Internal(format!("Redis zremrangebyscore failed: {}", e)))?;
        
        // Count entries in current window
        let burst_count: u32 = conn.zcard(&burst_key).await
            .map_err(|e| ApiError::Internal(format!("Redis zcard failed: {}", e)))?;
        
        // Set expiration for burst key
        let _: () = conn.expire(&burst_key, burst_window as i64).await
            .map_err(|e| ApiError::Internal(format!("Redis expire failed: {}", e)))?;
        
        if burst_count > config.burst_limit {
            return Ok(RateLimitInfo {
                limit: config.burst_limit,
                remaining: 0,
                reset_time: now + burst_window,
                retry_after: Some(burst_window),
            });
        }
        
        // Return success with remaining count
        let remaining = std::cmp::min(
            config.requests_per_minute - minute_count,
            std::cmp::min(
                config.requests_per_hour - hour_count,
                config.requests_per_day - day_count
            )
        );
        
        Ok(RateLimitInfo {
            limit: config.requests_per_minute,
            remaining,
            reset_time: (now / 60 + 1) * 60,
            retry_after: None,
        })
    }
    
    fn check_memory_rate_limit(
        &self,
        identifier: &str,
        config: &RateLimitConfig,
    ) -> ApiResult<RateLimitInfo> {
        let mut store = self.memory_store.lock().unwrap();
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let requests = store.entry(identifier.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests (older than 1 hour)
        requests.retain(|&timestamp| now - timestamp < 3600);
        
        // Add current request
        requests.push(now);
        
        // Check limits
        let minute_requests = requests.iter().filter(|&&t| now - t < 60).count() as u32;
        let hour_requests = requests.len() as u32;
        
        if minute_requests > config.requests_per_minute {
            return Ok(RateLimitInfo {
                limit: config.requests_per_minute,
                remaining: 0,
                reset_time: now + 60,
                retry_after: Some(60),
            });
        }
        
        if hour_requests > config.requests_per_hour {
            return Ok(RateLimitInfo {
                limit: config.requests_per_hour,
                remaining: 0,
                reset_time: now + 3600,
                retry_after: Some(3600),
            });
        }
        
        let remaining = std::cmp::min(
            config.requests_per_minute - minute_requests,
            config.requests_per_hour - hour_requests
        );
        
        Ok(RateLimitInfo {
            limit: config.requests_per_minute,
            remaining,
            reset_time: now + 60,
            retry_after: None,
        })
    }
    
    pub fn get_user_tier_config(&self, tier: &str) -> RateLimitConfig {
        match tier {
            "free" => UserTier::free().rate_limit,
            "pro" => UserTier::pro().rate_limit,
            "enterprise" => UserTier::enterprise().rate_limit,
            _ => self.default_config.clone(),
        }
    }
    
    pub fn get_ip_rate_limit_config(&self) -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 100,
            requests_per_hour: 2000,
            requests_per_day: 20000,
            burst_limit: 20,
        }
    }
}