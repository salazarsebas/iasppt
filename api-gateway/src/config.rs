use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub near: NearConfig,
    pub rate_limits: RateLimitConfig,
    pub admin: AdminConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearConfig {
    pub network_id: String,
    pub contract_account_id: String,
    pub rpc_url: String,
    pub wallet_url: String,
    pub explorer_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub max_concurrent_tasks: u32,
    pub task_submission_cooldown_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfig {
    pub admin_accounts: Vec<String>,
    pub metrics_retention_days: u32,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        let config = Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:deai_gateway.db".to_string()),
            
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string()),
            
            near: NearConfig {
                network_id: env::var("NEAR_NETWORK_ID")
                    .unwrap_or_else(|_| "testnet".to_string()),
                contract_account_id: env::var("NEAR_CONTRACT_ACCOUNT_ID")
                    .unwrap_or_else(|_| "deai-compute.testnet".to_string()),
                rpc_url: env::var("NEAR_RPC_URL")
                    .unwrap_or_else(|_| "https://rpc.testnet.near.org".to_string()),
                wallet_url: env::var("NEAR_WALLET_URL")
                    .unwrap_or_else(|_| "https://testnet.mynearwallet.com".to_string()),
                explorer_url: env::var("NEAR_EXPLORER_URL")
                    .unwrap_or_else(|_| "https://testnet.nearblocks.io".to_string()),
            },
            
            rate_limits: RateLimitConfig {
                requests_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
                requests_per_hour: env::var("RATE_LIMIT_PER_HOUR")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
                max_concurrent_tasks: env::var("MAX_CONCURRENT_TASKS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                task_submission_cooldown_seconds: env::var("TASK_COOLDOWN_SECONDS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
            },
            
            admin: AdminConfig {
                admin_accounts: env::var("ADMIN_ACCOUNTS")
                    .unwrap_or_else(|_| "admin.testnet".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                metrics_retention_days: env::var("METRICS_RETENTION_DAYS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
            },
        };
        
        config.validate()?;
        Ok(config)
    }
    
    fn validate(&self) -> Result<()> {
        if self.jwt_secret.len() < 32 {
            anyhow::bail!("JWT secret must be at least 32 characters long");
        }
        
        if self.near.contract_account_id.is_empty() {
            anyhow::bail!("Near contract account ID cannot be empty");
        }
        
        if self.rate_limits.requests_per_minute == 0 {
            anyhow::bail!("Rate limit per minute must be greater than 0");
        }
        
        Ok(())
    }
    
    pub fn is_admin_account(&self, account_id: &str) -> bool {
        self.admin.admin_accounts.contains(&account_id.to_string())
    }
}