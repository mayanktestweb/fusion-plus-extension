use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::{ext_contract, NearToken};
use near_sdk::{AccountId, Promise};


#[ext_contract(ext_ft)]
pub trait FungibleToken {
    // Transfer tokens to another account
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: NearToken, memo: Option<String>);

    // Transfer tokens and call a method on receiver contract
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: NearToken,
        msg: String,
        memo: Option<String>,
    ) -> Promise;

    // View balance of an account
    fn ft_balance_of(&self, account_id: AccountId) -> NearToken;

    // View total supply
    fn ft_total_supply(&self) -> NearToken;

    // View token metadata
    fn ft_metadata(&self) -> TokenMetadata;

    // Deposit storage for an account
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;

    // View storage balance of an account
    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance>;
}

// Optional: define metadata and storage balance types
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<String>,
    pub decimals: u8,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: NearToken,
    pub available: NearToken,
}
