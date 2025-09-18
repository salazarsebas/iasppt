use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node: NodeSettings,
    pub near: NearConfig,
    pub ai: AiConfig,
    pub hardware: HardwareConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSettings {
    pub account_id: String,
    pub private_key: String,
    pub public_ip: String,
    pub api_port: u16,
    pub stake_amount: String, // In NEAR tokens
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
pub struct AiConfig {
    pub python_path: String,
    pub models_cache_dir: String,
    pub max_model_size_gb: u64,
    pub huggingface_token: Option<String>,
    pub supported_frameworks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub gpu_specs: String,
    pub cpu_specs: String,
    pub memory_gb: u64,
    pub storage_gb: u64,
    pub max_concurrent_tasks: u32,
}

impl NodeConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        
        let config: NodeConfig = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;
        
        Ok(())
    }
    
    pub fn create_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = Self::default();
        config.save(&path)?;
        Ok(config)
    }
    
    fn validate(&self) -> Result<()> {
        if self.node.account_id.is_empty() {
            anyhow::bail!("Node account_id cannot be empty");
        }
        
        if self.node.private_key.is_empty() {
            anyhow::bail!("Node private_key cannot be empty");
        }
        
        if self.node.public_ip.is_empty() {
            anyhow::bail!("Node public_ip cannot be empty");
        }
        
        if self.near.contract_account_id.is_empty() {
            anyhow::bail!("Contract account_id cannot be empty");
        }
        
        if self.ai.python_path.is_empty() {
            anyhow::bail!("Python path cannot be empty");
        }
        
        Ok(())
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node: NodeSettings {
                account_id: "your-node.testnet".to_string(),
                private_key: "YOUR_PRIVATE_KEY_HERE".to_string(),
                public_ip: "127.0.0.1".to_string(),
                api_port: 8080,
                stake_amount: "1.0".to_string(),
            },
            near: NearConfig {
                network_id: "testnet".to_string(),
                contract_account_id: "deai-compute.testnet".to_string(),
                rpc_url: "https://rpc.testnet.near.org".to_string(),
                wallet_url: "https://testnet.mynearwallet.com".to_string(),
                explorer_url: "https://testnet.nearblocks.io".to_string(),
            },
            ai: AiConfig {
                python_path: "/usr/bin/python3".to_string(),
                models_cache_dir: "./models_cache".to_string(),
                max_model_size_gb: 10,
                huggingface_token: None,
                supported_frameworks: vec![
                    "pytorch".to_string(),
                    "tensorflow".to_string(),
                    "transformers".to_string(),
                ],
            },
            hardware: HardwareConfig {
                gpu_specs: "NVIDIA RTX 4090".to_string(),
                cpu_specs: "Intel i9-13900K".to_string(),
                memory_gb: 32,
                storage_gb: 1000,
                max_concurrent_tasks: 2,
            },
        }
    }
}