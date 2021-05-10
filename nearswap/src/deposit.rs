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

        self.deposit_token(&sender_id, &token, amount.into());
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
    Add tokens to account deposit whitelist
    */
    pub fn add_to_account_whitelist(
        &mut self, token_ids: &Vec<ValidAccountId>) {
        let sender_id = env::predecessor_account_id();
        let mut d = self.get_deposit(&sender_id);
        d.add_to_whitelist(token_ids);
        self.deposits.insert(&sender_id, &d);
    }

    /// Record deposit of some number of tokens to this contract.
    /// Fails if account is not registered or if token isn't whitelisted.
    pub(crate) fn deposit_token(
        &mut self,
        sender_id: &AccountId,
        token_id: &AccountId,
        amount: Balance,
    ) {
        let mut d = self.get_deposit(sender_id);
        assert!(
            self.whitelisted_tokens.contains(token_id)
                && d.tokens.contains_key(token_id),
            "{}",
            ERR23_TOKEN_NOT_WHITELISTED
        );
        d.add(token_id, amount);
        self.deposits.insert(&sender_id, &d);
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
    /// add given token to whitelist and set balance to 0.
    /// Fails if not enough amount to cover new storage usage.
    pub(crate) fn add_to_whitelist(&mut self, token_ids: &Vec<ValidAccountId>) {
        for token_id in token_ids {
            let t = token_id.as_ref();
            if !self.tokens.contains_key(t) {
                self.tokens.insert(t.clone(), 0);
            }
        }
        self.assert_storage();
    }

    /// Remove `token_id` from this account whitelist and remove balance.
    /// Panics if the `token_id` balance is not 0.
    pub(crate) fn remove_from_whitelist(&mut self, token_id: &AccountId) {
        let amount = self.tokens.remove(token_id).unwrap_or_default();
        assert_eq!(amount, 0, "{}", ERR24_NON_ZERO_TOKEN_BALANCE);
    }
    /**
    deposit `token`s. If this is a first depoisit, a new record is created and the minimum
    required storage is increased. 
    Fails if account is not registered or if token isn't whitelisted.
    */
    pub(crate) fn add(&mut self, token: &AccountId, amount: u128) {
        if let Some(x) = self.tokens.get_mut(token) {
            *x = *x + amount;
        } else {
            self.tokens.insert(token.clone(), amount);
        }
    }

    // add near to current deposit
    pub(crate) fn add_near(&mut self, amount: u128) {
        self.ynear += amount;
    }

    pub(crate) fn remove(&mut self, token: &AccountId, amount: u128) {
        if let Some(x) = self.tokens.get_mut(token) {
            assert!(*x >= amount, ERR13_NOT_ENOUGH_TOKENS_DEPOSITED);
            *x = *x - amount;
        } else {
            panic!(ERR13_NOT_ENOUGH_TOKENS_DEPOSITED);
        }
    }

    // asserts that the account has enough NEAR to cover storage and use of `amount` NEAR.
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

    /// Updates the account storage usage. This has to be called after all non AcountDeposit
    /// changs are saved. Otherwise we will not take into account storage acquired in that
    /// changes.
    /// Panics if there is not enought $NEAR to cover storage usage.
    pub(crate) fn update_storage(&mut self, tx_start_storage: StorageUsage) {
        self.storage_used += env::storage_usage() - tx_start_storage;
        self.assert_storage();
    }
}

#[cfg(test)]
mod tests {
    use super::AccountDeposit;
    use near_sdk::env;
    use near_sdk::test_utils::{VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain};

    fn init_blockchain() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
    }

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
        init_blockchain();
        let deposit = AccountDeposit {
            ynear: 9900000000000000000000,
            storage_used: 100,
            tokens: [("token1".to_string(), 100)].iter().cloned().collect(),
        };

        AccountDeposit::assert_storage(&deposit);
    }

    #[test]
    #[should_panic(expected = r#"E21: Not enough NEAR to cover storage. Deposit more NEAR"#)]
    fn assert_storage_low() {
        init_blockchain();
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

    #[test]
    fn update_storage_works() {
        init_blockchain();
        let mut d = new_account_deposit();
        d.ynear = 1000_0000_0000_0000_0000_0000_0000;
        d.storage_used = 84;

        let initial = env::storage_usage();
        d.update_storage(2);

        let expected = initial - 2 + 84;
        assert!(d.storage_used == expected, "Storage Mismatch");
    }
}
