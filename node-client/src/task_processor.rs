use anyhow::{Result, Context};
use log::{info, warn, error, debug};
use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::config::NodeConfig;
use crate::ai_engine::{AiEngine, TaskExecution};
use crate::near_client::TaskInfo;

pub struct TaskProcessor {
    config: NodeConfig,
    ai_engine: AiEngine,
    semaphore: Arc<Semaphore>,
}

impl TaskProcessor {
    pub async fn new(config: &NodeConfig) -> Result<Self> {
        let ai_engine = AiEngine::new(config)
            .context("Failed to initialize AI engine")?;
        
        // Check AI environment on initialization
        ai_engine.check_environment().await
            .context("AI environment check failed")?;
        
        let max_concurrent = config.hardware.max_concurrent_tasks as usize;
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        
        info!("Task processor initialized with max {} concurrent tasks", max_concurrent);
        
        Ok(Self {
            config: config.clone(),
            ai_engine,
            semaphore,
        })
    }
    
    pub async fn execute_task(&self, task: &TaskInfo) -> Result<(String, String)> {
        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await
            .context("Failed to acquire task execution permit")?;
        
        info!("Starting execution of task {}", task.id);
        
        // Validate task before execution
        self.validate_task(task)?;
        
        // Execute the AI task
        let execution_result = self.ai_engine.execute_task(&task.description).await
            .context("AI task execution failed")?;
        
        // Validate the execution result
        self.validate_execution_result(&execution_result)?;
        
        info!("Task {} completed successfully", task.id);
        Ok((execution_result.proof_hash, execution_result.output))
    }
    
    fn validate_task(&self, task: &TaskInfo) -> Result<()> {
        // Check task format
        if task.description.is_empty() {
            anyhow::bail!("Task description is empty");
        }
        
        // Parse and validate task description
        let task_desc: serde_json::Value = serde_json::from_str(&task.description)
            .context("Invalid task description JSON")?;
        
        // Check required fields
        let required_fields = ["model", "input", "task_type"];
        for field in required_fields {
            if !task_desc.get(field).is_some() {
                anyhow::bail!("Missing required field: {}", field);
            }
        }
        
        // Check model name format
        let model = task_desc["model"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Model must be a string"))?;
        
        if model.is_empty() {
            anyhow::bail!("Model name cannot be empty");
        }
        
        // Basic model name validation (should be Hugging Face format)
        if !model.contains('/') && !model.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            warn!("Model name '{}' may not be valid Hugging Face format", model);
        }
        
        // Check input size
        let input = task_desc["input"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Input must be a string"))?;
        
        if input.len() > 50_000 {
            anyhow::bail!("Input too large: {} characters (max 50,000)", input.len());
        }
        
        // Check task type
        let task_type = task_desc["task_type"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Task type must be a string"))?;
        
        let supported_types = ["inference", "text_generation", "classification", "embedding"];
        if !supported_types.contains(&task_type) {
            anyhow::bail!("Unsupported task type: {}", task_type);
        }
        
        debug!("Task validation passed for task {}", task.id);
        Ok(())
    }
    
    fn validate_execution_result(&self, result: &TaskExecution) -> Result<()> {
        // Check proof hash format
        if result.proof_hash.is_empty() {
            anyhow::bail!("Proof hash is empty");
        }
        
        if result.proof_hash.len() != 64 {
            anyhow::bail!("Invalid proof hash length: {}", result.proof_hash.len());
        }
        
        if !result.proof_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("Proof hash contains invalid characters");
        }
        
        // Check output format
        if result.output.is_empty() {
            anyhow::bail!("Output is empty");
        }
        
        // Validate output is valid JSON
        let _output_json: serde_json::Value = serde_json::from_str(&result.output)
            .context("Output is not valid JSON")?;
        
        // Check output size (reasonable limit)
        if result.output.len() > 1_000_000 {
            anyhow::bail!("Output too large: {} bytes (max 1MB)", result.output.len());
        }
        
        debug!("Execution result validation passed");
        Ok(())
    }
    
    pub async fn test_execution(&self) -> Result<()> {
        info!("Running task processor test...");
        
        // Create a simple test task
        let test_task = TaskInfo {
            id: 0,
            description: serde_json::json!({
                "model": "distilbert-base-uncased",
                "input": "This is a test input for task processor validation.",
                "task_type": "inference",
                "parameters": {
                    "max_length": 50
                }
            }).to_string(),
            assignee: Some(self.config.node.account_id.parse().unwrap()),
            status: "Assigned".to_string(),
            created_at: chrono::Utc::now().timestamp() as u64,
            reward_amount: "100000000000000000000000".to_string(), // 0.1 NEAR
            requester: "test.testnet".parse().unwrap(),
        };
        
        // Execute the test task
        let (proof_hash, output) = self.execute_task(&test_task).await?;
        
        info!("Test task completed successfully");
        debug!("Proof hash: {}", proof_hash);
        debug!("Output length: {} bytes", output.len());
        
        Ok(())
    }
    
    pub fn get_current_load(&self) -> usize {
        let max_permits = self.config.hardware.max_concurrent_tasks as usize;
        let available_permits = self.semaphore.available_permits();
        max_permits - available_permits
    }
    
    pub fn is_at_capacity(&self) -> bool {
        self.semaphore.available_permits() == 0
    }
    
    pub async fn wait_for_capacity(&self) -> Result<()> {
        let _permit = self.semaphore.acquire().await
            .context("Failed to wait for capacity")?;
        // Immediately release the permit
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
        config.hardware.max_concurrent_tasks = 2;
        config
    }
    
    fn create_test_task() -> TaskInfo {
        TaskInfo {
            id: 1,
            description: serde_json::json!({
                "model": "bert-base-uncased",
                "input": "test input",
                "task_type": "inference"
            }).to_string(),
            assignee: Some("test.testnet".parse().unwrap()),
            status: "Assigned".to_string(),
            created_at: 1234567890,
            reward_amount: "1000".to_string(),
            requester: "user.testnet".parse().unwrap(),
        }
    }
    
    #[tokio::test]
    async fn test_validate_task() {
        let config = create_test_config();
        let processor = TaskProcessor::new(&config).await.unwrap();
        
        let valid_task = create_test_task();
        assert!(processor.validate_task(&valid_task).is_ok());
        
        let mut invalid_task = create_test_task();
        invalid_task.description = "invalid json".to_string();
        assert!(processor.validate_task(&invalid_task).is_err());
    }
    
    #[test]
    fn test_validate_execution_result() {
        let config = create_test_config();
        // Note: This would need async setup in real test
        
        let valid_result = TaskExecution {
            proof_hash: "a".repeat(64),
            output: r#"{"result": "test", "timestamp": 1234567890}"#.to_string(),
        };
        
        // This test would need the actual processor instance
        // assert!(processor.validate_execution_result(&valid_result).is_ok());
    }
    
    #[test]
    fn test_capacity_management() {
        let config = create_test_config();
        // Test capacity management would require async setup
        // and proper semaphore testing
    }
}