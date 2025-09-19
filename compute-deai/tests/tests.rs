#[cfg(test)]
mod tests {
    use compute_deai::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, NearToken, AccountId};
    use near_contract_standards::fungible_token::Balance;

    const MIN_STAKE: Balance = 1_000_000_000_000_000_000_000_000; // 1 NEAR
    const STORAGE_COST: Balance = 1_000_000_000_000_000_000_000; // 0.001 NEAR
    const ONE_YOCTO: Balance = 1;

    fn get_context(predecessor_account_id: AccountId, attached_deposit: Balance) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id)
            .attached_deposit(NearToken::from_yoctonear(attached_deposit));
        builder
    }

    #[test]
    fn test_new_contract() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let contract = DeAICompute::new(accounts(1));
        assert_eq!(contract.get_task_count(), 0);
        assert_eq!(contract.get_active_nodes().len(), 0);
    }

    #[test]
    fn test_register_node() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Test successful node registration
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        let node_info = contract.get_node_info(accounts(2)).unwrap();
        assert_eq!(node_info.public_ip, "192.168.1.100");
        assert_eq!(node_info.stake, MIN_STAKE);
        assert!(node_info.is_active);
        
        let active_nodes = contract.get_active_nodes();
        assert_eq!(active_nodes.len(), 1);
    }

    #[test]
    #[should_panic(expected = "Insufficient stake")]
    fn test_register_node_insufficient_stake() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        let mut context = get_context(accounts(2), MIN_STAKE / 2);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
    }

    #[test]
    #[should_panic(expected = "IP address already registered")]
    fn test_register_node_duplicate_ip() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register first node
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Try to register another node with same IP
        let mut context = get_context(accounts(3), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(), // Same IP
            "RTX 3080".to_string(),
            "Intel i7".to_string(),
            "http://192.168.1.100:8081".to_string(),
        );
    }

    #[test]
    fn test_submit_task() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node first
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Submit a task
        let task_cost = 100_000_000_000_000_000_000_000; // 0.1 NEAR
        let mut context = get_context(accounts(3), task_cost + STORAGE_COST);
        testing_env!(context.build());
        
        contract.submit_task(
            r#"{"model": "gpt2", "input": "Hello world", "task_type": "inference"}"#.to_string(),
            task_cost.into(),
            Some(TaskPriority::Normal),
        );
        
        assert_eq!(contract.get_task_count(), 1);
        
        // Check task was assigned to node
        let assigned_tasks = contract.get_assigned_tasks(accounts(2));
        assert_eq!(assigned_tasks.len(), 1);
        assert_eq!(assigned_tasks[0].status, TaskStatus::Assigned);
    }

    #[test]
    fn test_submit_result() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Submit a task
        let task_cost = 100_000_000_000_000_000_000_000; // 0.1 NEAR
        let mut context = get_context(accounts(3), task_cost + STORAGE_COST);
        testing_env!(context.build());
        
        contract.submit_task(
            r#"{"model": "gpt2", "input": "Hello world", "task_type": "inference"}"#.to_string(),
            task_cost.into(),
            Some(TaskPriority::Normal),
        );
        
        // Submit result as node
        let mut context = get_context(accounts(2), ONE_YOCTO);
        testing_env!(context.build());
        
        contract.submit_result(
            0, // task_id
            "abc123hash".to_string(),
            "Hello world response".to_string(),
        );
        
        // Check task was completed
        let result = contract.get_task_result(0).unwrap();
        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.output.unwrap(), "Hello world response");
        
        // Check node received reward tokens
        let balance = contract.ft_balance_of(accounts(2));
        assert_eq!(balance.0, task_cost);
        
        // Check node stats updated
        let node_info = contract.get_node_info(accounts(2)).unwrap();
        assert_eq!(node_info.total_tasks_completed, 1);
        assert_eq!(node_info.reputation_score, 110); // 100 + 10
    }

    #[test]
    fn test_heartbeat() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        let initial_heartbeat = contract.get_node_info(accounts(2)).unwrap().last_heartbeat;
        
        // Advance time and send heartbeat
        let mut context = get_context(accounts(2), 0);
        context.block_timestamp(initial_heartbeat + 60_000_000_000); // +1 minute
        testing_env!(context.build());
        
        contract.heartbeat();
        
        let updated_heartbeat = contract.get_node_info(accounts(2)).unwrap().last_heartbeat;
        assert!(updated_heartbeat > initial_heartbeat);
    }

    #[test]
    fn test_multiple_nodes_task_assignment() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register two nodes
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        let mut context = get_context(accounts(3), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.101".to_string(),
            "RTX 3080".to_string(),
            "Intel i7".to_string(),
            "http://192.168.1.101:8080".to_string(),
        );
        
        // Submit two tasks
        let task_cost = 100_000_000_000_000_000_000_000; // 0.1 NEAR
        
        let mut context = get_context(accounts(4), task_cost + STORAGE_COST);
        testing_env!(context.build());
        contract.submit_task("Task 1".to_string(), task_cost.into(), Some(TaskPriority::Normal));
        
        let mut context = get_context(accounts(4), task_cost + STORAGE_COST);
        testing_env!(context.build());
        contract.submit_task("Task 2".to_string(), task_cost.into(), Some(TaskPriority::High));
        
        // Check both tasks were assigned
        assert_eq!(contract.get_task_count(), 2);
        
        let node2_tasks = contract.get_assigned_tasks(accounts(2));
        let node3_tasks = contract.get_assigned_tasks(accounts(3));
        
        // With reputation-based assignment, the first node (higher reputation from being registered first) gets both tasks
        // since max_tasks_per_node is 5 by default
        assert!(node2_tasks.len() >= 1 || node3_tasks.len() >= 1);
        assert_eq!(node2_tasks.len() + node3_tasks.len(), 2);
    }

    #[test]
    fn test_token_operations() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Initial supply should be 0
        assert_eq!(contract.ft_total_supply().0, 0);
        
        // Register a node and complete a task to mint tokens
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Submit and complete a task
        let task_cost = 100_000_000_000_000_000_000_000; // 0.1 NEAR
        let mut context = get_context(accounts(3), task_cost + STORAGE_COST);
        testing_env!(context.build());
        
        contract.submit_task("Test task".to_string(), task_cost.into(), Some(TaskPriority::Normal));
        
        let mut context = get_context(accounts(2), ONE_YOCTO);
        testing_env!(context.build());
        
        contract.submit_result(0, "proof_hash".to_string(), "result".to_string());
        
        // Check tokens were minted
        assert_eq!(contract.ft_balance_of(accounts(2)).0, task_cost);
        assert_eq!(contract.ft_total_supply().0, task_cost);
        assert_eq!(contract.get_total_rewards_distributed().0, task_cost);
    }

    // Security and Administrative Tests
    #[test]
    #[should_panic(expected = "Contract is paused")]
    fn test_pause_functionality() {
        let context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Pause contract as owner
        let context = get_context(accounts(1), ONE_YOCTO);
        testing_env!(context.build());
        contract.pause_contract();
        
        // Try to register node - should panic
        let context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
    }
    
    #[test]
    #[should_panic(expected = "Only owner can call this method")]
    fn test_unauthorized_admin_access() {
        let context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Try to pause as non-owner
        let context = get_context(accounts(2), ONE_YOCTO);
        testing_env!(context.build());
        contract.pause_contract();
    }
    
    #[test]
    #[should_panic(expected = "Exactly 1 yoctoNEAR required for security")]
    fn test_one_yocto_security() {
        let context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node first
        let context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Try to deactivate without 1 yoctoNEAR
        let context = get_context(accounts(2), 0);
        testing_env!(context.build());
        contract.deactivate_node();
    }
    
    #[test]
    fn test_contract_stats() {
        let context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        let (active_nodes, total_nodes, active_tasks, completed_tasks, paused) = contract.get_contract_stats();
        assert_eq!(active_nodes, 0);
        assert_eq!(total_nodes, 0);
        assert_eq!(active_tasks, 0);
        assert_eq!(completed_tasks, 0);
        assert_eq!(paused, false);
        
        // Register a node
        let context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        let (active_nodes, total_nodes, active_tasks, completed_tasks, paused) = contract.get_contract_stats();
        assert_eq!(active_nodes, 1);
        assert_eq!(total_nodes, 1);
    }
    
    #[test]
    fn test_task_priority_assignment() {
        let context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node
        let context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        let task_cost = 100_000_000_000_000_000_000_000;
        
        // Submit low priority task
        let context = get_context(accounts(3), task_cost + STORAGE_COST);
        testing_env!(context.build());
        contract.submit_task("Low priority task".to_string(), task_cost.into(), Some(TaskPriority::Low));
        
        // Submit urgent priority task
        let context = get_context(accounts(3), task_cost + STORAGE_COST);
        testing_env!(context.build());
        contract.submit_task("Urgent task".to_string(), task_cost.into(), Some(TaskPriority::Urgent));
        
        // Both tasks should be assigned since max_tasks_per_node is 5
        let assigned_tasks = contract.get_assigned_tasks(accounts(2));
        assert_eq!(assigned_tasks.len(), 2);
        
        // Get the tasks to verify they exist and have correct priorities
        let task_0 = contract.get_active_task(0);
        let task_1 = contract.get_active_task(1);
        
        assert!(task_0.is_some());
        assert!(task_1.is_some());
        assert_eq!(task_0.unwrap().priority, TaskPriority::Low);
        assert_eq!(task_1.unwrap().priority, TaskPriority::Urgent);
    }
    
    #[test]
    fn test_admin_functions() {
        let context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Test update min stake
        let new_stake = 2_000_000_000_000_000_000_000_000u128;
        let context = get_context(accounts(1), ONE_YOCTO);
        testing_env!(context.build());
        contract.update_min_stake(new_stake.into());
        
        // Test update max tasks per node
        let context = get_context(accounts(1), ONE_YOCTO);
        testing_env!(context.build());
        contract.update_max_tasks_per_node(10);
        
        // Test update task timeout
        let new_timeout = 7200_000_000_000u64; // 2 hours
        let context = get_context(accounts(1), ONE_YOCTO);
        testing_env!(context.build());
        contract.update_task_timeout(new_timeout);
        
        // Test pause/unpause
        let context = get_context(accounts(1), ONE_YOCTO);
        testing_env!(context.build());
        contract.pause_contract();
        
        let context = get_context(accounts(1), ONE_YOCTO);
        testing_env!(context.build());
        contract.unpause_contract();
    }
    
    #[test]
    fn test_node_deactivation_with_active_tasks() {
        let context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node
        let context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Submit a task
        let task_cost = 100_000_000_000_000_000_000_000;
        let context = get_context(accounts(3), task_cost + STORAGE_COST);
        testing_env!(context.build());
        
        contract.submit_task("Test task".to_string(), task_cost.into(), Some(TaskPriority::Normal));
        
        // Try to deactivate node with active task - should panic
        let context = get_context(accounts(2), ONE_YOCTO);
        testing_env!(context.build());
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract.deactivate_node();
        }));
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_input_validation() {
        let context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Test empty description
        let context = get_context(accounts(2), STORAGE_COST + 1000);
        testing_env!(context.build());
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract.submit_task("".to_string(), 1000u128.into(), Some(TaskPriority::Normal));
        }));
        
        assert!(result.is_err());
        
        // Test empty IP registration
        let context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract.register_node(
                "".to_string(), // Empty IP
                "RTX 4090".to_string(),
                "Intel i9".to_string(),
                "http://192.168.1.100:8080".to_string(),
            );
        }));
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_emergency_withdraw() {
        let context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // First pause the contract
        let context = get_context(accounts(1), ONE_YOCTO);
        testing_env!(context.build());
        contract.pause_contract();
        
        // Test emergency withdraw - should succeed with sufficient balance simulation
        let withdraw_amount = 1000u128;
        
        // Set context with very high balance to simulate contract having funds
        let mut context = get_context(accounts(1), ONE_YOCTO);
        context.account_balance(NearToken::from_yoctonear(withdraw_amount * 2));
        testing_env!(context.build());
        
        // This should succeed with sufficient balance
        contract.emergency_withdraw(withdraw_amount.into());
    }
    
    #[test]
    fn test_get_active_task() {
        let context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node
        let context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Submit a task
        let task_cost = 100_000_000_000_000_000_000_000;
        let context = get_context(accounts(3), task_cost + STORAGE_COST);
        testing_env!(context.build());
        
        contract.submit_task("Test task".to_string(), task_cost.into(), Some(TaskPriority::Normal));
        
        // Get active task
        let active_task = contract.get_active_task(0);
        assert!(active_task.is_some());
        assert_eq!(active_task.unwrap().status, TaskStatus::Assigned);
        
        // Test non-existent task
        let non_existent = contract.get_active_task(999);
        assert!(non_existent.is_none());
    }
    
    #[test]
    fn test_task_timeout_and_slashing() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Submit a task
        let task_cost = 100_000_000_000_000_000_000_000;
        let mut context = get_context(accounts(3), task_cost + STORAGE_COST);
        testing_env!(context.build());
        
        contract.submit_task("Test task".to_string(), task_cost.into(), Some(TaskPriority::Normal));
        
        // Get initial node reputation
        let initial_reputation = contract.get_node_info(accounts(2)).unwrap().reputation_score;
        assert_eq!(initial_reputation, 100);
        
        // Simulate time passing beyond timeout (1 hour + buffer)
        let mut context = get_context(accounts(4), ONE_YOCTO);
        context.block_timestamp(3700_000_000_000); // 1 hour 1 minute
        testing_env!(context.build());
        
        // Timeout the task
        contract.timeout_task(0);
        
        // Check task status
        let completed_task = contract.get_task_result(0);
        assert!(completed_task.is_some());
        assert_eq!(completed_task.unwrap().status, TaskStatus::TimedOut);
        
        // Check node was slashed
        let node_info = contract.get_node_info(accounts(2)).unwrap();
        assert_eq!(node_info.reputation_score, initial_reputation - 50); // REPUTATION_LOSS = 50
        assert_eq!(node_info.slashed_amount, MIN_STAKE / 10); // 10% of stake
    }
    
    #[test]
    #[should_panic(expected = "Task has not timed out yet")]
    fn test_premature_timeout() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Submit a task
        let task_cost = 100_000_000_000_000_000_000_000;
        let mut context = get_context(accounts(3), task_cost + STORAGE_COST);
        testing_env!(context.build());
        
        contract.submit_task("Test task".to_string(), task_cost.into(), Some(TaskPriority::Normal));
        
        // Try to timeout immediately (should fail)
        let mut context = get_context(accounts(4), ONE_YOCTO);
        testing_env!(context.build());
        
        contract.timeout_task(0);
    }
    
    #[test]
    fn test_reputation_system() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Submit and complete multiple tasks to test reputation gain
        for i in 0..5 {
            let task_cost = 100_000_000_000_000_000_000_000;
            let mut context = get_context(accounts(3), task_cost + STORAGE_COST);
            testing_env!(context.build());
            
            contract.submit_task(format!("Task {}", i), task_cost.into(), Some(TaskPriority::Normal));
            
            let mut context = get_context(accounts(2), ONE_YOCTO);
            testing_env!(context.build());
            
            contract.submit_result(i, format!("proof_{}", i), format!("result_{}", i));
        }
        
        // Check reputation increased
        let node_info = contract.get_node_info(accounts(2)).unwrap();
        assert_eq!(node_info.reputation_score, 150); // 100 + (5 * 10)
        assert_eq!(node_info.total_tasks_completed, 5);
    }
    
    #[test]
    fn test_max_reputation_cap() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        // Complete many tasks to test reputation cap (MAX_REPUTATION = 1000)
        for i in 0..100 {
            let task_cost = 100_000_000_000_000_000_000_000;
            let mut context = get_context(accounts(3), task_cost + STORAGE_COST);
            testing_env!(context.build());
            
            contract.submit_task(format!("Task {}", i), task_cost.into(), Some(TaskPriority::Normal));
            
            let mut context = get_context(accounts(2), ONE_YOCTO);
            testing_env!(context.build());
            
            contract.submit_result(i, format!("proof_{}", i), format!("result_{}", i));
        }
        
        // Check reputation capped at MAX_REPUTATION
        let node_info = contract.get_node_info(accounts(2)).unwrap();
        assert_eq!(node_info.reputation_score, 1000); // MAX_REPUTATION
        assert_eq!(node_info.total_tasks_completed, 100);
    }
    
    #[test]
    fn test_ft_transfer_security() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Register a node and complete a task to mint tokens
        let mut context = get_context(accounts(2), MIN_STAKE);
        testing_env!(context.build());
        
        contract.register_node(
            "192.168.1.100".to_string(),
            "RTX 4090".to_string(),
            "Intel i9".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );
        
        let task_cost = 100_000_000_000_000_000_000_000;
        let mut context = get_context(accounts(3), task_cost + STORAGE_COST);
        testing_env!(context.build());
        
        contract.submit_task("Test task".to_string(), task_cost.into(), Some(TaskPriority::Normal));
        
        let mut context = get_context(accounts(2), ONE_YOCTO);
        testing_env!(context.build());
        
        contract.submit_result(0, "proof_hash".to_string(), "result".to_string());
        
        // Test token transfer with 1 yoctoNEAR security
        let mut context = get_context(accounts(2), ONE_YOCTO);
        testing_env!(context.build());
        
        contract.ft_transfer(accounts(3), (task_cost / 2).into(), Some("test transfer".to_string()));
        
        // Check balances
        assert_eq!(contract.ft_balance_of(accounts(2)).0, task_cost / 2);
        assert_eq!(contract.ft_balance_of(accounts(3)).0, task_cost / 2);
    }
    
    #[test]
    #[should_panic(expected = "Exactly 1 yoctoNEAR required for security")]
    fn test_ft_transfer_without_security() {
        let mut context = get_context(accounts(1), 0);
        testing_env!(context.build());
        
        let mut contract = DeAICompute::new(accounts(1));
        
        // Try to transfer without 1 yoctoNEAR
        let mut context = get_context(accounts(2), 0);
        testing_env!(context.build());
        
        contract.ft_transfer(accounts(3), 1000u128.into(), None);
    }
}