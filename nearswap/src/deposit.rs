/**********************
  DEPOSIT AND STORAGE
  MANAGEMENT
***********************/

use std::collections::HashMap;
use std::convert::TryInto;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, AccountId, Balance, PromiseOrValue, StorageUsage,
};

//use crate::errors::*;
use crate::constants::*;
use crate::ft_token::*;
use crate::*;

// token deposits are done through NEP-141 ft_transfer_call to the NEARswap contract.
#[near_bindgen]
impl FungibleTokenReceiver for NearSwap {
    /**
    Callback on receiving tokens by this contract.
    Returns zero.
    Panics when account is not registered. */
    #[allow(unused_variables)]
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token = env::predecessor_account_id();
        let sender_id = AccountId::from(sender_id);
        // TODO: chekc if token is whitelisted to avoid spam attacks.

        let mut d = self.get_deposit(&sender_id);
        d.add(&token, amount.into());
        self.deposits.insert(&sender_id, &d);
        env_log!("Deposit, {} {}", amount.0, token);

        return PromiseOrValue::Value(U128(0));
    }
}

#[near_bindgen]
impl NearSwap {
    /**
    Deposits attached NEAR.
    Panics if the sender account is not registered. */
    #[payable]
    pub fn deposit_near(&mut self) {
        let sender = env::predecessor_account_id();
        let mut d = self.get_deposit(&sender);
        let amount = env::attached_deposit();
        d.ynear += amount;
        self.deposits.insert(&sender, &d);
        env_log!("Deposit, {} yNEAR", amount);
    }

    /**
    Withdraws near from deposit.
    Requires payment of exactly one yNEAR to enforce wallet confirmation. */
    #[payable]
    pub fn withdraw_near(&mut self, amount: U128, recipient: Option<ValidAccountId>) -> Promise {
        assert_one_yocto();
        let sender = env::predecessor_account_id();
        let recipient = if let Some(a) = recipient {
            AccountId::from(a)
        } else {
            sender.clone()
        };
        env_log!("Deposit withdraw, {} yNEAR", amount.0);
        let amount = u128::from(amount);
        let mut d = self.get_deposit(&sender);
        d.remove_near(amount);
        self.deposits.insert(&sender, &d);
        Promise::new(recipient).transfer(amount)
    }

    /**
    Withdraws tokens from deposit.
    Requires payment of exactly one yNEAR to enforce wallet confirmation.
    Note: `token` doesn't need to be ValidAccountId because it's already registered. */
    #[payable]
    pub fn withdraw_token(
        &mut self,
        token: AccountId,
        amount: U128,
        recipient: Option<ValidAccountId>,
        is_contract: bool,
        tx_call_msg: String,
    ) {
        assert_one_yocto();
        let sender = env::predecessor_account_id();
        let recipient = if let Some(a) = recipient {
            AccountId::from(a)
        } else {
            sender.clone()
        };
        env_log!("Deposit withdraw, {} {}", amount.0, token);
        let mut d = self.get_deposit(&sender);
        let amount = u128::from(amount);
        d.remove(&token, amount);
        self.deposits.insert(&sender, &d);

        if is_contract {
            ext_fungible_token::ft_transfer(
                recipient.try_into().unwrap(),
                amount.into(),
                Some("NEARswap withdraw".to_string()),
                &token,
                1, // required 1yNEAR for transfers
                GAS_FOR_FT_TRANSFER,
            );
        } else {
            ext_fungible_token::ft_transfer_call(
                recipient.try_into().unwrap(),
                amount.into(),
                Some("NEARswap withdraw".to_string()),
                tx_call_msg,
                &token,
                1, // required 1yNEAR for transfers
                GAS_FOR_FT_TRANSFER,
            );
        }
    }

    #[inline]
    pub(crate) fn get_deposit(&self, from: &AccountId) -> AccountDeposit {
        self.deposits.get(from).expect(ERR20_ACC_NOT_REGISTERED)
    }

    #[inline]
    pub(crate) fn set_deposit(&mut self, from: &AccountId, d: &AccountDeposit) {
        self.deposits.insert(from, d);
    }
}

/// Account deposits information and storage cost.
#[derive(BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "test", derive(Default, Clone))]
pub struct AccountDeposit {
    /// Native amount sent to the exchange.
    /// Used for storage now, but in future can be used for trading as well.
    /// MUST be always bigger than `storage_used * STORAGE_PRICE_PER_BYTE`.
    pub ynear: Balance,
    /// Amount of storage bytes used by the account,
    pub storage_used: StorageUsage,
    /// Deposited token balances.
    pub tokens: HashMap<AccountId, Balance>,
}

impl AccountDeposit {
    /**
    deposit `token`s. If this is a first depoisit, a new record is created and the minimum
    required storage is increased. */
    pub(crate) fn add(&mut self, token: &AccountId, amount: u128) {
        if let Some(x) = self.tokens.get_mut(token) {
            *x = *x + amount;
        } else {
            self.tokens.insert(token.clone(), amount);
        }
    }

    pub(crate) fn remove(&mut self, token: &AccountId, amount: u128) {
        if let Some(x) = self.tokens.get_mut(token) {
            assert!(*x >= amount, ERR13_NOT_ENOUGH_TOKENS_DEPOSITED);
            *x = *x - amount;
        } else {
            panic!(ERR13_NOT_ENOUGH_TOKENS_DEPOSITED);
        }
    }

    // asserts that the account has eough NEAR to cover storage and use of `amount` NEAR.
    #[inline]
    pub(crate) fn remove_near(&mut self, ynear: u128) {
        assert!(
            self.ynear >= ynear + (self.storage_used as u128) * env::storage_byte_cost(),
            ERR14_NOT_ENOUGH_NEAR_DEPOSITED,
        );
        self.ynear -= ynear;
    }

    pub fn storage_usage(&self) -> Balance {
        self.storage_used as Balance * env::storage_byte_cost()
    }

    #[inline]
    pub(crate) fn assert_storage(&self) {
        assert!(
            self.storage_used >= INIT_ACCOUNT_STORAGE
                && self.ynear >= (self.storage_used as u128) * env::storage_byte_cost(),
            ERR21_ACC_STORAGE_TOO_LOW
        )
    }

    // TODO: add unit tests
    pub(crate) fn update_storage(&mut self, tx_start_storage: StorageUsage) {
        let s = env::storage_usage();
        self.storage_used += s - tx_start_storage;
        self.assert_storage();
    }
}

#[cfg(test)]
mod tests {
    use super::AccountDeposit;

    fn new_account_deposit() -> AccountDeposit {
        AccountDeposit {
            ynear: 12,
            storage_used: 10,
            tokens: [("token1".to_string(), 100), ("token2".to_string(), 50)]
                .iter()
                .cloned()
                .collect(),
        }
    }

    #[test]
    fn add_works() {
        let mut deposit = new_account_deposit();

        AccountDeposit::add(&mut deposit, &"token1".to_string(), 10);
        assert_eq!(deposit.tokens.get(&"token1".to_string()), Some(&110));
    }

    #[test]
    fn add_new_works() {
        let mut deposit = new_account_deposit();

        AccountDeposit::add(&mut deposit, &"token33".to_string(), 100);
        assert_eq!(deposit.tokens.get(&"token33".to_string()), Some(&100));
    }

    #[test]
    fn remove_works() {
        let mut deposit = new_account_deposit();

        AccountDeposit::remove(&mut deposit, &"token2".to_string(), 10);
        assert_eq!(deposit.tokens.get(&"token2".to_string()), Some(&40));
    }

    #[test]
    #[should_panic(expected = r#"E13: Insufficient amount of tokens in deposit"#)]
    fn remove_deposit_low() {
        let mut deposit = new_account_deposit();

        AccountDeposit::remove(&mut deposit, &"token2".to_string(), 1000);
    }

    #[test]
    fn assert_storage_works() {
        let deposit = AccountDeposit {
            ynear: 990000000000000000000,
            storage_used: 10,
            tokens: [("token1".to_string(), 100)].iter().cloned().collect(),
        };

        AccountDeposit::assert_storage(&deposit);
    }

    #[test]
    #[should_panic(expected = r#"E21: Not enough NEAR to cover storage. Deposit more NEAR"#)]
    fn assert_storage_low() {
        let deposit = new_account_deposit();

        AccountDeposit::assert_storage(&deposit);
    }

    #[test]
    fn remove_near_works() {
        let amount: u128 = 990000000000000000000;
        let mut d = AccountDeposit {
            ynear: amount,
            storage_used: 10,
            tokens: [("token1".to_string(), 100)].iter().cloned().collect(),
        };
        d.remove_near(10);
        assert_eq!(d.ynear, amount - 10);
    }

    #[test]
    #[should_panic(expected = r#"E14: Insufficient amount of NEAR in deposit"#)]
    fn remove_near_insufficient() {
        let mut d = new_account_deposit();
        d.remove_near(10);
    }
}
