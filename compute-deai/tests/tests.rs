#[cfg(test)]
mod tests {
    use compute_deai::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, NearToken, AccountId};
    use near_contract_standards::fungible_token::Balance;

    const MIN_STAKE: Balance = 1_000_000_000_000_000_000_000_000; // 1 NEAR

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
        assert_eq!(node_info.stake.as_yoctonear(), MIN_STAKE);
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
        let mut context = get_context(accounts(3), task_cost);
        testing_env!(context.build());
        
        contract.submit_task(
            r#"{"model": "gpt2", "input": "Hello world", "task_type": "inference"}"#.to_string(),
            task_cost.into(),
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
        let mut context = get_context(accounts(3), task_cost);
        testing_env!(context.build());
        
        contract.submit_task(
            r#"{"model": "gpt2", "input": "Hello world", "task_type": "inference"}"#.to_string(),
            task_cost.into(),
        );
        
        // Submit result as node
        let mut context = get_context(accounts(2), 0);
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
        
        let mut context = get_context(accounts(4), task_cost);
        testing_env!(context.build());
        contract.submit_task("Task 1".to_string(), task_cost.into());
        
        let mut context = get_context(accounts(4), task_cost);
        testing_env!(context.build());
        contract.submit_task("Task 2".to_string(), task_cost.into());
        
        // Check both tasks were assigned
        assert_eq!(contract.get_task_count(), 2);
        
        let node2_tasks = contract.get_assigned_tasks(accounts(2));
        let node3_tasks = contract.get_assigned_tasks(accounts(3));
        
        // One of the nodes should have a task assigned
        assert!(node2_tasks.len() == 1 || node3_tasks.len() == 1);
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
        let mut context = get_context(accounts(3), task_cost);
        testing_env!(context.build());
        
        contract.submit_task("Test task".to_string(), task_cost.into());
        
        let mut context = get_context(accounts(2), 0);
        testing_env!(context.build());
        
        contract.submit_result(0, "proof_hash".to_string(), "result".to_string());
        
        // Check tokens were minted
        assert_eq!(contract.ft_balance_of(accounts(2)).0, task_cost);
        assert_eq!(contract.ft_total_supply().0, task_cost);
        assert_eq!(contract.get_total_rewards_distributed().0, task_cost);
    }
}