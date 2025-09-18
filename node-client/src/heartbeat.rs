use anyhow::{Result, Context};
use tokio::time::{interval, Duration, Instant};
use log::{info, warn, error, debug};
use std::sync::Arc;
use crate::near_client::NearClient;

pub struct HeartbeatManager {
    near_client: Arc<NearClient>,
    interval_seconds: u64,
    max_retries: u32,
}

impl HeartbeatManager {
    pub fn new(near_client: Arc<NearClient>) -> Self {
        Self {
            near_client,
            interval_seconds: 60, // 1 minute intervals
            max_retries: 3,
        }
    }
    
    pub fn with_interval(mut self, seconds: u64) -> Self {
        self.interval_seconds = seconds;
        self
    }
    
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
    
    pub async fn start(&self) {
        info!("Starting heartbeat manager with {} second intervals", self.interval_seconds);
        
        let mut interval = interval(Duration::from_secs(self.interval_seconds));
        let mut consecutive_failures = 0u32;
        let mut last_success = Instant::now();
        
        loop {
            interval.tick().await;
            
            match self.send_heartbeat().await {
                Ok(_) => {
                    if consecutive_failures > 0 {
                        info!("Heartbeat recovered after {} failures", consecutive_failures);
                        consecutive_failures = 0;
                    } else {
                        debug!("Heartbeat sent successfully");
                    }
                    last_success = Instant::now();
                }
                Err(e) => {
                    consecutive_failures += 1;
                    error!("Heartbeat failed (attempt {}): {}", consecutive_failures, e);
                    
                    if consecutive_failures >= self.max_retries {
                        error!("Max heartbeat failures reached. Node may be marked inactive.");
                        
                        // Wait longer before retrying after max failures
                        tokio::time::sleep(Duration::from_secs(self.interval_seconds * 2)).await;
                        consecutive_failures = 0; // Reset to keep trying
                    }
                }
            }
            
            // Check if we've been down for too long
            let time_since_success = last_success.elapsed();
            if time_since_success > Duration::from_secs(self.interval_seconds * 5) {
                warn!("No successful heartbeat for {} seconds", time_since_success.as_secs());
            }
        }
    }
    
    async fn send_heartbeat(&self) -> Result<()> {
        debug!("Sending heartbeat to DeAI network");
        
        let start_time = Instant::now();
        
        let result = self.near_client.heartbeat().await
            .context("Failed to send heartbeat transaction")?;
        
        let duration = start_time.elapsed();
        
        debug!("Heartbeat transaction completed in {:?}: {}", 
               duration, result.transaction.hash);
        
        // Verify transaction success
        if let Some(failure) = result.status.as_failure() {
            anyhow::bail!("Heartbeat transaction failed: {:?}", failure);
        }
        
        Ok(())
    }
    
    pub async fn send_immediate_heartbeat(&self) -> Result<()> {
        info!("Sending immediate heartbeat");
        self.send_heartbeat().await
    }
    
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let start_time = Instant::now();
        
        // Check if we can connect to Near network
        let node_info = self.near_client.get_node_info().await
            .context("Failed to fetch node info for health check")?;
        
        let network_latency = start_time.elapsed();
        
        let status = if let Some(info) = node_info {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
            
            let time_since_heartbeat = current_time.saturating_sub(info.last_heartbeat);
            let heartbeat_age_seconds = time_since_heartbeat / 1_000_000_000;
            
            HealthStatus {
                is_registered: true,
                is_active: info.is_active,
                network_latency,
                last_heartbeat_age_seconds: heartbeat_age_seconds,
                reputation_score: info.reputation_score,
                total_tasks_completed: info.total_tasks_completed,
                current_stake: info.stake,
            }
        } else {
            HealthStatus {
                is_registered: false,
                is_active: false,
                network_latency,
                last_heartbeat_age_seconds: u64::MAX,
                reputation_score: 0,
                total_tasks_completed: 0,
                current_stake: "0".to_string(),
            }
        };
        
        Ok(status)
    }
}

#[derive(Debug)]
pub struct HealthStatus {
    pub is_registered: bool,
    pub is_active: bool,
    pub network_latency: Duration,
    pub last_heartbeat_age_seconds: u64,
    pub reputation_score: u32,
    pub total_tasks_completed: u64,
    pub current_stake: String,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.is_registered && 
        self.is_active && 
        self.last_heartbeat_age_seconds < 300 && // Less than 5 minutes old
        self.network_latency < Duration::from_secs(10) // Less than 10 second latency
    }
    
    pub fn get_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        if !self.is_registered {
            issues.push("Node not registered".to_string());
        }
        
        if !self.is_active {
            issues.push("Node marked as inactive".to_string());
        }
        
        if self.last_heartbeat_age_seconds > 300 {
            issues.push(format!("Last heartbeat too old: {} seconds", self.last_heartbeat_age_seconds));
        }
        
        if self.network_latency > Duration::from_secs(10) {
            issues.push(format!("High network latency: {:?}", self.network_latency));
        }
        
        if self.reputation_score < 50 {
            issues.push(format!("Low reputation score: {}", self.reputation_score));
        }
        
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_health_status_healthy() {
        let healthy_status = HealthStatus {
            is_registered: true,
            is_active: true,
            network_latency: Duration::from_millis(100),
            last_heartbeat_age_seconds: 60,
            reputation_score: 100,
            total_tasks_completed: 10,
            current_stake: "1000000000000000000000000".to_string(),
        };
        
        assert!(healthy_status.is_healthy());
        assert!(healthy_status.get_issues().is_empty());
    }
    
    #[test]
    fn test_health_status_unhealthy() {
        let unhealthy_status = HealthStatus {
            is_registered: false,
            is_active: false,
            network_latency: Duration::from_secs(15),
            last_heartbeat_age_seconds: 600,
            reputation_score: 30,
            total_tasks_completed: 0,
            current_stake: "0".to_string(),
        };
        
        assert!(!unhealthy_status.is_healthy());
        let issues = unhealthy_status.get_issues();
        assert!(issues.len() > 0);
        assert!(issues.iter().any(|i| i.contains("not registered")));
        assert!(issues.iter().any(|i| i.contains("inactive")));
        assert!(issues.iter().any(|i| i.contains("latency")));
        assert!(issues.iter().any(|i| i.contains("heartbeat too old")));
        assert!(issues.iter().any(|i| i.contains("reputation")));
    }
    
    #[test]
    fn test_heartbeat_manager_creation() {
        // This would require a mock NearClient for proper testing
        // let mock_client = Arc::new(MockNearClient::new());
        // let manager = HeartbeatManager::new(mock_client);
        // assert_eq!(manager.interval_seconds, 60);
        // assert_eq!(manager.max_retries, 3);
    }
}