use std::str::FromStr;

use near_sdk::{borsh::{BorshDeserialize, BorshSerialize}, env, near_bindgen, require, serde::{Deserialize, Serialize}, store::LookupMap, AccountId, Gas, NearSchema, NearToken, Promise, PromiseOrValue};
use shared_lib::{fungible_tokens::ext_ft, immutables::Immutables};

pub mod ft_functions;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct ResolverOrder {
    pub immutables: Immutables,
    pub safty_deposit: NearToken
}

const ZERO_NEAR: NearToken = NearToken::from_yoctonear(0);

#[near_bindgen]
#[derive(Debug, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct EscrowDst {
    pub resolvers_orders: LookupMap<String, ResolverOrder>
}

impl Default for EscrowDst {
    fn default() -> Self {
        Self {
            resolvers_orders: LookupMap::new(b"r")
        }
    }
}

#[near_bindgen]
impl EscrowDst {
    // This function is called when a fungible token is transferred to the contract
    // It expects a hex-encoded string representing the immutable order data
    pub fn ft_on_transfer(
        &mut self, 
        sender_id: AccountId, 
        amount: NearToken, 
        msg: String
    ) -> PromiseOrValue<NearToken> {
        // Validate hex string
        let bytes_hex = hex::decode(msg).expect("Invalid hex string provided");

        let immutables = Immutables::try_from_slice(&bytes_hex)
            .expect("Invalid immutable data");

        // validate the amount of tokens and return if there are extra
        require!(immutables.taking_amount <= amount, "Insufficient amount...");

        // validate the sender
        require!(sender_id == immutables.taker, "Invalid sender...");

        // Check that the escrow cancellation will start not later than the cancellation time on the source chain.
        require!(immutables.timelock.dst_cancellation < immutables.timelock.src_cancellation, "Invalid cancellation time...");
     
        let unused_tokens = amount.checked_sub(immutables.making_amount)
            .expect("Overflow when calculating unused tokens");


        // create order and refund unused amount
        self.resolvers_orders.insert(immutables.hash(), ResolverOrder { immutables, safty_deposit: ZERO_NEAR});
        
        PromiseOrValue::Value(unused_tokens)
    }

    // This function is used by resolver to deposit the safty amount
    // For now anyone can call it as long as they do it before dst_withdrawal
    #[payable]
    pub fn deposit_safty_amount(&mut self, immutables: Immutables) {
        // safty amount should be deposited before dst withdrow
        // if failed, resolver can only cancel it after dst_cancel
        require!(shared_lib::utils::_only_before(immutables.timelock.dst_withdrawal), "Too late to deposit safty amount...");

        let attached_deposit = env::attached_deposit();
        require!(attached_deposit == immutables.dst_safty_deposit, "Invalid or no safty deposit...");

        if let Some(value) = self.resolvers_orders.get_mut(&immutables.hash()) {
            value.safty_deposit = attached_deposit;
        }
    }

    /**
     * @dev The function works on the time intervals highlighted with capital letters:
     * ---- contract deployed --/-- finality --/-- PRIVATE WITHDRAWAL --/-- PUBLIC WITHDRAWAL --/-- private cancellation ----
     */
    pub fn withdraw(&mut self, secret: String, immutables: Immutables) {
        // only taker can call it
        require!(env::predecessor_account_id() == immutables.taker, "Only taker can withdraw...");
        require!(shared_lib::utils::_only_after(immutables.timelock.dst_withdrawal));
        require!(shared_lib::utils::_only_before(immutables.timelock.dst_cancellation));

        // validate secret
        require!(shared_lib::utils::validate_secret(secret, immutables.hashlock), "Invalid secret...");

        // withdraw tokens
        let taking_token = AccountId::from_str(&immutables.taking_token).expect("invalid token...");
        let receiver_id = AccountId::from_str(&immutables.maker).expect("Invalid receiver account...");
        self.safe_ft_transfer(
            taking_token, 
            receiver_id, 
            immutables.taking_amount
        );

        // recover dst safty amount
        Promise::new(env::predecessor_account_id()).transfer(immutables.dst_safty_deposit);
    }


    /**
     * @dev The function works on the time intervals highlighted with capital letters:
     * ---- contract deployed --/-- finality --/-- private withdrawal --/-- PUBLIC WITHDRAWAL --/-- private cancellation ----
     */
    pub fn public_withdraw(&mut self,secret: String, immutables: Immutables) {
        // anyone can call it
        require!(shared_lib::utils::_only_after(immutables.timelock.dst_public_withdrawal));
        require!(shared_lib::utils::_only_before(immutables.timelock.dst_cancellation));
        
        // validate secret
        require!(shared_lib::utils::validate_secret(secret, immutables.hashlock), "Invalid secret...");
        
        // withdraw tokens
        let taking_token = AccountId::from_str(&immutables.taking_token).expect("invalid token...");
        let receiver_id = AccountId::from_str(&immutables.maker).expect("Invalid receiver account...");
        self.safe_ft_transfer(
            taking_token, 
            receiver_id, 
            immutables.taking_amount
        );

        // recover dst safty amount
        Promise::new(env::predecessor_account_id()).transfer(immutables.dst_safty_deposit);
    }


    /**
     * @dev The function works on the time interval highlighted with capital letters:
     * ---- contract deployed --/-- finality --/-- private withdrawal --/-- public withdrawal --/-- PRIVATE CANCELLATION ----
     */
    pub fn cancel(&mut self,  immutables: Immutables) {
        // only taker can call it
        require!(env::predecessor_account_id() == immutables.taker, "Only taker can cancel");
        require!(shared_lib::utils::_only_after(immutables.timelock.dst_cancellation));

        let taking_token = AccountId::from_str(&immutables.taking_token)
            .expect("invalid token...");
        ext_ft::ext(taking_token)
            .with_static_gas(Gas::from_tgas(30))
            .ft_transfer(env::predecessor_account_id(), immutables.taking_amount, Some("Order Cancelled".to_string()));
    
        // recover dst safty amount
        Promise::new(env::predecessor_account_id()).transfer(immutables.dst_safty_deposit);
    }
}