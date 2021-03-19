use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use crate::*;

/// Implements users storage management for NearSwap.
#[near_bindgen]
impl StorageManagement for NearSwap {

    // Register the caller and store minimal deposit.
    #[payable]
    fn storage_deposit(
        &mut self,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount = env::attached_deposit();
        let account_id = env::predecessor_account_id();
        let registration_only = registration_only.unwrap_or(false);
        let min_balance = self.storage_balance_bounds().min.0;
        if amount < min_balance {
            env::panic(ERR12_NOT_ENOUGH_NEAR);
        }
        if registration_only {
            // Registration only setups the account but doesn't leave space for tokens.
            if self.deposits.contains_key(&account_id) {
                log!(ERR22_ACC_ALREADY_REGISTERED);
                if amount > 0 {
                    Promise::new(env::predecessor_account_id()).transfer(amount);
                }
            } else {
                self.deposit_near(registration_only);
                let refund = amount - min_balance;
                if refund > 0 {
                    Promise::new(env::predecessor_account_id()).transfer(refund);
                }
            }
        } else {
            self.deposit_near(registration_only);
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
        
        // storagre available
        let available = account_deposit.near - self.storage_usage();
        let amount = if let Some(a) = amount { a.0 } else { available };
        assert!(amount <= available, ERR14_NOT_ENOUGH_NEAR_DEPOSITED);
        Promise::new(account_id.clone()).transfer(amount);
        self.storage_balance_of(account_id.try_into().unwrap())
            .unwrap()
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: AccountDeposit::min_storage_usage().into(),
            max: None,
        }
    }

    // check if a user is registered by calling
    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance> {
        let acc_deposits = self
            .deposits
            .get(account_id.as_ref())
            .expect(ERR20_ACC_NOT_REGISTERED);
        Some(StorageBalance {
            total: U128(acc_deposits.amount),
            available: U128(acc_deposits.storage_available()),
        })
    }
}
