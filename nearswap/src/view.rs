//! View functions for the contract.

use near_sdk::json_types::U128;
use near_sdk::{near_bindgen, AccountId};

use crate::*;

#[near_bindgen]
impl NearSwap {
    /// Returns current balance of given token for given user. If there is nothing recorded, returns 0.
    pub(crate) fn internal_get_deposit(
        &self,
        sender_id: &AccountId,
        token_id: &AccountId,
    ) -> Balance {
        self.deposits
            .get(sender_id)
            .and_then(|d| AccountDepositV1::from(d).tokens.get(token_id).cloned())
            .unwrap_or_default()
    }

    /// Returns balance of the deposit for given user outside of any pools.
    pub fn get_deposit_token(&self, account_id: AccountId, token_id: AccountId) -> U128 {
        self.internal_get_deposit(&account_id, &token_id).into()
    }

    /// Returns near balance of the deposit for given user outside of any pools.
    pub fn get_deposit_near(&self, account_id: AccountId) -> U128 {
        let res = self.get_deposit(&account_id);
        U128(res.ynear)
    }
}
