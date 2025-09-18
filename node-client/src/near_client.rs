use anyhow::{Result, Context};
use near_jsonrpc_client::{JsonRpcClient, methods, auth};
use near_primitives::{
    account::{AccessKey, AccessKeyPermission},
    hash::CryptoHash,
    transaction::{Action, FunctionCallAction, Transaction, SignedTransaction},
    types::{AccountId, Balance, Gas, Nonce, BlockReference},
    views::{FinalExecutionOutcomeView, AccessKeyView},
};
use near_crypto::{InMemorySigner, KeyType, PublicKey, SecretKey, Signature};
use serde_json::{Value, json};
use log::{info, warn, error, debug};
use std::str::FromStr;
use crate::config::NodeConfig;

pub struct NearClient {
    client: JsonRpcClient,
    signer: InMemorySigner,
    contract_id: AccountId,
}

#[derive(serde::Deserialize, Debug)]
pub struct TaskInfo {
    pub id: u64,
    pub description: String,
    pub assignee: Option<AccountId>,
    pub status: String,
    pub created_at: u64,
    pub reward_amount: String,
    pub requester: AccountId,
}

#[derive(serde::Deserialize, Debug)]
pub struct NodeInfo {
    pub account_id: AccountId,
    pub stake: String,
    pub public_ip: String,
    pub gpu_specs: String,
    pub cpu_specs: String,
    pub api_endpoint: String,
    pub is_active: bool,
    pub last_heartbeat: u64,
    pub total_tasks_completed: u64,
    pub reputation_score: u32,
}

impl NearClient {
    pub async fn new(config: &NodeConfig) -> Result<Self> {
        let client = JsonRpcClient::connect(&config.near.rpc_url);
        
        let secret_key = SecretKey::from_str(&config.node.private_key)
            .context("Invalid private key format")?;
        
        let account_id = AccountId::from_str(&config.node.account_id)
            .context("Invalid account ID format")?;
        
        let signer = InMemorySigner::from_secret_key(account_id, secret_key);
        
        let contract_id = AccountId::from_str(&config.near.contract_account_id)
            .context("Invalid contract account ID")?;
        
        Ok(Self {
            client,
            signer,
            contract_id,
        })
    }
    
    pub async fn register_node(
        &self,
        public_ip: &str,
        gpu_specs: &str,
        cpu_specs: &str,
        api_endpoint: &str,
        stake_amount: Balance,
    ) -> Result<FinalExecutionOutcomeView> {
        info!("Registering node with stake: {} yoctoNEAR", stake_amount);
        
        let args = json!({
            "public_ip": public_ip,
            "gpu_specs": gpu_specs,
            "cpu_specs": cpu_specs,
            "api_endpoint": api_endpoint,
        });
        
        self.call_contract_method(
            "register_node",
            args,
            300_000_000_000_000, // 300 TGas
            stake_amount,
        ).await
    }
    
    pub async fn heartbeat(&self) -> Result<FinalExecutionOutcomeView> {
        debug!("Sending heartbeat");
        
        self.call_contract_method(
            "heartbeat",
            json!({}),
            30_000_000_000_000, // 30 TGas
            0,
        ).await
    }
    
    pub async fn get_assigned_tasks(&self) -> Result<Vec<TaskInfo>> {
        debug!("Fetching assigned tasks");
        
        let result = self.view_contract_method(
            "get_assigned_tasks",
            json!({ "node_id": self.signer.account_id }),
        ).await?;
        
        let tasks: Vec<TaskInfo> = serde_json::from_value(result)
            .context("Failed to parse tasks response")?;
        
        Ok(tasks)
    }
    
    pub async fn submit_result(
        &self,
        task_id: u64,
        proof_hash: &str,
        output: &str,
    ) -> Result<FinalExecutionOutcomeView> {
        info!("Submitting result for task {}", task_id);
        
        let args = json!({
            "task_id": task_id,
            "proof_hash": proof_hash,
            "output": output,
        });
        
        self.call_contract_method(
            "submit_result",
            args,
            100_000_000_000_000, // 100 TGas
            0,
        ).await
    }
    
    pub async fn get_node_info(&self) -> Result<Option<NodeInfo>> {
        debug!("Fetching node info");
        
        let result = self.view_contract_method(
            "get_node_info",
            json!({ "node_id": self.signer.account_id }),
        ).await?;
        
        if result.is_null() {
            return Ok(None);
        }
        
        let node_info: NodeInfo = serde_json::from_value(result)
            .context("Failed to parse node info response")?;
        
        Ok(Some(node_info))
    }
    
    pub async fn deactivate_node(&self) -> Result<FinalExecutionOutcomeView> {
        info!("Deactivating node");
        
        self.call_contract_method(
            "deactivate_node",
            json!({}),
            50_000_000_000_000, // 50 TGas
            0,
        ).await
    }
    
    async fn call_contract_method(
        &self,
        method_name: &str,
        args: Value,
        gas: Gas,
        deposit: Balance,
    ) -> Result<FinalExecutionOutcomeView> {
        let access_key = self.get_access_key().await?;
        
        let transaction = Transaction {
            signer_id: self.signer.account_id.clone(),
            public_key: self.signer.public_key(),
            nonce: access_key.nonce + 1,
            receiver_id: self.contract_id.clone(),
            block_hash: self.get_latest_block_hash().await?,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: method_name.to_string(),
                args: args.to_string().into_bytes(),
                gas,
                deposit,
            }))],
        };
        
        let signed_transaction = SignedTransaction::new(
            self.signer.sign(&transaction.get_hash_and_size().0),
            transaction,
        );
        
        let request = methods::send_tx::RpcSendTransactionRequest {
            signed_transaction,
            wait_until: near_primitives::views::TxExecutionStatus::Final,
        };
        
        let response = self.client.call(request).await
            .context("Failed to send transaction")?;
        
        if let Some(failure) = &response.status.as_failure() {
            error!("Transaction failed: {:?}", failure);
            anyhow::bail!("Transaction failed: {:?}", failure);
        }
        
        Ok(response)
    }
    
    async fn view_contract_method(
        &self,
        method_name: &str,
        args: Value,
    ) -> Result<Value> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: self.contract_id.clone(),
                method_name: method_name.to_string(),
                args: args.to_string().into_bytes().into(),
            },
        };
        
        let response = self.client.call(request).await
            .context("Failed to query contract")?;
        
        if let near_primitives::views::QueryResponseKind::CallResult(result) = response.kind {
            let value: Value = serde_json::from_slice(&result.result)
                .context("Failed to parse view result")?;
            Ok(value)
        } else {
            anyhow::bail!("Unexpected query response type");
        }
    }
    
    async fn get_access_key(&self) -> Result<AccessKeyView> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: self.signer.account_id.clone(),
                public_key: self.signer.public_key(),
            },
        };
        
        let response = self.client.call(request).await
            .context("Failed to get access key")?;
        
        if let near_primitives::views::QueryResponseKind::AccessKey(access_key) = response.kind {
            Ok(access_key)
        } else {
            anyhow::bail!("Unexpected access key response type");
        }
    }
    
    async fn get_latest_block_hash(&self) -> Result<CryptoHash> {
        let request = methods::block::RpcBlockRequest {
            block_reference: BlockReference::latest(),
        };
        
        let response = self.client.call(request).await
            .context("Failed to get latest block")?;
        
        Ok(response.header.hash)
    }
    
    pub async fn get_account_balance(&self) -> Result<Balance> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccount {
                account_id: self.signer.account_id.clone(),
            },
        };
        
        let response = self.client.call(request).await
            .context("Failed to get account info")?;
        
        if let near_primitives::views::QueryResponseKind::ViewAccount(account) = response.kind {
            Ok(account.amount)
        } else {
            anyhow::bail!("Unexpected account response type");
        }
    }
}