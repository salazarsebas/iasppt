use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{near, AccountId, Promise, json_types::U128, ext_contract, Gas, NearToken};
use serde::{Deserialize, Serialize};

// Ref Finance contract interface
#[ext_contract(ref_finance)]
trait RefFinance {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> Promise;
    
    fn add_liquidity(
        &mut self,
        token_id: AccountId,
        amount: U128,
        min_amount_out: U128,
    ) -> Promise;
    
    fn swap(
        &mut self,
        actions: Vec<SwapAction>,
        referral_id: Option<AccountId>,
    ) -> Promise;
    
    fn get_pool_info(&self, pool_id: u64) -> PoolInfo;
    fn get_deposits(&self, account_id: AccountId) -> Vec<U128>;
}

// Token contract interface for NEP-141 compliance
#[ext_contract(fungible_token)]
trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> Promise;
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
}

// Data structures for Ref Finance integration
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    pub pool_id: u64,
    pub token_in: AccountId,
    pub amount_in: Option<U128>,
    pub token_out: AccountId,
    pub min_amount_out: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolInfo {
    pub token_account_ids: Vec<AccountId>,
    pub amounts: Vec<U128>,
    pub total_fee: u32,
    pub shares_total_supply: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidityPoolConfig {
    pub pool_id: u64,
    pub token_a: AccountId,
    pub token_b: AccountId,
    pub fee_rate: u32,
    pub min_liquidity: U128,
    pub is_active: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidityPosition {
    pub pool_id: u64,
    pub shares: U128,
    pub token_a_amount: U128,
    pub token_b_amount: U128,
    pub created_at: u64,
    pub last_updated: u64,
}

// Constants for Ref Finance integration
pub const REF_FINANCE_CONTRACT: &str = "v2.ref-finance.near";
pub const DEAI_TOKEN_DECIMALS: u8 = 18;
pub const MIN_LIQUIDITY_AMOUNT: u128 = 1_000_000_000_000_000_000_000; // 1000 DEAI
pub const SLIPPAGE_TOLERANCE: u32 = 300; // 3% in basis points
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 15);
pub const GAS_FOR_SWAP: Gas = Gas(Gas::ONE_TERA.0 * 50);
pub const GAS_FOR_ADD_LIQUIDITY: Gas = Gas(Gas::ONE_TERA.0 * 100);

impl crate::DeAICompute {
    /// Initialize Ref Finance integration
    pub fn init_ref_finance_integration(&mut self, pool_id: u64) {
        self.assert_owner();
        
        let pool_config = LiquidityPoolConfig {
            pool_id,
            token_a: near_sdk::env::current_account_id(), // DEAI token
            token_b: "wrap.near".parse().unwrap(), // wNEAR
            fee_rate: 25, // 0.25%
            min_liquidity: U128(MIN_LIQUIDITY_AMOUNT),
            is_active: true,
        };
        
        // Store pool configuration
        // self.ref_pool_config = Some(pool_config);
        
        near_sdk::log!("Ref Finance integration initialized with pool ID: {}", pool_id);
    }
    
    /// Add liquidity to the DEAI/wNEAR pool on Ref Finance
    #[payable]
    pub fn add_liquidity_to_ref(
        &mut self,
        deai_amount: U128,
        min_wnear_amount: U128,
    ) -> Promise {
        self.assert_owner();
        
        let deai_amount_val: u128 = deai_amount.into();
        let attached_near = near_sdk::env::attached_deposit();
        
        assert!(deai_amount_val >= MIN_LIQUIDITY_AMOUNT, "DEAI amount too small");
        assert!(attached_near.as_yoctonear() > 0, "Must attach NEAR for liquidity");
        
        // First, transfer DEAI tokens to Ref Finance
        let transfer_msg = serde_json::json!({
            "AddLiquidity": {
                "pool_id": 1, // Assuming DEAI/wNEAR pool ID is 1
                "amounts": [deai_amount, U128(attached_near.as_yoctonear())],
                "min_amounts": [deai_amount, min_wnear_amount]
            }
        }).to_string();
        
        fungible_token::ext(near_sdk::env::current_account_id())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer_call(
                REF_FINANCE_CONTRACT.parse().unwrap(),
                deai_amount,
                Some("Adding liquidity to DEAI/wNEAR pool".to_string()),
                transfer_msg,
            )
    }
    
    /// Remove liquidity from the DEAI/wNEAR pool
    pub fn remove_liquidity_from_ref(
        &mut self,
        shares: U128,
        min_deai_amount: U128,
        min_wnear_amount: U128,
    ) -> Promise {
        self.assert_owner();
        
        // Call Ref Finance to remove liquidity
        ref_finance::ext(REF_FINANCE_CONTRACT.parse().unwrap())
            .with_static_gas(GAS_FOR_SWAP)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .remove_liquidity(
                1, // pool_id
                shares,
                vec![min_deai_amount, min_wnear_amount],
            )
    }
    
    /// Swap DEAI tokens for wNEAR on Ref Finance
    pub fn swap_deai_for_wnear(
        &mut self,
        deai_amount: U128,
        min_wnear_amount: U128,
    ) -> Promise {
        let deai_amount_val: u128 = deai_amount.into();
        
        assert!(deai_amount_val > 0, "Amount must be positive");
        assert!(
            self.token.accounts.get(&near_sdk::env::predecessor_account_id()).unwrap_or(0) >= deai_amount_val,
            "Insufficient DEAI balance"
        );
        
        // Burn DEAI tokens from user account
        self.token.internal_withdraw(&near_sdk::env::predecessor_account_id(), deai_amount_val);
        
        // Prepare swap action
        let swap_action = SwapAction {
            pool_id: 1, // DEAI/wNEAR pool
            token_in: near_sdk::env::current_account_id(),
            amount_in: Some(deai_amount),
            token_out: "wrap.near".parse().unwrap(),
            min_amount_out: min_wnear_amount,
        };
        
        // Execute swap on Ref Finance
        ref_finance::ext(REF_FINANCE_CONTRACT.parse().unwrap())
            .with_static_gas(GAS_FOR_SWAP)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .swap(
                vec![swap_action],
                None, // No referral
            )
    }
    
    /// Swap wNEAR for DEAI tokens on Ref Finance
    #[payable]
    pub fn swap_wnear_for_deai(
        &mut self,
        min_deai_amount: U128,
    ) -> Promise {
        let wnear_amount = near_sdk::env::attached_deposit();
        
        assert!(wnear_amount.as_yoctonear() > 0, "Must attach wNEAR for swap");
        
        // Prepare swap action
        let swap_action = SwapAction {
            pool_id: 1, // DEAI/wNEAR pool
            token_in: "wrap.near".parse().unwrap(),
            amount_in: Some(U128(wnear_amount.as_yoctonear())),
            token_out: near_sdk::env::current_account_id(),
            min_amount_out: min_deai_amount,
        };
        
        // Execute swap on Ref Finance
        ref_finance::ext(REF_FINANCE_CONTRACT.parse().unwrap())
            .with_static_gas(GAS_FOR_SWAP)
            .with_attached_deposit(wnear_amount)
            .swap(
                vec![swap_action],
                None, // No referral
            )
    }
    
    /// Get current pool information from Ref Finance
    pub fn get_ref_pool_info(&self, pool_id: u64) -> Promise {
        ref_finance::ext(REF_FINANCE_CONTRACT.parse().unwrap())
            .with_static_gas(Gas::ONE_TERA)
            .get_pool_info(pool_id)
    }
    
    /// Get DEAI token price in wNEAR from Ref Finance
    pub fn get_deai_price(&self) -> Promise {
        self.get_ref_pool_info(1) // Assuming DEAI/wNEAR pool ID is 1
    }
    
    /// Enable automated liquidity management
    pub fn enable_automated_liquidity(&mut self, target_ratio: u32) {
        self.assert_owner();
        
        assert!(target_ratio <= 10000, "Ratio cannot exceed 100%");
        
        // Store automated liquidity settings
        // self.automated_liquidity_enabled = true;
        // self.target_liquidity_ratio = target_ratio;
        
        near_sdk::log!("Automated liquidity management enabled with {}% target ratio", target_ratio as f64 / 100.0);
    }
    
    /// Disable automated liquidity management
    pub fn disable_automated_liquidity(&mut self) {
        self.assert_owner();
        
        // self.automated_liquidity_enabled = false;
        
        near_sdk::log!("Automated liquidity management disabled");
    }
    
    /// Calculate optimal liquidity amounts based on current pool state
    pub fn calculate_optimal_liquidity(&self, total_amount: U128) -> (U128, U128) {
        // This would typically query the current pool state and calculate optimal amounts
        // For now, we'll use a simple 50/50 split
        let half_amount = u128::from(total_amount) / 2;
        (U128(half_amount), U128(half_amount))
    }
    
    /// Handle token economics for rewards distribution
    pub fn distribute_defi_rewards(&mut self, total_rewards: U128) {
        self.assert_owner();
        
        let total_rewards_val: u128 = total_rewards.into();
        
        // Calculate distribution:
        // 70% to node operators (already handled in submit_result)
        // 20% to liquidity providers
        // 10% to platform treasury
        
        let liquidity_rewards = total_rewards_val * 20 / 100;
        let treasury_rewards = total_rewards_val * 10 / 100;
        
        // Mint tokens for liquidity rewards
        self.token.internal_deposit(&"liquidity-rewards.deai.near".parse().unwrap(), liquidity_rewards);
        
        // Mint tokens for treasury
        self.token.internal_deposit(&self.owner_id, treasury_rewards);
        
        near_sdk::log!(
            "DeFi rewards distributed: {} to liquidity providers, {} to treasury",
            liquidity_rewards,
            treasury_rewards
        );
    }
    
    /// Get token economics statistics
    pub fn get_token_economics_stats(&self) -> TokenEconomicsStats {
        let total_supply = self.token.total_supply;
        let total_rewards_distributed = self.total_rewards_distributed;
        let circulation_supply = total_supply; // Simplified for now
        
        TokenEconomicsStats {
            total_supply: U128(total_supply),
            circulating_supply: U128(circulation_supply),
            total_rewards_distributed: U128(total_rewards_distributed),
            active_nodes: self.get_active_nodes().len() as u32,
            total_tasks_completed: self.task_counter,
            avg_reward_per_task: if self.task_counter > 0 {
                U128(total_rewards_distributed / self.task_counter)
            } else {
                U128(0)
            },
        }
    }
    
    /// Update token metadata for NEP-141 compliance
    pub fn update_token_metadata(&mut self, metadata: FungibleTokenMetadata) {
        self.assert_owner();
        // Update token metadata for better Ref Finance integration
        // self.token_metadata = metadata;
    }
    
    /// Emergency functions for liquidity management
    pub fn emergency_withdraw_liquidity(&mut self, pool_id: u64) -> Promise {
        self.assert_owner();
        
        // Emergency withdrawal from Ref Finance pool
        ref_finance::ext(REF_FINANCE_CONTRACT.parse().unwrap())
            .with_static_gas(GAS_FOR_SWAP)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .emergency_withdraw(pool_id)
    }
    
    /// Callback for handling swap results
    #[private]
    pub fn on_swap_callback(
        &mut self,
        account_id: AccountId,
        amount_out: U128,
        token_out: AccountId,
    ) {
        let promise_result = near_sdk::env::promise_result(0);
        
        match promise_result {
            near_sdk::PromiseResult::Successful(_) => {
                // Mint DEAI tokens if swapping to DEAI
                if token_out == near_sdk::env::current_account_id() {
                    self.token.internal_deposit(&account_id, amount_out.into());
                }
                
                near_sdk::log!("Swap completed successfully for {}", account_id);
            }
            _ => {
                near_sdk::log!("Swap failed for {}", account_id);
                // Handle failure - potentially refund
            }
        }
    }
    
    /// Helper function to assert owner-only access
    fn assert_owner(&self) {
        assert_eq!(
            near_sdk::env::predecessor_account_id(),
            self.owner_id,
            "Only owner can call this method"
        );
    }
}

// Token economics statistics structure
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenEconomicsStats {
    pub total_supply: U128,
    pub circulating_supply: U128,
    pub total_rewards_distributed: U128,
    pub active_nodes: u32,
    pub total_tasks_completed: u64,
    pub avg_reward_per_task: U128,
}

// Token metadata structure for NEP-141 compliance
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<String>,
    pub decimals: u8,
}

// External contract interfaces for callbacks
#[ext_contract(ext_self)]
trait ExtSelf {
    fn on_swap_callback(
        &mut self,
        account_id: AccountId,
        amount_out: U128,
        token_out: AccountId,
    );
}

#[ext_contract(ref_finance_extended)]
trait RefFinanceExtended {
    fn remove_liquidity(
        &mut self,
        pool_id: u64,
        shares: U128,
        min_amounts: Vec<U128>,
    ) -> Promise;
    
    fn emergency_withdraw(&mut self, pool_id: u64) -> Promise;
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain};
    
    #[test]
    fn test_calculate_optimal_liquidity() {
        let context = VMContextBuilder::new()
            .current_account_id(accounts(0))
            .build();
        testing_env!(context);
        
        let contract = crate::DeAICompute::new(accounts(0));
        let (amount_a, amount_b) = contract.calculate_optimal_liquidity(U128(1000));
        
        assert_eq!(amount_a, U128(500));
        assert_eq!(amount_b, U128(500));
    }
    
    #[test]
    fn test_token_economics_stats() {
        let context = VMContextBuilder::new()
            .current_account_id(accounts(0))
            .build();
        testing_env!(context);
        
        let contract = crate::DeAICompute::new(accounts(0));
        let stats = contract.get_token_economics_stats();
        
        assert_eq!(stats.total_supply, U128(0));
        assert_eq!(stats.total_tasks_completed, 0);
    }
}