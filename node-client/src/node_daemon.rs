use anyhow::{Result, Context};
use tokio::time::{interval, Duration};
use log::{info, warn, error, debug};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::NodeConfig;
use crate::near_client::NearClient;
use crate::task_processor::TaskProcessor;
use crate::heartbeat::HeartbeatManager;

pub struct NodeDaemon {
    config: NodeConfig,
    near_client: Arc<NearClient>,
    task_processor: Arc<Mutex<TaskProcessor>>,
    heartbeat_manager: Arc<HeartbeatManager>,
}

impl NodeDaemon {
    pub async fn new(config: NodeConfig) -> Result<Self> {
        info!("Initializing node daemon for account: {}", config.node.account_id);
        
        let near_client = Arc::new(
            NearClient::new(&config).await
                .context("Failed to initialize Near client")?
        );
        
        let task_processor = Arc::new(Mutex::new(
            TaskProcessor::new(&config).await
                .context("Failed to initialize task processor")?
        ));
        
        let heartbeat_manager = Arc::new(
            HeartbeatManager::new(near_client.clone())
        );
        
        Ok(Self {
            config,
            near_client,
            task_processor,
            heartbeat_manager,
        })
    }
    
    pub async fn register(&self) -> Result<()> {
        info!("Registering node with DeAI network...");
        
        // Check if already registered
        if let Some(node_info) = self.near_client.get_node_info().await? {
            warn!("Node already registered: {:?}", node_info);
            return Ok(());
        }
        
        // Parse stake amount
        let stake_amount = self.parse_stake_amount()?;
        
        // Check account balance
        let balance = self.near_client.get_account_balance().await?;
        info!("Account balance: {} yoctoNEAR", balance);
        
        if balance < stake_amount {
            anyhow::bail!(
                "Insufficient balance. Required: {} yoctoNEAR, Available: {} yoctoNEAR",
                stake_amount,
                balance
            );
        }
        
        // Construct API endpoint
        let api_endpoint = format!("http://{}:{}", self.config.node.public_ip, self.config.node.api_port);
        
        // Register with the contract
        let result = self.near_client.register_node(
            &self.config.node.public_ip,
            &self.config.hardware.gpu_specs,
            &self.config.hardware.cpu_specs,
            &api_endpoint,
            stake_amount,
        ).await?;
        
        info!("Node registered successfully! Transaction: {}", result.transaction.hash);
        
        // Verify registration
        match self.near_client.get_node_info().await? {
            Some(node_info) => {
                info!("Registration verified: {:?}", node_info);
            }
            None => {
                warn!("Registration may have failed - node info not found");
            }
        }
        
        Ok(())
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("Starting node daemon...");
        
        // Verify node is registered
        let node_info = self.near_client.get_node_info().await?
            .context("Node not registered. Please run 'register' command first.")?;
        
        info!("Node info: {:?}", node_info);
        
        if !node_info.is_active {
            warn!("Node is not active. You may need to re-register.");
        }
        
        // Start heartbeat manager
        let heartbeat_handle = {
            let heartbeat_manager = self.heartbeat_manager.clone();
            tokio::spawn(async move {
                heartbeat_manager.start().await;
            })
        };
        
        // Start task polling loop
        let task_handle = {
            let near_client = self.near_client.clone();
            let task_processor = self.task_processor.clone();
            tokio::spawn(async move {
                Self::task_polling_loop(near_client, task_processor).await;
            })
        };
        
        info!("Node daemon started. Press Ctrl+C to stop.");
        
        // Wait for interrupt signal
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received shutdown signal");
            }
            _ = heartbeat_handle => {
                error!("Heartbeat manager stopped unexpectedly");
            }
            _ = task_handle => {
                error!("Task polling stopped unexpectedly");
            }
        }
        
        info!("Shutting down node daemon...");
        Ok(())
    }
    
    pub async fn status(&self) -> Result<()> {
        info!("Checking node status...");
        
        match self.near_client.get_node_info().await? {
            Some(node_info) => {
                println!("Node Status:");
                println!("  Account ID: {}", node_info.account_id);
                println!("  Stake: {} yoctoNEAR", node_info.stake);
                println!("  Public IP: {}", node_info.public_ip);
                println!("  GPU Specs: {}", node_info.gpu_specs);
                println!("  CPU Specs: {}", node_info.cpu_specs);
                println!("  API Endpoint: {}", node_info.api_endpoint);
                println!("  Active: {}", node_info.is_active);
                println!("  Last Heartbeat: {}", node_info.last_heartbeat);
                println!("  Tasks Completed: {}", node_info.total_tasks_completed);
                println!("  Reputation Score: {}", node_info.reputation_score);
                
                // Check assigned tasks
                let tasks = self.near_client.get_assigned_tasks().await?;
                println!("  Assigned Tasks: {}", tasks.len());
                
                if !tasks.is_empty() {
                    println!("  Current Tasks:");
                    for task in tasks {
                        println!("    - Task {}: {} ({})", task.id, task.description, task.status);
                    }
                }
            }
            None => {
                println!("Node not registered.");
            }
        }
        
        Ok(())
    }
    
    pub async fn deactivate(&self) -> Result<()> {
        info!("Deactivating node...");
        
        let result = self.near_client.deactivate_node().await?;
        info!("Node deactivated successfully! Transaction: {}", result.transaction.hash);
        
        Ok(())
    }
    
    async fn task_polling_loop(
        near_client: Arc<NearClient>,
        task_processor: Arc<Mutex<TaskProcessor>>,
    ) {
        let mut interval = interval(Duration::from_secs(10)); // Poll every 10 seconds
        
        loop {
            interval.tick().await;
            
            match Self::process_pending_tasks(&near_client, &task_processor).await {
                Ok(processed_count) => {
                    if processed_count > 0 {
                        debug!("Processed {} tasks", processed_count);
                    }
                }
                Err(e) => {
                    error!("Error processing tasks: {}", e);
                }
            }
        }
    }
    
    async fn process_pending_tasks(
        near_client: &NearClient,
        task_processor: &Arc<Mutex<TaskProcessor>>,
    ) -> Result<usize> {
        let tasks = near_client.get_assigned_tasks().await?;
        
        if tasks.is_empty() {
            return Ok(0);
        }
        
        debug!("Found {} assigned tasks", tasks.len());
        let mut processed_count = 0;
        
        for task in tasks {
            if task.status == "Assigned" {
                info!("Processing task {}: {}", task.id, task.description);
                
                let processor = task_processor.lock().await;
                match processor.execute_task(&task).await {
                    Ok((proof_hash, output)) => {
                        drop(processor); // Release lock before network call
                        
                        match near_client.submit_result(task.id, &proof_hash, &output).await {
                            Ok(result) => {
                                info!("Task {} completed successfully! Transaction: {}", 
                                      task.id, result.transaction.hash);
                                processed_count += 1;
                            }
                            Err(e) => {
                                error!("Failed to submit result for task {}: {}", task.id, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to execute task {}: {}", task.id, e);
                    }
                }
            }
        }
        
        Ok(processed_count)
    }
    
    fn parse_stake_amount(&self) -> Result<u128> {
        let stake_near: f64 = self.config.node.stake_amount.parse()
            .context("Invalid stake amount format")?;
        
        let stake_yocto = (stake_near * 1e24) as u128;
        Ok(stake_yocto)
    }
}