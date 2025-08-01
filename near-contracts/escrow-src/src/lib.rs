use near_sdk::{borsh::{BorshDeserialize, BorshSerialize}, env, log, near_bindgen, require, serde::{Deserialize, Serialize}, store::LookupMap, AccountId, NearSchema, NearToken, PromiseOrValue};
use shared_lib::{immutables::Immutables, merkle_verifier::MerkleVerifier};


// Main User Order
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct MakerOrder {
    root_hash: String,              // hashlock for single, merkle_root for multi fill
    token: AccountId,               // token used by maker to make exchange
    total_amount: NearToken,        // total tokens maker is putting
    parts: u16,                     // parts the order is devided in (default 1)
    filled_amount: NearToken,       // taker placed amount
    withdrawn_amount: NearToken,    // withdrawn amount
    maker: AccountId,               // maker account
    expiration: u64                 // timestamp beyond which user can run do self withdrawal
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct ResolverOrderFill {
    immutables: Immutables
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct EscrowSrc {
    // orders placed by makers
    // delete entry once order amount is fully withdrawn
    // entry key: maker_order.root_hash
    pub makers_orders: LookupMap<String, MakerOrder>,

    // fill-orders placed by resolvers
    // delete entry once a fill order is withdrawn or cancelled
    // entry key: resolver_order_fill.immutables.hash()
    pub resolver_orders: LookupMap<String, ResolverOrderFill>
}

impl Default for EscrowSrc {
    fn default() -> Self {
        Self {
            makers_orders: LookupMap::new(b"m"),
            resolver_orders: LookupMap::new(b"r")
        }
    }
}

#[near_bindgen]
impl EscrowSrc {

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

        if maker_order.maker != sender_id {
            log!("Maker order does not match the sender ID: {}", sender_id);
            return PromiseOrValue::Value(amount);
        }

        if maker_order.total_amount < amount {
            log!("Maker order total amount is less than the transferred amount");
            return PromiseOrValue::Value(amount);
        }

        // validate the token itself
        if maker_order.token != env::predecessor_account_id() {
            log!("Invalid token: {}", maker_order.token);
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


    #[payable]
    pub fn create_resolver_fill_order(
        &mut self,
        immutables: Immutables, 
        idx: Option<u16>,                           // index of secret being used (multi-fill)
        merkle_proof: Option<Vec<String>>           // merkle proof (multi-fill)
    ) {
        // first check if safty deposit is there
        let attached_deposit = env::attached_deposit();
        require!(attached_deposit == immutables.safty_deposit,
        "Invalid or no safty deposit...!");

        // check if maker order exists to fill
        require!(self.makers_orders.contains_key(&immutables.order_root_hash), "Order doesn't exist...");

        let maker_order = self.makers_orders.get(&immutables.order_root_hash).unwrap();
        let mut total_amount = NearToken::from_yoctonear(0);
        let mut filled_amount = NearToken::from_yoctonear(0);
        let making_amount = &immutables.making_amount;
        let root_hash = &immutables.order_root_hash;
        let parts = maker_order.parts;

        if let Some(maker_order) = self.makers_orders.get(&immutables.order_root_hash) {
            total_amount = maker_order.total_amount.clone();
            filled_amount = maker_order.filled_amount.clone();
        }


        // TODO: validate Immutables


        // if its multi fill check if idx of secret is correct
        // then verify merkle proof using (haslock, idx and )
        if maker_order.parts > 1 {
            // ensure new order fill the remaings of last partial fill
            require!(Self::completes_last_partial_fill(&total_amount, &filled_amount, making_amount, parts),
                "Making amount doesn't cover remaining amount of last partial fill..."
            );

            // ensure the index of fill is valid
            let idx = idx.expect("Invalid order fill index...");
            let valid_index = Self::compute_valid_index(&total_amount, &filled_amount, making_amount, parts);
            require!(idx == valid_index, "Invalid order fill index...");

            // verify merkle proof
            let merkle_proof = merkle_proof.expect("Merkle proof not provided...");
            let leaf_hex = MerkleVerifier::indexed_secret_hash_string(idx, &immutables.hashlock);
            let is_valid_merkle_proof = MerkleVerifier::verify(&leaf_hex, merkle_proof, &immutables.order_root_hash);
            require!(is_valid_merkle_proof, "Invalid proof or hashlock...");
        } else {
            require!(immutables.hashlock == maker_order.root_hash, "Invalid Hashlock...");
        }

        // place the order
        self.resolver_orders.insert(immutables.hash(), ResolverOrderFill { immutables: immutables.clone() });

        // add as filled amount in maker order
        if let Some(value) = self.makers_orders.get_mut(&root_hash.clone()) {
            let filled_amount_u128: u128 = filled_amount.as_yoctonear();
            let making_amount_u128: u128 = making_amount.as_yoctonear();
            let new_filled_amount = filled_amount_u128.checked_add(making_amount_u128)
                .expect("Overflow when calculating new filled amount");
            value.filled_amount = NearToken::from_yoctonear(new_filled_amount);
        }
    }


    /**
     * @dev The function works on the time interval highlighted with capital letters:
     * ---- contract deployed --/-- finality --/-- PRIVATE WITHDRAWAL --/-- PUBLIC WITHDRAWAL --/--
     * --/-- private cancellation --/-- public cancellation ----
     */
    pub fn withdraw(&mut self, secret: String, immutables: Immutables) {
        // only taker can call it
        require!(env::predecessor_account_id() == immutables.taker, "Only taker can withdraw...",);
        require!(Self::_only_after(immutables.timelock.src_withdrawal));
        require!(Self::_only_before(immutables.timelock.src_cancellation));

        // validate secret
        require!(Self::validate_secret(secret, immutables.hashlock), "Invalid secret...");

        unimplemented!()
    }


    /**
     * @dev The function works on the time interval highlighted with capital letters:
     * ---- contract deployed --/-- finality --/-- PRIVATE WITHDRAWAL --/-- PUBLIC WITHDRAWAL --/--
     * --/-- private cancellation --/-- public cancellation ----
     */
    pub fn withdraw_to(&mut self, secret: String, immutables: Immutables) {
        // only taker can call it
        require!(env::predecessor_account_id() == immutables.taker, "Only taker can withdraw...",);
        require!(Self::_only_after(immutables.timelock.src_withdrawal));
        require!(Self::_only_before(immutables.timelock.src_cancellation));
        
        // validate secret
        require!(Self::validate_secret(secret, immutables.hashlock), "Invalid secret...");
        
        unimplemented!()
    }

    /**
     * @dev The function works on the time interval highlighted with capital letters:
     * ---- contract deployed --/-- finality --/-- private withdrawal --/-- PUBLIC WITHDRAWAL --/--
     * --/-- private cancellation --/-- public cancellation ----
     */
    pub fn pubic_withdraw(&mut self, secret: String, immutables: Immutables) {
        // anyone can call it
        
        require!(Self::_only_after(immutables.timelock.src_public_withdrawal));
        require!(Self::_only_before(immutables.timelock.src_cancellation));
        
        // validate secret
        require!(Self::validate_secret(secret, immutables.hashlock), "Invalid secret...");
        
        unimplemented!()
    }
    
    /**
     * @dev The function works on the time intervals highlighted with capital letters:
     * ---- contract deployed --/-- finality --/-- private withdrawal --/-- public withdrawal --/--
     * --/-- PRIVATE CANCELLATION --/-- PUBLIC CANCELLATION ----
     */
    pub fn cancel(&mut self, immutables: Immutables) {
        // only taker can call it
        require!(env::predecessor_account_id() == immutables.taker, "Only taker can cancel...");
        require!(Self::_only_after(immutables.timelock.src_cancellation));

        unimplemented!()
    }

    /**
     * @dev The function works on the time intervals highlighted with capital letters:
     * ---- contract deployed --/-- finality --/-- private withdrawal --/-- public withdrawal --/--
     * --/-- private cancellation --/-- PUBLIC CANCELLATION ----
     */
    pub fn public_cancel(&mut self, immutables: Immutables) {
        // anyone can call it

        // only after Timelock.src_cancellation
        require!(Self::_only_after(immutables.timelock.src_public_cancellation));
        unimplemented!()
    }
    
}



// block of static functions
#[near_bindgen]
impl EscrowSrc  {
    fn completes_last_partial_fill(
        total_amount: &NearToken,
        filled_amount: &NearToken,
        making_amount: &NearToken,
        parts: u16
    ) -> bool {
        let total_amount_u128: u128 = total_amount.as_yoctonear();
        let filled_amount_u128: u128 = filled_amount.as_yoctonear();
        let making_amount_u128: u128 = making_amount.as_yoctonear();
        let parts_u128: u128 = parts.into();

        let x = filled_amount_u128.checked_mul(parts_u128)
            .expect("Overflow!");

        let x = x.checked_div(total_amount_u128)
            .expect("Overflow");

        let remaning = filled_amount_u128 - (total_amount_u128/parts_u128)*x;

        making_amount_u128 > remaning
    }
    
    // Compute valid index for a fill
    fn compute_valid_index(
        total_amount: &NearToken,
        filled_amount: &NearToken,
        making_amount: &NearToken,
        parts: u16
    ) -> u16 {
        let total_amount_u128: u128 = total_amount.as_yoctonear();
        let filled_amount_u128: u128 = filled_amount.as_yoctonear();
        let making_amount_u128: u128 = making_amount.as_yoctonear();
        let parts_u128: u128 = parts.into();

        require!(total_amount_u128 > 0, "Total amount must be greater than zero");

        let current_filled = filled_amount_u128.checked_add(making_amount_u128)
            .expect("Overflow when calculating current filled amount");

        require!(current_filled > 0, "Current filled amount must be positive");

        // If its completing full order use the last secret        
        if current_filled == total_amount_u128 {
            return parts;
        }


        // As per the formula: index = (filled_amount + making_amount - 1) * parts / total_amount
        // This calculates a 0-based index.
        let numerator = (current_filled - 1)
            .checked_mul(parts_u128)
            .expect("Overflow when calculating numerator for index");
        
        let index = numerator / total_amount_u128;

        // The result should be less than `parts`. Since `parts` is u16, this conversion is safe.
        index as u16
    }

    fn _only_after(timestamp: u64) -> bool {
        env::block_timestamp() > timestamp
    }

    fn _only_before(timestamp: u64) -> bool {
        env::block_timestamp() < timestamp
    }

    fn validate_secret(secret: String, hashlock: String) -> bool {
        let hash = env::keccak256(secret.as_bytes());
        let hash_hex = hex::encode(hash);
        let hashlock = match hashlock.starts_with("0x") {
            true => hashlock[2..].to_string(),
            false => hashlock.to_string()
        };
        hash_hex == hashlock
    }
}