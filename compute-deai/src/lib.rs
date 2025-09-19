use near_sdk::borsh::{BorshDeserialize, BorshSerialize, BorshSchema};
use near_sdk::collections::{UnorderedMap, Vector, LookupMap};
use near_sdk::{near, AccountId, env, Promise, json_types::U128, PanicOnDefault, NearToken, log, require, Gas};
use schemars::JsonSchema;
use near_contract_standards::fungible_token::{FungibleToken, FungibleTokenCore, Balance};
use serde::{Deserialize, Serialize};


pub const MIN_STAKE_YOCTO: u128 = 1_000_000_000_000_000_000_000_000; // 1 NEAR
pub const STORAGE_COST: Balance = 1_000_000_000_000_000_000_000; // 0.001 NEAR
pub const ONE_YOCTO: u128 = 1;
pub const HEARTBEAT_TIMEOUT: u64 = 300_000_000_000; // 5 minutes in nanoseconds
pub const MAX_REPUTATION: u32 = 1000;
pub const REPUTATION_GAIN: u32 = 10;
pub const REPUTATION_LOSS: u32 = 50;
pub const CALLBACK_GAS: Gas = Gas::from_tgas(5); // 5 TGas for callbacks
pub const MAX_TASK_TIMEOUT: u64 = 3600_000_000_000; // 1 hour in nanoseconds

#[derive(BorshDeserialize, BorshSerialize, BorshSchema, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct NodeInfo {
    pub account_id: String,
    pub stake: u128,
    pub public_ip: String,
    pub gpu_specs: String,
    pub cpu_specs: String,
    pub api_endpoint: String,
    pub is_active: bool,
    pub last_heartbeat: u64,
    pub total_tasks_completed: u64,
    pub reputation_score: u32,
    pub slashed_amount: u128,
    pub registration_time: u64,
}

#[derive(BorshDeserialize, BorshSerialize, BorshSchema, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Task {
    pub id: u64,
    pub description: String,  // JSON: {model: "url", input: "data", task_type: "inference"}
    pub assignee: Option<String>,
    pub status: TaskStatus,
    pub output: Option<String>,
    pub proof_hash: Option<String>,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub assigned_at: Option<u64>,
    pub timeout_at: Option<u64>,
    pub reward_amount: Balance,
    pub requester: String,
    pub priority: TaskPriority,
}

#[derive(BorshDeserialize, BorshSerialize, BorshSchema, Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
    TimedOut,
    Disputed,
}

#[derive(BorshDeserialize, BorshSerialize, BorshSchema, Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct DeAICompute {
    pub nodes: UnorderedMap<AccountId, NodeInfo>,
    pub active_tasks: UnorderedMap<u64, Task>,
    pub completed_tasks: LookupMap<u64, Task>,
    pub pending_tasks: Vector<u64>,
    pub task_counter: u64,
    pub token: FungibleToken,
    pub min_stake: u128,
    pub total_rewards_distributed: Balance,
    pub owner_id: AccountId,
    pub paused: bool,
    pub max_tasks_per_node: u32,
    pub task_timeout_duration: u64,
}

#[near]
impl DeAICompute {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        require!(!env::state_exists(), "Contract already initialized");
        
        let mut token = FungibleToken::new(b"t".to_vec());
        token.internal_register_account(&owner_id);
        
        Self {
            nodes: UnorderedMap::new(b"n".to_vec()),
            active_tasks: UnorderedMap::new(b"at".to_vec()),
            completed_tasks: LookupMap::new(b"ct".to_vec()),
            pending_tasks: Vector::new(b"pt".to_vec()),
            task_counter: 0,
            token,
            min_stake: MIN_STAKE_YOCTO,
            total_rewards_distributed: 0,
            owner_id,
            paused: false,
            max_tasks_per_node: 5,
            task_timeout_duration: MAX_TASK_TIMEOUT,
        }
    }

    // Security modifiers
    fn assert_not_paused(&self) {
        require!(!self.paused, "Contract is paused");
    }
    
    fn assert_owner(&self) {
        require!(env::predecessor_account_id() == self.owner_id, "Only owner can call this method");
    }
    
    fn assert_one_yocto(&self) {
        require!(env::attached_deposit().as_yoctonear() == ONE_YOCTO, "Exactly 1 yoctoNEAR required for security");
    }

    // Node Registry Functions
    #[payable]
    pub fn register_node(
        &mut self,
        public_ip: String,
        gpu_specs: String,
        cpu_specs: String,
        api_endpoint: String,
    ) {
        self.assert_not_paused();
        let account_id = env::predecessor_account_id();
        let stake = env::attached_deposit();
        
        require!(stake.as_yoctonear() >= self.min_stake, "Insufficient stake. Minimum: {} yoctoNEAR");
        require!(self.nodes.get(&account_id).is_none(), "Node already registered");
        require!(!public_ip.is_empty(), "Public IP cannot be empty");
        require!(!api_endpoint.is_empty(), "API endpoint cannot be empty");
        require!(gpu_specs.len() <= 500, "GPU specs too long");
        require!(cpu_specs.len() <= 500, "CPU specs too long");
        require!(api_endpoint.len() <= 200, "API endpoint too long");

        // Validate IP is unique - use iterator for efficiency
        for node in self.nodes.values() {
            require!(node.public_ip != public_ip, "IP address already registered");
        }

        let node_info = NodeInfo {
            account_id: account_id.to_string(),
            stake: stake.as_yoctonear(),
            public_ip,
            gpu_specs,
            cpu_specs,
            api_endpoint,
            is_active: true,
            last_heartbeat: env::block_timestamp(),
            total_tasks_completed: 0,
            reputation_score: 100, // Start with base reputation
            slashed_amount: 0,
            registration_time: env::block_timestamp(),
        };

        self.nodes.insert(&account_id, &node_info);
        
        // Register account for token rewards
        if !self.token.accounts.contains_key(&account_id) {
            self.token.internal_register_account(&account_id);
        }
        
        log!("Node registered: {}", account_id);
    }

    pub fn heartbeat(&mut self) {
        self.assert_not_paused();
        let account_id = env::predecessor_account_id();
        let mut node = self.nodes.get(&account_id).expect("Node not registered").clone();
        
        node.last_heartbeat = env::block_timestamp();
        node.is_active = true;
        
        self.nodes.insert(&account_id, &node);
        log!("Heartbeat from node: {}", account_id);
    }

    #[payable]
    pub fn deactivate_node(&mut self) {
        self.assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let mut node = self.nodes.get(&account_id).expect("Node not registered").clone();
        
        require!(node.is_active, "Node already inactive");
        
        // Check if node has pending tasks
        let has_active_tasks = self.node_has_active_task(&account_id);
        require!(!has_active_tasks, "Cannot deactivate node with active tasks");
        
        // Calculate amount to return (stake minus any slashing)
        let return_amount = node.stake.saturating_sub(node.slashed_amount);
        
        if return_amount > 0 {
            Promise::new(account_id.clone()).transfer(NearToken::from_yoctonear(return_amount));
        }
        
        node.is_active = false;
        self.nodes.insert(&account_id, &node);
        
        log!("Node deactivated: {}, returned: {} yoctoNEAR", account_id, return_amount);
    }

    // Task Management Functions
    #[payable]
    pub fn submit_task(&mut self, description: String, estimated_compute_cost: U128, priority: Option<TaskPriority>) {
        self.assert_not_paused();
        let requester = env::predecessor_account_id();
        let fee = env::attached_deposit();
        let compute_cost: Balance = estimated_compute_cost.into();
        
        require!(fee.as_yoctonear() >= compute_cost + STORAGE_COST, "Insufficient payment for compute cost and storage");
        require!(!description.is_empty(), "Task description cannot be empty");
        require!(description.len() <= 1000, "Task description too long");
        require!(compute_cost > 0, "Compute cost must be positive");

        // Register requester for token operations if needed
        if !self.token.accounts.contains_key(&requester) {
            self.token.internal_register_account(&requester);
        }

        let task = Task {
            id: self.task_counter,
            description,
            assignee: None,
            status: TaskStatus::Pending,
            output: None,
            proof_hash: None,
            created_at: env::block_timestamp(),
            completed_at: None,
            assigned_at: None,
            timeout_at: None,
            reward_amount: compute_cost,
            requester: requester.to_string(),
            priority: priority.unwrap_or(TaskPriority::Normal),
        };

        self.active_tasks.insert(&self.task_counter, &task);
        self.pending_tasks.push(&self.task_counter);
        self.task_counter += 1;
        
        log!("Task submitted: {}, requester: {}, amount: {}", self.task_counter - 1, requester, compute_cost);
        
        // Try to assign to available node
        self.try_assign_next_task();
    }

    #[payable]
    pub fn submit_result(&mut self, task_id: u64, proof_hash: String, output: String) {
        self.assert_one_yocto();
        let account_id = env::predecessor_account_id();
        
        // Get task from active tasks
        let mut task = self.active_tasks.get(&task_id).expect("Task not found").clone();
        
        require!(task.assignee.as_ref() == Some(&account_id.to_string()), "Not assigned to this node");
        require!(matches!(task.status, TaskStatus::Assigned | TaskStatus::InProgress), "Task not in assignable state");
        require!(!proof_hash.is_empty(), "Proof hash cannot be empty");
        require!(!output.is_empty(), "Output cannot be empty");
        require!(proof_hash.len() <= 64, "Proof hash too long");
        require!(output.len() <= 10000, "Output too long");

        // Check if task has timed out
        if let Some(timeout) = task.timeout_at {
            require!(env::block_timestamp() <= timeout, "Task has timed out");
        }

        // Update task
        task.status = TaskStatus::Completed;
        task.output = Some(output);
        task.proof_hash = Some(proof_hash);
        task.completed_at = Some(env::block_timestamp());

        // Update node stats
        let mut node = self.nodes.get(&account_id).unwrap().clone();
        node.total_tasks_completed += 1;
        node.reputation_score = std::cmp::min(MAX_REPUTATION, node.reputation_score + REPUTATION_GAIN);
        self.nodes.insert(&account_id, &node);

        // Mint reward tokens
        self.token.internal_deposit(&account_id, task.reward_amount);
        self.total_rewards_distributed += task.reward_amount;

        // Move task to completed
        self.active_tasks.remove(&task_id);
        self.completed_tasks.insert(&task_id, &task);
        
        log!("Task completed: {}, node: {}, reward: {}", task_id, account_id, task.reward_amount);

        // Try to assign next task
        self.try_assign_next_task();
    }

    fn try_assign_next_task(&mut self) {
        if let Some(available_node) = self.get_available_node() {
            // Find highest priority pending task
            let mut best_task_index = None;
            let mut best_priority = TaskPriority::Low;
            
            for i in 0..self.pending_tasks.len() {
                let task_id = self.pending_tasks.get(i).unwrap();
                if let Some(task) = self.active_tasks.get(&task_id) {
                    if task.status == TaskStatus::Pending {
                        if best_task_index.is_none() || self.priority_value(&task.priority) > self.priority_value(&best_priority) {
                            best_task_index = Some(i);
                            best_priority = task.priority.clone();
                        }
                    }
                }
            }
            
            if let Some(index) = best_task_index {
                let task_id = self.pending_tasks.get(index).unwrap();
                self.pending_tasks.swap_remove(index);
                if let Some(task) = self.active_tasks.get(&task_id) {
                    let mut updated_task = task.clone();
                    updated_task.assignee = Some(available_node.to_string());
                    updated_task.status = TaskStatus::Assigned;
                    updated_task.assigned_at = Some(env::block_timestamp());
                    updated_task.timeout_at = Some(env::block_timestamp() + self.task_timeout_duration);
                    
                    self.active_tasks.insert(&task_id, &updated_task);
                    log!("Task assigned: {} to node: {}", task_id, available_node);
                }
            }
        }
    }
    
    fn priority_value(&self, priority: &TaskPriority) -> u8 {
        match priority {
            TaskPriority::Low => 1,
            TaskPriority::Normal => 2,
            TaskPriority::High => 3,
            TaskPriority::Urgent => 4,
        }
    }

    fn get_available_node(&self) -> Option<AccountId> {
        let current_time = env::block_timestamp();
        
        // Find node with highest reputation that's available
        let mut best_node = None;
        let mut best_reputation = 0;
        
        for (account_id, node) in self.nodes.iter() {
            if node.is_active 
                && current_time - node.last_heartbeat < HEARTBEAT_TIMEOUT
                && node.reputation_score > best_reputation
                && self.get_node_active_task_count(&account_id) < self.max_tasks_per_node {
                best_node = Some(account_id.clone());
                best_reputation = node.reputation_score;
            }
        }
        best_node
    }

    fn node_has_active_task(&self, node_id: &AccountId) -> bool {
        self.get_node_active_task_count(node_id) > 0
    }
    
    fn get_node_active_task_count(&self, node_id: &AccountId) -> u32 {
        let mut count = 0;
        for task in self.active_tasks.values() {
            if task.assignee.as_ref() == Some(&node_id.to_string()) && 
               matches!(task.status, TaskStatus::Assigned | TaskStatus::InProgress) {
                count += 1;
            }
        }
        count
    }

    // Timeout and slashing functions
    #[payable] 
    pub fn timeout_task(&mut self, task_id: u64) {
        self.assert_one_yocto();
        let mut task = self.active_tasks.get(&task_id).expect("Task not found").clone();
        
        require!(matches!(task.status, TaskStatus::Assigned | TaskStatus::InProgress), "Task not active");
        
        if let Some(timeout) = task.timeout_at {
            require!(env::block_timestamp() > timeout, "Task has not timed out yet");
        }
        
        // Slash node reputation and stake
        if let Some(assignee_str) = &task.assignee {
            if let Ok(assignee_id) = assignee_str.parse::<AccountId>() {
                if let Some(mut node) = self.nodes.get(&assignee_id) {
                    let mut updated_node = node.clone();
                    updated_node.reputation_score = updated_node.reputation_score.saturating_sub(REPUTATION_LOSS);
                    
                    // Slash 10% of stake
                    let slash_amount = updated_node.stake / 10;
                    updated_node.slashed_amount += slash_amount;
                    
                    self.nodes.insert(&assignee_id, &updated_node);
                    log!("Node slashed for timeout: {}, amount: {}", assignee_id, slash_amount);
                }
            }
        }
        
        task.status = TaskStatus::TimedOut;
        task.completed_at = Some(env::block_timestamp());
        
        // Return funds to requester
        if let Ok(requester_id) = task.requester.parse::<AccountId>() {
            Promise::new(requester_id).transfer(NearToken::from_yoctonear(task.reward_amount));
        }
        
        self.active_tasks.remove(&task_id);
        self.completed_tasks.insert(&task_id, &task);
        
        log!("Task timed out: {}", task_id);
    }

    // View Functions
    pub fn get_task_result(&self, task_id: u64) -> Option<Task> {
        self.completed_tasks.get(&task_id).map(|t| t.clone())
    }
    
    pub fn get_active_task(&self, task_id: u64) -> Option<Task> {
        self.active_tasks.get(&task_id).map(|t| t.clone())
    }

    pub fn get_assigned_tasks(&self, node_id: AccountId) -> Vec<Task> {
        self.active_tasks.values()
            .filter(|task| task.assignee.as_ref() == Some(&node_id.to_string()))
            .map(|task| task.clone())
            .collect()
    }

    pub fn get_node_info(&self, node_id: AccountId) -> Option<NodeInfo> {
        self.nodes.get(&node_id).map(|n| n.clone())
    }

    pub fn get_active_nodes(&self) -> Vec<NodeInfo> {
        let current_time = env::block_timestamp();
        
        self.nodes.values()
            .filter(|node| node.is_active && current_time - node.last_heartbeat < HEARTBEAT_TIMEOUT)
            .map(|node| node.clone())
            .collect()
    }

    pub fn get_pending_tasks(&self) -> Vec<Task> {
        let mut pending_tasks = Vec::new();
        for task_id in self.pending_tasks.iter() {
            if let Some(task) = self.active_tasks.get(&task_id) {
                if task.status == TaskStatus::Pending {
                    pending_tasks.push(task.clone());
                }
            }
        }
        pending_tasks
    }

    pub fn get_task_count(&self) -> u64 {
        self.task_counter
    }

    pub fn get_total_rewards_distributed(&self) -> U128 {
        self.total_rewards_distributed.into()
    }
    
    pub fn get_contract_stats(&self) -> (u64, u64, u64, u64, bool) {
        let active_nodes = self.get_active_nodes().len() as u64;
        let total_nodes = self.nodes.len() as u64;
        let active_tasks = self.active_tasks.len() as u64;
        let completed_tasks = 0u64; // LookupMap doesn't have len()
        
        (active_nodes, total_nodes, active_tasks, completed_tasks, self.paused)
    }

    // Token Functions
    #[payable]
    pub fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.assert_one_yocto();
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    pub fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }

    pub fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    // Admin Functions
    #[payable]
    pub fn update_min_stake(&mut self, new_min_stake: U128) {
        self.assert_owner();
        self.assert_one_yocto();
        require!(new_min_stake.0 > 0, "Min stake must be positive");
        
        let old_stake = self.min_stake;
        self.min_stake = new_min_stake.into();
        
        log!("Min stake updated from {} to {}", old_stake, self.min_stake);
    }
    
    #[payable]
    pub fn pause_contract(&mut self) {
        self.assert_owner();
        self.assert_one_yocto();
        require!(!self.paused, "Contract already paused");
        
        self.paused = true;
        log!("Contract paused");
    }
    
    #[payable]
    pub fn unpause_contract(&mut self) {
        self.assert_owner();
        self.assert_one_yocto();
        require!(self.paused, "Contract not paused");
        
        self.paused = false;
        log!("Contract unpaused");
    }
    
    #[payable]
    pub fn update_max_tasks_per_node(&mut self, max_tasks: u32) {
        self.assert_owner();
        self.assert_one_yocto();
        require!(max_tasks > 0 && max_tasks <= 100, "Invalid max tasks per node");
        
        self.max_tasks_per_node = max_tasks;
        log!("Max tasks per node updated to {}", max_tasks);
    }
    
    #[payable]
    pub fn update_task_timeout(&mut self, timeout_duration: u64) {
        self.assert_owner();
        self.assert_one_yocto();
        require!(timeout_duration >= 300_000_000_000, "Timeout too short (min 5 minutes)"); // 5 minutes minimum
        require!(timeout_duration <= 86400_000_000_000, "Timeout too long (max 24 hours)"); // 24 hours maximum
        
        self.task_timeout_duration = timeout_duration;
        log!("Task timeout updated to {} nanoseconds", timeout_duration);
    }
    
    #[payable]
    pub fn emergency_withdraw(&mut self, amount: U128) {
        self.assert_owner();
        self.assert_one_yocto();
        require!(self.paused, "Contract must be paused for emergency withdrawal");
        
        let withdraw_amount: u128 = amount.into();
        let contract_balance = env::account_balance().as_yoctonear();
        require!(withdraw_amount <= contract_balance, "Insufficient contract balance");
        
        Promise::new(self.owner_id.clone()).transfer(NearToken::from_yoctonear(withdraw_amount));
        log!("Emergency withdrawal: {} yoctoNEAR", withdraw_amount);
    }
}