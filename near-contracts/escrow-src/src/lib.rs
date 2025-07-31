use near_sdk::{borsh::{BorshDeserialize, BorshSerialize}, env, log, near_bindgen, serde::{Deserialize, Serialize}, store::LookupMap, AccountId, NearSchema, NearToken, PromiseOrValue};
use shared_lib::immutables::{Immutables, TimeLock};


// Main User Order
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
struct MakerOrder {
    salt: String,
    root_hash: String,
    token: AccountId,
    total_amount: NearToken,
    making_token: String,
    is_multi_fill: bool,
    parts: u16,
    filled_amount: NearToken,
    maker: String,
    expiration: u64
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
struct ResolverOrderFill {
    root_hash: String,
    immutables: Immutables
}

#[near_bindgen]
// #[derive(Default)]
pub struct EscrowSrc {

    pub makers_orders: LookupMap<String, MakerOrder>

}

impl EscrowSrc {

    fn test() {
        let timelock = env::block_timestamp();
    }

    // it will place the maker order
    pub fn place_maker_order() {

    }


    // This function is called when a fungible token is transferred to the contract
    // It expects a hex-encoded string representing the maker order
    // If the order is valid, it stores the order in the lookup map
    // If the order is invalid, it returns the transferred amount back to the sender
    pub fn ft_on_transfer(
        &mut self, 
        sender_id: AccountId, 
        amount: NearToken, 
        msg: String
    ) -> PromiseOrValue<NearToken>  {

        // Validate hex string
        let bytes_hex = hex::decode(msg);
        if bytes_hex.is_err() {
            log!("Invlid hex string provided");
            return PromiseOrValue::Value(amount);
        }

        // Deserialize the maker order from the hex string
        // This will fail if the data is not a valid MakerOrder
        let maker_order = MakerOrder::try_from_slice(&bytes_hex.unwrap());
        if maker_order.is_err() {
            log!("Invalid maker order data");
            return PromiseOrValue::Value(amount);
        }

        let maker_order = maker_order.unwrap();

        if maker_order.maker != sender_id.to_string() {
            log!("Maker order does not match the sender ID: {}", sender_id);
            return PromiseOrValue::Value(amount);
        }

        if maker_order.expiration < env::block_timestamp() + 500 {
            log!("expiration is too close: {}", maker_order.expiration);
            return PromiseOrValue::Value(amount);
        }



        // Return unused tokens if any
        let unused_tokens = amount.checked_sub(maker_order.total_amount);

        if unused_tokens.is_some() && unused_tokens.unwrap() > NearToken::from_yoctonear(0) {
            log!("Unused tokens detected: {}", unused_tokens.unwrap());
            return PromiseOrValue::Value(unused_tokens.unwrap());
        }

        // Store the maker order in the lookup map
        self.makers_orders.insert(maker_order.root_hash.clone(), maker_order);

        PromiseOrValue::Value(NearToken::from_yoctonear(0))    
    }
}