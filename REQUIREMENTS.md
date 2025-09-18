# Specifications for the MVP of the Standalone dApp with Basic Smart Contracts on Near Protocol (Focus on Independent Nodes for DeAI)

## 1. Introduction

### 1.1 Project Description
This document outlines the specifications for developing a Minimum Viable Product (MVP) of a decentralized dApp for AI computation on Near Protocol, with independent nodes dedicated exclusively to DeAI (Decentralized AI) computation. These nodes will form a separate network of physical hardware (e.g., servers with GPUs managed by participants), coordinated by smart contracts on Near, but with the computational power reserved for third-party AI tasks (such as training, inference, or model processing).

The goal is to create a "distributed server" composed of these independent nodes, acting as a DePIN (Decentralized Physical Infrastructure Network) infrastructure for AI. Near will be used only for the on-chain coordination layer (registration, verification, incentives, and token transactions), without absorbing the AI computation into its main network. This ensures that the computational power is available for third-party applications, accessible via APIs or off-chain interfaces, similar to how projects like Akash or Render work, but optimized for AI on Near.

Since Near is a sharded L1 (it does not support standard L2s or rollups like Ethereum), we will not create an "L2 of Near." Instead, we leverage its sharding for scalability in coordination, while the DeAI nodes operate off-chain autonomously. If you prefer a custom chain, options like Cosmos SDK or Substrate could be used for a fully independent blockchain, but we stick with Near as selected.

### 1.2 MVP Objectives
- Demonstrate independent nodes: Hardware dedicated to AI, not integrated with Near validators.
- Test accessibility for third parties: Interfaces for external users to use the computation (e.g., submit AI tasks).
- Minimize complexity: Basic verification, with focus on off-chain compute.
- Estimated time: 3-6 months.
- Estimated cost: $50,000-$200,000 (including hardware testing for nodes; Near AI grants for non-dilutive funding).

### 1.3 MVP Scope
- **Includes**: Registration of independent nodes, staking, submission of AI tasks by third parties, off-chain execution, simple verification, token issuance, token usage/sale.
- **Excludes**: Advanced verification (full ZK/TEEs), heavy AI training (only light inference), integration with external chains, DAO governance.
- **Node Focus**: Nodes as external servers (e.g., AWS instances or PCs with GPUs), connected via client software to the Near contract.

## 2. Functional Requirements

### 2.1 Registration of Independent Nodes
- Participants register off-chain nodes (hardware dedicated to AI) by depositing collateral on Near to prevent fraud incentives.
- Requirements:
  - On-chain method: `register_node` – Requires minimum deposit (e.g., 1 NEAR), stores metadata (e.g., public IP, GPU/CPU specs, node API endpoint).
  - Validation: Verify deposit and uniqueness (e.g., no duplicate IPs).
  - Off-chain: Node must run a client daemon that connects periodically to the contract for heartbeat (proof of activity).

### 2.2 Submission of AI Tasks by Third Parties
- External users (third parties) submit AI tasks on-chain (e.g., "run inference on model X with input Y").
- Requirements:
  - Method: `submit_task` – Accepts task description (JSON: model URL, input data), pays with tokens or NEAR, assigns to available nodes (simple round-robin in MVP).
  - Assignment: Contract selects active nodes based on metadata (e.g., nodes with GPU for heavy tasks).

### 2.3 Execution and Computation Contributions
- Independent nodes execute tasks off-chain (e.g., download model from Hugging Face, process input via PyTorch/TensorFlow, generate output).
- Report results: Upload simple proof (e.g., output hash + timestamp) and result on-chain.
- Requirements:
  - Off-chain Client: Node polls the contract for assigned tasks, executes (e.g., supports light training or inference), uploads via `submit_result`.
  - On-chain method: `submit_result` – Verifies basic proof, calculates reward (e.g., based on estimated FLOPs or time).
  - Dedication to AI: Nodes configured only for AI tasks, not for Near validation.

### 2.4 Token Issuance and Management
- Native token (NEP-141) for node rewards and task payments.
- Requirements:
  - Mint: Automatic upon validating results (e.g., 1 token per completed task).
  - Transfer/Sale: Standard NEP-141; basic integration with DEX like Ref Finance.
  - Usage: `spend_tokens` to burn tokens when submitting tasks (payment for compute).
  - For third parties: Purchased/sold tokens allow access to the "node server network" (nodes aggregated as a virtual cluster).

### 2.5 Basic Verification and Access for Third Parties
- Verification: Simple on-chain (e.g., signature matching or nonce); third parties verify outputs off-chain.
- Requirements: Method `get_task_result` for third parties to retrieve outputs.
- Interface for Third Parties: Off-chain API wrapper (e.g., REST endpoint that interacts with the contract).

### 2.6 Basic User Interface
- CLI for nodes (register, poll tasks, submit results).
- Simple web dashboard (via BOS) for third parties to submit tasks and view results.

## 3. Non-Functional Requirements

### 3.1 Technical
- Language: Rust for contracts; Rust/Python for node clients (e.g., PyTorch for AI).
- Blockchain: Near testnet/mainnet for coordination.
- Nodes: Independent (e.g., min specs: NVIDIA RTX GPU, 16GB RAM); open-source client software.
- Security: Audit contracts; HTTPS for off-chain comms.
- Dependencies: near-sdk, near-contract-standards; for nodes: reqwest (HTTP), serde (JSON), torch-rs (AI).

### 3.2 Performance
- Latency: <5s for task assignment; off-chain compute variable (seconds to minutes per task).
- Scalability: Near sharding handles >4,000 TPS for coordination; up to 500 nodes in MVP.
- Costs: Near fees <0.01 per tx; nodes pay their own energy/hardware.

### 3.3 Usability and Compliance
- Accessible: Near wallets for auth; docs for node setup.
- Regulations: Tokens as utility; AI data privacy (e.g., do not store sensitive data on-chain).
- Environment: Testnet for dev; bootstrap incentives to attract initial nodes.

## 4. Architecture

### 4.1 High-Level Diagram
- **On-Chain (Near)**: Contracts for coordination (registry, task queue, verification, tokens).
- **Off-Chain Independent Nodes**: Dedicated DeAI network – Each node runs daemon: Poll tasks → Execute AI (training/processing) → Upload results. Aggregated as "virtual server" via off-chain load balancer.
- **Third Parties**: Interact via API/CLI to submit tasks, pay with tokens, receive outputs.
- Flow: Third party submits task → Contract assigns to node → Node executes off-chain → Uploads proof/output → Contract mints rewards → Third party retrieves result.

### 4.2 Components
- **Smart Contracts**: As in previous version, but add `submit_task` and `get_task_result`.
- **Node Client**: Rust/Python app: Connects via NEAR API, executes tasks (e.g., integrates Hugging Face API for models).
- **API for Third Parties**: Serverless (e.g., AWS Lambda) or BOS-based to wrap calls.
- **Verifiable Compute**: Simple hash in MVP; upgradable to NEAR AI primitives.

## 5. Smart Contract Specifications (Rust) – Updated

### 5.1 Dependencies (Cargo.toml)
Same as previous, add for tasks: `serde` for JSON.

### 5.2 Basic Structure (lib.rs) – Updated Excerpt
Add structs for tasks.

```rust
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near, AccountId, Balance, env, Promise, json_types::U128};
use near_contract_standards::fungible_token::FungibleToken;
use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Task {
    id: u64,
    description: String,  // JSON: {model: "url", input: "data"}
    assignee: Option<AccountId>,
    status: String,  // pending/completed
    output: Option<String>,
}

#[near(contract_state)]
pub struct DeAICompute {
    nodes: HashMap<AccountId, NodeInfo>,
    tasks: VecDeque<Task>,
    task_counter: u64,
    token: FungibleToken,
    min_stake: Balance,
    // ... (rest as previous)
}

#[near]
impl DeAICompute {
    #[payable]
    pub fn submit_task(&mut self, description: String) {
        let fee = env::attached_deposit();  // Or use tokens
        assert!(fee > 0, "Fee required");
        let task_id = self.task_counter;
        self.tasks.push_back(Task {
            id: task_id,
            description,
            assignee: None,
            status: "pending".to_string(),
            output: None,
        });
        self.task_counter += 1;
        // Assign to available node (simple: first active)
        if let Some(node_id) = self.get_available_node() {
            let mut task = self.tasks.back_mut().unwrap();
            task.assignee = Some(node_id);
        }
    }

    pub fn submit_result(&mut self, task_id: u64, proof: String, output: String) {
        let account = env::predecessor_account_id();
        // Verify assignee == account, valid proof
        // Mint rewards as previous
        // Update task.output and status
    }

    pub fn get_task_result(&self, task_id: u64) -> Option<String> {
        // Return output if completed
    }

    fn get_available_node(&self) -> Option<AccountId> {
        // Simple logic: iterate active nodes
    }
    // ... (previous methods)
}
```

### 5.3 Off-Chain Integration
- Node Client: Poll `get_assigned_tasks`, execute (e.g., Python subprocess for AI), upload via NEAR API.
- Example Task Exec: Download model, process input, generate output hash as proof.

## 6. Development and Testing Plan

### 6.1 Phases
- **Phase 1**: Setup, contracts with task logic.
- **Phase 2**: Node clients for AI exec (test with dummy model like linear regression).
- **Phase 3**: End-to-end testing: Submit task → Node processes → Result accessible.
- Add: Simulate independent nodes in local VMs.

### 6.2 Testing
- Unit: Task assignment, results.
- Integration: Full flow with mock nodes.
- Load: 100 tasks, 10 nodes.

## 7. Deployment and Maintenance

### 7.1 Deployment
Same as previous; deploy contract, distribute client software via GitHub.

### 7.2 Monitoring
- Near Explorer for tx; custom dashboard for active nodes and tasks.

## 8. Risks and Mitigations
- Risk: Non-dedicated nodes – Mitigation: Metadata verification, slashing for non-AI tasks.
- Risk: Inaccessible compute – Mitigation: Standardized APIs for third parties.
- Risk: Off-chain scalability – Mitigation: Near sharding for coordination; peer-to-peer nodes for compute.
