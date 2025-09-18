use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{near, AccountId, env, Promise, json_types::U128, PanicOnDefault, NearToken};
use near_contract_standards::fungible_token::{FungibleToken, FungibleTokenCore, Balance};
use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};


pub const MIN_STAKE_YOCTO: u128 = 1_000_000_000_000_000_000_000_000; // 1 NEAR
pub const STORAGE_COST: Balance = 1_000_000_000_000_000_000_000; // 0.001 NEAR

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NodeInfo {
    pub account_id: AccountId,
    pub stake: NearToken,
    pub public_ip: String,
    pub gpu_specs: String,
    pub cpu_specs: String,
    pub api_endpoint: String,
    pub is_active: bool,
    pub last_heartbeat: u64,
    pub total_tasks_completed: u64,
    pub reputation_score: u32,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Task {
    pub id: u64,
    pub description: String,  // JSON: {model: "url", input: "data", task_type: "inference"}
    pub assignee: Option<AccountId>,
    pub status: TaskStatus,
    pub output: Option<String>,
    pub proof_hash: Option<String>,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub reward_amount: Balance,
    pub requester: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct DeAICompute {
    pub nodes: HashMap<AccountId, NodeInfo>,
    pub tasks: VecDeque<Task>,
    pub completed_tasks: HashMap<u64, Task>,
    pub task_counter: u64,
    pub token: FungibleToken,
    pub min_stake: NearToken,
    pub total_rewards_distributed: Balance,
    pub owner_id: AccountId,
}

#[near]
impl DeAICompute {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        let mut token = FungibleToken::new(b"t".to_vec());
        token.internal_register_account(&owner_id);
        
        Self {
            nodes: HashMap::new(),
            tasks: VecDeque::new(),
            completed_tasks: HashMap::new(),
            task_counter: 0,
            token,
            min_stake: NearToken::from_yoctonear(MIN_STAKE_YOCTO),
            total_rewards_distributed: 0,
            owner_id,
        }
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
        let account_id = env::predecessor_account_id();
        let stake = env::attached_deposit();
        
        assert!(stake >= self.min_stake, "Insufficient stake. Minimum: {} yoctoNEAR", self.min_stake.as_yoctonear());
        assert!(!self.nodes.contains_key(&account_id), "Node already registered");
        assert!(!public_ip.is_empty(), "Public IP cannot be empty");
        assert!(!api_endpoint.is_empty(), "API endpoint cannot be empty");

        // Validate IP is unique
        for (_, node) in &self.nodes {
            assert!(node.public_ip != public_ip, "IP address already registered");
        }

        let node_info = NodeInfo {
            account_id: account_id.clone(),
            stake,
            public_ip,
            gpu_specs,
            cpu_specs,
            api_endpoint,
            is_active: true,
            last_heartbeat: env::block_timestamp(),
            total_tasks_completed: 0,
            reputation_score: 100, // Start with base reputation
        };

        self.nodes.insert(account_id.clone(), node_info);
        
        // Register account for token rewards
        if !self.token.accounts.contains_key(&account_id) {
            self.token.internal_register_account(&account_id);
        }
    }

    pub fn heartbeat(&mut self) {
        let account_id = env::predecessor_account_id();
        let mut node = self.nodes.get(&account_id).expect("Node not registered").clone();
        
        node.last_heartbeat = env::block_timestamp();
        node.is_active = true;
        
        self.nodes.insert(account_id, node);
    }

    pub fn deactivate_node(&mut self) {
        let account_id = env::predecessor_account_id();
        let mut node = self.nodes.get(&account_id).expect("Node not registered").clone();
        
        // Return staked amount
        Promise::new(account_id.clone()).transfer(node.stake);
        
        node.is_active = false;
        self.nodes.insert(account_id, node);
    }

    // Task Management Functions
    #[payable]
    pub fn submit_task(&mut self, description: String, estimated_compute_cost: U128) {
        let requester = env::predecessor_account_id();
        let fee = env::attached_deposit();
        let compute_cost: Balance = estimated_compute_cost.into();
        
        assert!(fee.as_yoctonear() >= compute_cost, "Insufficient payment for compute cost");
        assert!(!description.is_empty(), "Task description cannot be empty");

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
            reward_amount: compute_cost,
            requester,
        };

        self.tasks.push_back(task);
        self.task_counter += 1;
        
        // Try to assign to available node
        self.try_assign_next_task();
    }

    pub fn submit_result(&mut self, task_id: u64, proof_hash: String, output: String) {
        let account_id = env::predecessor_account_id();
        
        // Find task in active tasks
        let task_index = self.tasks.iter().position(|t| t.id == task_id)
            .expect("Task not found");
        
        let mut task = self.tasks.remove(task_index).unwrap();
        
        assert_eq!(task.assignee.as_ref(), Some(&account_id), "Not assigned to this node");
        assert_eq!(task.status, TaskStatus::Assigned, "Task not in assigned state");
        assert!(!proof_hash.is_empty(), "Proof hash cannot be empty");
        assert!(!output.is_empty(), "Output cannot be empty");

        // Update task
        task.status = TaskStatus::Completed;
        task.output = Some(output);
        task.proof_hash = Some(proof_hash);
        task.completed_at = Some(env::block_timestamp());

        // Update node stats
        let mut node = self.nodes.get(&account_id).unwrap().clone();
        node.total_tasks_completed += 1;
        node.reputation_score = std::cmp::min(1000, node.reputation_score + 10);
        self.nodes.insert(account_id.clone(), node);

        // Mint reward tokens
        self.token.internal_deposit(&account_id, task.reward_amount);
        self.total_rewards_distributed += task.reward_amount;

        // Store completed task
        self.completed_tasks.insert(task_id, task);

        // Try to assign next task
        self.try_assign_next_task();
    }

    fn try_assign_next_task(&mut self) {
        if let Some(available_node) = self.get_available_node() {
            if let Some(task) = self.tasks.front_mut() {
                if task.status == TaskStatus::Pending {
                    task.assignee = Some(available_node);
                    task.status = TaskStatus::Assigned;
                }
            }
        }
    }

    fn get_available_node(&self) -> Option<AccountId> {
        let current_time = env::block_timestamp();
        let heartbeat_timeout = 300_000_000_000; // 5 minutes in nanoseconds
        
        for (account_id, node) in &self.nodes {
            if node.is_active 
                && current_time - node.last_heartbeat < heartbeat_timeout
                && !self.node_has_active_task(account_id) {
                return Some(account_id.clone());
            }
        }
        None
    }

    fn node_has_active_task(&self, node_id: &AccountId) -> bool {
        self.tasks.iter().any(|task| 
            task.assignee.as_ref() == Some(node_id) && 
            matches!(task.status, TaskStatus::Assigned | TaskStatus::InProgress)
        )
    }

    // View Functions
    pub fn get_task_result(&self, task_id: u64) -> Option<Task> {
        self.completed_tasks.get(&task_id).cloned()
    }

    pub fn get_assigned_tasks(&self, node_id: AccountId) -> Vec<Task> {
        self.tasks.iter()
            .filter(|task| task.assignee.as_ref() == Some(&node_id))
            .cloned()
            .collect()
    }

    pub fn get_node_info(&self, node_id: AccountId) -> Option<NodeInfo> {
        self.nodes.get(&node_id).cloned()
    }

    pub fn get_active_nodes(&self) -> Vec<NodeInfo> {
        let current_time = env::block_timestamp();
        let heartbeat_timeout = 300_000_000_000; // 5 minutes
        
        self.nodes.values()
            .filter(|node| node.is_active && current_time - node.last_heartbeat < heartbeat_timeout)
            .cloned()
            .collect()
    }

    pub fn get_pending_tasks(&self) -> Vec<Task> {
        self.tasks.iter()
            .filter(|task| task.status == TaskStatus::Pending)
            .cloned()
            .collect()
    }

    pub fn get_task_count(&self) -> u64 {
        self.task_counter
    }

    pub fn get_total_rewards_distributed(&self) -> U128 {
        self.total_rewards_distributed.into()
    }

    // Token Functions
    pub fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    pub fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }

    pub fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    // Admin Functions
    pub fn update_min_stake(&mut self, new_min_stake: U128) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only owner can update min stake");
        self.min_stake = NearToken::from_yoctonear(new_min_stake.into());
    }
}