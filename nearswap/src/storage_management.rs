use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::{assert_one_yocto, env, near_bindgen};
use std::collections::HashMap;
use std::convert::TryInto;

use crate::constants::*;
use crate::*;

/// Implements users storage management for NearSwap.
#[near_bindgen]
impl StorageManagement for NearSwap {
    // Register the caller and store minimal deposit.
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<ValidAccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount = env::attached_deposit();
        let account_id = if let Some(a) = account_id {
            a.into()
        } else {
            env::predecessor_account_id()
        };
        let registration_only = registration_only.unwrap_or(false);
        let min_balance = self.storage_balance_bounds().min.0;
        if amount < min_balance && registration_only {
            env::panic(b"E12: Insufficient amount of NEAR attached");
        }
        if registration_only {
            // Registration only setups the account but doesn't leave space for tokens.
            if self.deposits.contains_key(&account_id) {
                env::log(format!("Account already registered").as_bytes());
                if amount > 0 {
                    Promise::new(env::predecessor_account_id()).transfer(amount);
                }
            } else {
                let refund = amount - min_balance;
                if refund > 0 {
                    Promise::new(env::predecessor_account_id()).transfer(refund);
                }

                let acc_deposit = AccountDeposit {
                    ynear: min_balance,
                    storage_used: INIT_ACCOUNT_STORAGE,
                    tokens: HashMap::new(),
                };
                self.deposits.insert(&account_id, &acc_deposit);
                return StorageBalance {
                    total: U128(min_balance),
                    available: U128(0),
                };
            }
        } else {
            self.deposit_near();
        }
        self.storage_balance_of(account_id.try_into().unwrap())
            .unwrap()
    }

    #[allow(unused_variables)]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let account_deposit = self
            .deposits
            .get(&account_id)
            .expect(ERR20_ACC_NOT_REGISTERED);

        // storage available
        let available = account_deposit.ynear - account_deposit.storage_usage();
        let amount = if let Some(a) = amount { a.0 } else { available };
        assert!(amount <= available, ERR14_NOT_ENOUGH_NEAR_DEPOSITED);
        Promise::new(account_id.clone()).transfer(amount);
        self.storage_balance_of(account_id.try_into().unwrap())
            .unwrap()
    }

    #[allow(unused_variables)]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        if let Some(account_deposit) = self.deposits.get(&account_id) {
            assert!(
                account_deposit.tokens.is_empty(),
                "ERR_STORAGE_UNREGISTER_TOKENS_NOT_EMPTY"
            );
            self.deposits.remove(&account_id);
            Promise::new(account_id.clone()).transfer(account_deposit.ynear);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: U128(INIT_ACCOUNT_STORAGE as u128 * env::storage_byte_cost()),
            max: None,
        }
    }

    // check if a user is registered by calling
    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance> {
        if self.deposits.contains_key(account_id.as_ref()) {
            let acc_deposits = self.deposits.get(account_id.as_ref()).unwrap();
            return Some(StorageBalance {
                total: U128(acc_deposits.ynear),
                available: U128(acc_deposits.ynear - acc_deposits.storage_usage()),
            });
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StorageManagement;
    use super::*;

    use near_sdk::test_utils::{VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain};

    fn init_blockchain() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
    }

    fn new_near_swap() -> NearSwap {
        let ac = AccountDeposit {
            ynear: 9900000000000000000000,
            storage_used: 10,
            tokens: HashMap::new(),
        };

        let mut near = NearSwap {
            fee_dst: "owner".to_string(),
            owner: "owner".to_string(),
            pools: UnorderedMap::new("p".into()),
            deposits: LookupMap::new("d".into()),
            whitelisted_tokens: UnorderedSet::new("w".into()),
        };
        near.deposits.insert(&"owner".to_string(), &ac);

        return near;
    }

    #[test]
    fn storage_balance_works() {
        init_blockchain();
        let near_swap = new_near_swap();

        let result = StorageManagement::storage_balance_of(
            &near_swap,
            "owner".to_string().try_into().unwrap(),
        )
        .unwrap();

        assert_eq!(result.total.0, 9900000000000000000000);
    }

    #[test]
    fn storage_balance_error() {
        init_blockchain();
        let near_swap = new_near_swap();

        let result = StorageManagement::storage_balance_of(
            &near_swap,
            "owner1".to_string().try_into().unwrap(),
        )
        .is_none();

        assert!(result, true);
    }
}
