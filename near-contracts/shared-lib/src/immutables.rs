use near_sdk::{borsh::{BorshDeserialize, BorshSerialize}, serde::{Deserialize, Serialize}, NearSchema, NearToken};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Immutables {
    pub order_root_hash: String,        // root_hash of maker order to fill
    pub hashlock: String,               // hash lock of this part of order fill
    pub making_token: String,           // token used by maker to make exchange
    pub taking_token: String,           // token that user wants 
    pub making_amount: NearToken,       // total tokens maker is putting
    pub taking_amount: NearToken,       // tokens that token is expected to receive
    pub safty_deposit: NearToken,       // resolver's safty deposit
    pub timelock: TimeLock,             // transaction timelocks
    pub maker: String,                  // maker account
    pub taker: String,                  // taker account
}


impl Immutables {
    pub fn hash(&self) -> String {
        todo!("Implement hash function for Immutables");
    }
}


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct TimeLock {
    pub src_withdrawal: u64,
    pub src_public_withdrawal: u64,
    pub src_cancellation: u64,
    pub src_public_cancellation: u64,
    pub dst_withdrawal: u64,
    pub dst_public_withdrawal: u64,
    pub dst_cancellation: u64
}

impl TimeLock {
    pub fn verify(&self) -> bool {
        todo!()
    }
}