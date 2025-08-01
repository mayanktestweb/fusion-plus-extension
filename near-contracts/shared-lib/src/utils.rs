use near_sdk::{env};

pub fn _only_after(timestamp: u64) -> bool {
    env::block_timestamp() > timestamp
}

pub fn _only_before(timestamp: u64) -> bool {
    env::block_timestamp() < timestamp
}

pub fn validate_secret(secret: String, hashlock: String) -> bool {
    let hash = env::keccak256(secret.as_bytes());
    let hash_hex = hex::encode(hash);
    let hashlock = match hashlock.starts_with("0x") {
        true => hashlock[2..].to_string(),
        false => hashlock.to_string()
    };
    hash_hex == hashlock
}