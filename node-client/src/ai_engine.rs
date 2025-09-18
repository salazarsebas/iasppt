use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use log::{info, warn, error, debug};
use crate::config::NodeConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskExecution {
    pub proof_hash: String,
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDescription {
    pub model: String,
    pub input: String,
    pub task_type: String,
    pub parameters: Option<Value>,
}

pub struct AiEngine {
    config: NodeConfig,
    python_path: PathBuf,
    ai_worker_path: PathBuf,
}

impl AiEngine {
    pub fn new(config: &NodeConfig) -> Result<Self> {
        let python_path = PathBuf::from(&config.ai.python_path);
        let ai_worker_path = PathBuf::from("ai_engine/ai_worker.py");
        
        // Verify Python exists
        if !python_path.exists() {
            anyhow::bail!("Python path does not exist: {}", python_path.display());
        }
        
        // Verify AI worker script exists
        if !ai_worker_path.exists() {
            anyhow::bail!("AI worker script not found: {}", ai_worker_path.display());
        }
        
        Ok(Self {
            config: config.clone(),
            python_path,
            ai_worker_path,
        })
    }
    
    pub async fn execute_task(&self, task_description: &str) -> Result<TaskExecution> {
        info!("Executing AI task");
        
        // Parse task description
        let task_desc: TaskDescription = serde_json::from_str(task_description)
            .context("Failed to parse task description")?;
        
        debug!("Task: {} with model {}", task_desc.task_type, task_desc.model);
        
        // Validate task
        self.validate_task(&task_desc)?;
        
        // Prepare task data for Python worker
        let task_data = serde_json::json!({
            "description": task_description,
            "config": {
                "models_cache_dir": self.config.ai.models_cache_dir,
                "huggingface_token": self.config.ai.huggingface_token,
                "node_id": self.config.node.account_id
            }
        });
        
        // Execute Python AI worker
        let result = self.run_python_worker(&task_data).await?;
        
        info!("AI task completed successfully");
        Ok(result)
    }
    
    async fn run_python_worker(&self, task_data: &Value) -> Result<TaskExecution> {
        let task_json = serde_json::to_string(task_data)?;
        
        debug!("Running Python worker with task data");
        
        let mut cmd = Command::new(&self.python_path);
        cmd.arg(&self.ai_worker_path)
            .arg(&task_json)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        let output = cmd.output().await
            .context("Failed to execute Python AI worker")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Python worker failed: {}", stderr);
            anyhow::bail!("Python worker failed: {}", stderr);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("Python worker output: {}", stdout);
        
        let result: TaskExecution = serde_json::from_str(&stdout)
            .context("Failed to parse Python worker output")?;
        
        Ok(result)
    }
    
    fn validate_task(&self, task: &TaskDescription) -> Result<()> {
        // Check if model is reasonable size
        if task.model.is_empty() {
            anyhow::bail!("Model name cannot be empty");
        }
        
        // Check if task type is supported
        let supported_types = ["inference", "text_generation", "classification", "embedding"];
        if !supported_types.contains(&task.task_type.as_str()) {
            anyhow::bail!("Unsupported task type: {}", task.task_type);
        }
        
        // Check if framework is supported
        if !self.config.ai.supported_frameworks.contains(&"pytorch".to_string()) &&
           !self.config.ai.supported_frameworks.contains(&"transformers".to_string()) {
            anyhow::bail!("No supported AI frameworks configured");
        }
        
        // Check input size (basic validation)
        if task.input.len() > 10_000 {
            warn!("Large input detected: {} characters", task.input.len());
        }
        
        Ok(())
    }
    
    pub async fn check_environment(&self) -> Result<()> {
        info!("Checking AI environment...");
        
        // Check Python version
        let mut cmd = Command::new(&self.python_path);
        cmd.arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        let output = cmd.output().await
            .context("Failed to check Python version")?;
        
        if !output.status.success() {
            anyhow::bail!("Python check failed");
        }
        
        let version = String::from_utf8_lossy(&output.stdout);
        info!("Python version: {}", version.trim());
        
        // Check required packages
        self.check_python_packages().await?;
        
        info!("AI environment check completed successfully");
        Ok(())
    }
    
    async fn check_python_packages(&self) -> Result<()> {
        let required_packages = vec![
            "torch",
            "transformers",
            "huggingface_hub",
            "numpy",
        ];
        
        for package in required_packages {
            let mut cmd = Command::new(&self.python_path);
            cmd.arg("-c")
                .arg(format!("import {}", package))
                .stdout(Stdio::null())
                .stderr(Stdio::piped());
            
            let output = cmd.output().await
                .context(format!("Failed to check package: {}", package))?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Missing required package '{}': {}", package, stderr);
            }
        }
        
        info!("All required Python packages are available");
        Ok(())
    }
    
    pub async fn test_simple_task(&self) -> Result<()> {
        info!("Running simple AI task test...");
        
        let test_task = TaskDescription {
            model: "distilbert-base-uncased".to_string(),
            input: "Hello, this is a test input.".to_string(),
            task_type: "inference".to_string(),
            parameters: None,
        };
        
        let task_json = serde_json::to_string(&test_task)?;
        let result = self.execute_task(&task_json).await?;
        
        info!("Test task completed. Proof hash: {}", result.proof_hash);
        debug!("Test output: {}", result.output);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NodeConfig;
    
    fn create_test_config() -> NodeConfig {
        let mut config = NodeConfig::default();
        config.ai.python_path = "/usr/bin/python3".to_string();
        config.ai.models_cache_dir = "./test_models_cache".to_string();
        config.ai.supported_frameworks = vec!["pytorch".to_string(), "transformers".to_string()];
        config
    }
    
    #[test]
    fn test_validate_task() {
        let config = create_test_config();
        let engine = AiEngine::new(&config).unwrap();
        
        let valid_task = TaskDescription {
            model: "bert-base-uncased".to_string(),
            input: "test input".to_string(),
            task_type: "inference".to_string(),
            parameters: None,
        };
        
        assert!(engine.validate_task(&valid_task).is_ok());
        
        let invalid_task = TaskDescription {
            model: "".to_string(),
            input: "test input".to_string(),
            task_type: "invalid_type".to_string(),
            parameters: None,
        };
        
        assert!(engine.validate_task(&invalid_task).is_err());
    }
}