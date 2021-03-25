use std::collections::HashMap;
use std::convert::TryInto;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, PromiseOrValue, StorageUsage};

//use crate::errors::*;
use crate::ft_token::*;
use crate::constants::*;
use crate::*;

/**********************
   DEPOSIT AND STORAGE
       MANAGEMENT
***********************/

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
        d.near += amount;
        self.deposits.insert(&sender, &d);
        env_log!("Deposit, {} yNEAR", amount);
    }

    pub fn withdraw_near_deposit(
        &mut self,
        amount: U128,
        recipient: Option<ValidAccountId>,
    ) -> Promise {
        let sender = env::predecessor_account_id();
        let recipient = if let Some(a) = recipient {
            AccountId::from(a)
        } else {
            sender.clone()
        };
        env_log!("Deposit withdraw, {} yNEAR", amount.0);
        let amount = u128::from(amount);
        let mut d = self.get_deposit(&sender);
        d.assert_near(amount);
        d.near -= amount;
        self.deposits.insert(&sender, &d);
        Promise::new(recipient).transfer(amount)
    }

    pub fn withdraw_token_deposit(
        &mut self,
        token: ValidAccountId,
        amount: U128,
        recipient: Option<ValidAccountId>,
        is_contract: bool,
        tx_call_msg: String,
    ) {
        let sender = env::predecessor_account_id();
        let recipient = if let Some(a) = recipient {
            AccountId::from(a)
        } else {
            sender.clone()
        };
        let token_acc = AccountId::from(token.clone());
        env_log!("Deposit withdraw, {} {}", amount.0, token_acc);
        let mut d = self.get_deposit(&sender);
        let amount = u128::from(amount);
        d.remove(&token_acc, amount);
        self.deposits.insert(&sender, &d);

        if is_contract {
            ext_fungible_token::ft_transfer(
                recipient.try_into().unwrap(),
                amount.into(),
                Some("NEARswap withdraw".to_string()),
                token.as_ref(),
                1, // required 1yNEAR for transfers
                GAS_FOR_FT_TRANSFER,
            );
        } else {
            ext_fungible_token::ft_transfer_call(
                recipient.try_into().unwrap(),
                amount.into(),
                Some("NEARswap withdraw".to_string()),
                tx_call_msg,
                token.as_ref(),
                1, // required 1yNEAR for transfers
                GAS_FOR_FT_TRANSFER,
            );
        }
    }

    #[inline]
    fn get_deposit(&self, from: &AccountId) -> AccountDeposit {
        self.deposits.get(from).expect(ERR20_ACC_NOT_REGISTERED)
    }
}

/// Account deposits information and storage cost.
#[cfg(not(test))]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct AccountDeposit {
    /// Native amount sent to the exchange.
    /// Used for storage now, but in future can be used for trading as well.
    /// MUST be always bigger than `storage_used * STORAGE_PRICE_PER_BYTE`.
    pub near: Balance,
    /// Amount of storage bytes used by the account,
    pub storage_used: StorageUsage,
    /// Deposited token balances.
    pub tokens: HashMap<AccountId, Balance>,
}

/// Account deposits information and storage cost.
#[cfg(test)]
#[derive(BorshSerialize, BorshDeserialize, Default, Clone)]
pub struct AccountDeposit {
    pub near: Balance,
    pub storage_used: StorageUsage,
    pub tokens: HashMap<AccountId, Balance>,
}

impl AccountDeposit {
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
            panic!(ERR14_NOT_ENOUGH_NEAR_DEPOSITED);
        }
    }

    pub fn storage_usage(&self) -> Balance {
        (MIN_ACCOUNT_DEPOSIT_LENGTH + self.tokens.len() as u128 * (MAX_ACCOUNT_LENGTH + 16))
            * env::storage_byte_cost()
    }

    #[inline]
    pub(crate) fn assert_storage(&self) {
        assert!(
            self.near >= (self.storage_used as u128) * STORAGE_PRICE_PER_BYTE,
            ERR21_ACC_STORAGE_TOO_LOW
        )
    }

    /// asserts that the account has anough NEAR to cover storage and use of `amout` NEAR.
    #[inline]
    pub(crate) fn assert_near(&self, amount: u128) {
        assert!(
            self.near >= amount + (self.storage_used as u128) * STORAGE_PRICE_PER_BYTE,
            ERR14_NOT_ENOUGH_NEAR_DEPOSITED,
        )
    }
}

// TODO:
// + finish storage tracking, example: https://github.com/robert-zaremba/vostok-dao/blob/master/src/lib.rs#L97
//   we don't do the storage refunds, instead we shold accumulate what storage has been used and keeping the following invariant all the time: account_deposit.amount >= account_deposit.storage  * STORAGE_PRICE_PER_BYTE
// +

#[cfg(test)]
mod tests {
    use super::AccountDeposit;
    
    fn new_account_deposit() -> AccountDeposit {
        AccountDeposit {
            near: 12,
            storage_used: 10,
            tokens:[("token1".to_string(), 100),
            ("token2".to_string(), 50)]
            .iter().cloned().collect()
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
    #[should_panic(
        expected = r#"E13: Insufficient amount of tokens in deposit"#
    )]
    fn remove_deposit_low() {
        let mut deposit = new_account_deposit();

        AccountDeposit::remove(&mut deposit, &"token2".to_string(), 1000);
    }

    #[test]
    fn assert_storage_works() {
        let deposit = AccountDeposit {
            near: 990000000000000000000,
            storage_used: 10,
            tokens:[("token1".to_string(), 100)]
            .iter().cloned().collect()
        };

        AccountDeposit::assert_storage(&deposit);
    }

    #[test]
    #[should_panic(
        expected = r#"E21: Not enough NEAR to cover storage. Deposit more NEAR"#
    )]
    fn assert_storage_low() {
        let deposit = new_account_deposit();

        AccountDeposit::assert_storage(&deposit);
    }

    #[test]
    fn assert_near_works() {
        let deposit = AccountDeposit {
            near: 990000000000000000000,
            storage_used: 10,
            tokens:[("token1".to_string(), 100)]
            .iter().cloned().collect()
        };

        AccountDeposit::assert_near(&deposit, 10);
    }

    #[test]
    #[should_panic(
        expected = r#"E14: Insufficient amount of NEAR in deposit"#
    )]
    fn assert_near_insufficient() {
        let deposit = new_account_deposit();

        AccountDeposit::assert_near(&deposit, 1);
    }
}
