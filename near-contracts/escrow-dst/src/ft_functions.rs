use near_sdk::{
    env, near_bindgen, AccountId, Promise, Gas, ext_contract,
    PromiseResult,
};
use shared_lib::fungible_tokens::{ext_ft, StorageBalance};

const GAS_FOR_STORAGE_BALANCE_OF: Gas = Gas::from_tgas(10);
const GAS_FOR_STORAGE_DEPOSIT: Gas = Gas::from_tgas(20);
const GAS_FOR_FT_TRANSFER: Gas = Gas::from_tgas(15);
const STORAGE_DEPOSIT_AMOUNT: u128 = 1250000000000000000000; // 0.00125 NEAR

use crate::*;


#[ext_contract(ext_self)]
trait _Callbacks {
    fn on_check_storage(
        &mut self,
        token_contract: AccountId,
        receiver_id: AccountId,
        amount: NearToken,
    ) -> Promise;

    fn on_storage_deposit(
        &mut self,
        token_contract: AccountId,
        receiver_id: AccountId,
        amount: NearToken,
    ) -> Promise;
}

#[near_bindgen]
impl EscrowDst {
    #[private]
    pub fn safe_ft_transfer(
        &mut self,
        token_contract: AccountId,
        receiver_id: AccountId,
        amount: NearToken,
    ) -> Promise {
        ext_ft::ext(token_contract.clone())
            .with_static_gas(GAS_FOR_STORAGE_BALANCE_OF)
            .storage_balance_of(receiver_id.clone())
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_STORAGE_DEPOSIT)
                    .on_check_storage(token_contract, receiver_id, amount),
            )
    }

    #[private]
    pub fn on_check_storage(
        &mut self,
        token_contract: AccountId,
        receiver_id: AccountId,
        amount: NearToken,
    ) -> Promise {
        match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                let balance: Option<StorageBalance> =
                    near_sdk::serde_json::from_slice::<Option<StorageBalance>>(&value).unwrap_or(None);

                if balance.is_some() {
                    // Already registered, proceed to transfer
                    ext_ft::ext(token_contract)
                        .with_attached_deposit(NearToken::from_yoctonear(1)) // 1 yoctoNEAR for ft_transfer
                        .with_static_gas(GAS_FOR_FT_TRANSFER)
                        .ft_transfer(receiver_id, amount, None)
                } else {
                    // Not registered, need to deposit storage
                    ext_ft::ext(token_contract.clone())
                        .with_attached_deposit(NearToken::from_yoctonear(STORAGE_DEPOSIT_AMOUNT))
                        .with_static_gas(GAS_FOR_STORAGE_DEPOSIT)
                        .storage_deposit(Some(receiver_id.clone()), Some(true))
                        .then(
                            ext_self::ext(env::current_account_id())
                                .with_static_gas(GAS_FOR_FT_TRANSFER)
                                .on_storage_deposit(token_contract, receiver_id, amount),
                        )
                }
            }
            _ => env::panic_str("Failed to get storage balance"),
        }
    }

    #[private]
    pub fn on_storage_deposit(
        &mut self,
        token_contract: AccountId,
        receiver_id: AccountId,
        amount: NearToken,
    ) -> Promise {
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                // Proceed to transfer after successful registration
                ext_ft::ext(token_contract)
                    .with_attached_deposit(NearToken::from_yoctonear(1)) // 1 yoctoNEAR
                    .with_static_gas(GAS_FOR_FT_TRANSFER)
                    .ft_transfer(receiver_id, amount, None)
            }
            _ => env::panic_str("Failed to register receiver for FT"),
        }
    }
}
