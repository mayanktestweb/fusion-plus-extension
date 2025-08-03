use near_sdk::{borsh::{BorshDeserialize, BorshSerialize}, env, serde::{Deserialize, Serialize}, NearSchema, NearToken};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Immutables {
    pub salt: String,                   // random string to distinguish orders                     
    pub order_root_hash: String,        // root_hash of maker order to fill
    pub hashlock: String,               // hash lock of this part of order fill
    pub making_token: String,           // token used by maker to make exchange
    pub taking_token: String,           // token that user wants 
    pub making_amount: NearToken,       // total tokens maker is putting
    pub taking_amount: NearToken,       // tokens that token is expected to receive
    pub src_safty_deposit: NearToken,   // source chain safty deposit
    pub dst_safty_deposit: NearToken,   // destination chain safty deposit
    pub timelock: TimeLock,             // transaction timelocks
    pub maker: String,                  // maker account
    pub taker: String,                  // taker account
}


impl Immutables {
    pub fn hash(&self) -> String {
        let mut combined = Vec::new();
        combined.extend_from_slice(self.salt.as_bytes());
        combined.extend_from_slice(self.order_root_hash.as_bytes());
        combined.extend_from_slice(self.hashlock.as_bytes());
        combined.extend_from_slice(self.making_token.as_bytes());
        combined.extend_from_slice(self.taking_token.as_bytes());
        combined.extend_from_slice(&self.making_amount.as_yoctonear().to_be_bytes());
        combined.extend_from_slice(&self.taking_amount.as_yoctonear().to_be_bytes());
        combined.extend_from_slice(&self.src_safty_deposit.as_yoctonear().to_be_bytes());
        combined.extend_from_slice(&self.dst_safty_deposit.as_yoctonear().to_be_bytes());
        combined.extend_from_slice(&self.timelock.get_combined());
        combined.extend_from_slice(self.maker.as_bytes());
        combined.extend_from_slice(self.taker.as_bytes());
        let hash = env::keccak256(&combined);
        hex::encode(hash)
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

    pub fn get_combined(&self) -> Vec<u8> {
        let mut combined = Vec::new();
        combined.extend_from_slice(&self.src_withdrawal.to_be_bytes());
        combined.extend_from_slice(&self.src_public_withdrawal.to_be_bytes());
        combined.extend_from_slice(&self.src_cancellation.to_be_bytes());
        combined.extend_from_slice(&self.src_public_cancellation.to_be_bytes());
        combined.extend_from_slice(&self.dst_withdrawal.to_be_bytes());
        combined.extend_from_slice(&self.dst_public_withdrawal.to_be_bytes());
        combined.extend_from_slice(&self.dst_cancellation.to_be_bytes());
        combined
    }
}