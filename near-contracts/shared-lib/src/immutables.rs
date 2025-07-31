use near_sdk::{borsh::{BorshDeserialize, BorshSerialize}, serde::{Deserialize, Serialize}, NearSchema};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Immutables {
    hashlock: String,
    making_amount: u64,
    timelock: TimeLock
}


impl Immutables {
    pub fn hash(&self) -> String {
        todo!("Implement hash function for Immutables");
    }
}


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct TimeLock {
    src_withdrawal: u64,
    src_public_withdrawal: u64,
    src_cancellation: u32,
    src_public_cancellation: u32,
    dst_withdrawal: u32,
    dst_public_withdrawal: u32,
    dst_cancellation: u32,
    deployed_at: u32,
}