// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use internal::assert_min_buy;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise, StorageUsage,
};

mod constants;
mod deposit;
pub mod errors;
mod ft_token;
mod internal;
pub mod pool;
mod storage_management;
pub mod twap;
pub mod types;
pub mod util;
mod view;

use crate::deposit::*;
use crate::errors::*;
pub use crate::pool::*;
use crate::types::*;
use crate::util::*;

// a way to optimize memory management
near_sdk::setup_alloc!();

/// NearSwap is the main contract for managing the swap pools and liquidity.
/// It implements the NEARswap functionality.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NearSwap {
    pub fee_dst: AccountId,
    pub owner: AccountId,
    // we are using unordered map because it allows to iterate over the pools
    pools: UnorderedMap<AccountId, Pool>,

    // user deposits
    deposits: LookupMap<AccountId, Deposit>,

    // Set of whitelisted tokens by "owner".
    whitelisted_tokens: UnorderedSet<AccountId>,
}

//-------------------------
// CONTRACT PUBLIC API
//-------------------------
#[near_bindgen]
impl NearSwap {
    #[init]
    pub fn new(owner: ValidAccountId) -> Self {
        let o = AccountId::from(owner);
        Self {
            fee_dst: o.clone(),
            owner: o,
            pools: UnorderedMap::new(b"p".to_vec()),
            deposits: LookupMap::new(b"d".to_vec()),
            whitelisted_tokens: UnorderedSet::new(b"w".to_vec()),
        }
    }

    /// Updates the fee destination destination account
    pub fn set_fee_dst(&mut self, fee_dst: ValidAccountId) {
        self.assert_owner();
        self.fee_dst = fee_dst.into();
    }

    /// Owner is an account (can be a multisig) who has management rights to update
    /// fee size.
    pub fn change_owner(&mut self, new_owner: ValidAccountId) {
        self.assert_owner();
        let o = AccountId::from(new_owner);
        env_log!("Changing owner from {} to {}", self.owner, o);
        self.owner = o;
    }

    /// Extend whitelisted tokens with new tokens. Only can be called by owner.
    #[payable]
    pub fn extend_whitelisted_tokens(&mut self, tokens: Vec<ValidAccountId>) {
        self.assert_owner();
        for token in tokens {
            self.whitelisted_tokens.insert(token.as_ref());
        }
    }

    /// Remove whitelisted token. Only can be called by owner.
    pub fn remove_whitelisted_token(&mut self, token: ValidAccountId) {
        self.assert_owner();
        self.whitelisted_tokens.remove(token.as_ref());
    }

    /**********************
     POOL MANAGEMENT
    *********************/

    /// Allows any user to creat a new near-token pool. Each pool is identified by the `token`
    /// account - which we call the Pool Token.
    /// If a pool for give token exists then "E1" assert exception is thrown.
    /// TODO: charge user for a storage created!
    #[payable]
    pub fn create_pool(&mut self, token: ValidAccountId) {
        let token = AccountId::from(token);
        assert!(
            self.pools
                .insert(&token, &Pool::new(token.as_bytes().to_vec()))
                .is_none(),
            "E1: pool already exists"
        );
    }

    /// Extracts public information of the `token` pool.
    pub fn pool_info(&self, token: &AccountId) -> Option<PoolInfo> {
        match self.pools.get(&token) {
            None => None,
            Some(p) => Some(p.pool_info()),
        }
    }

    /// Returns list of pools identified by token AccountId.
    pub fn list_pools(&self) -> Vec<AccountId> {
        return self.pools.keys().collect();
    }

    /**
    Transfer $NEAR and tokens from deposit to a pool.
    The supplied funds must preserve current ratio of the liquidity pool.
    Arguments:
     * `ynear` - amount of yNEAR liquidity to add to the `token` pool. If it will require
       more tokens than `max_tokens`, then the `ynear` will be adjusted (lowerred) to meet
       the `max_tokens` constraint.
     * `max_tokens` - max amount of tokens to add to the liquidity
     * `min_shares` - minimum amount of shares to be minted to make the transaction successful.
        If 0, then min_shares constraint won't be checked.
    Returns: amount of LP Shares minted for the user.
    Panics when:
     * not enough tokens or NEAR in deposit
     * not enough NEAR to cover storage fees
     * `min_shares > 0` and the operation will mint less shares than required.
    Notes:
     * `token` - we don't need to use ValidAccountId because token is already registered. */
    #[payable]
    pub fn add_liquidity(
        &mut self,
        token: AccountId,
        ynear: U128,
        max_tokens: U128,
        min_shares: U128,
    ) -> U128 {
        assert_one_yocto();
        let start_storage = env::storage_usage();
        let mut p = self.get_pool(&token);
        let caller = env::predecessor_account_id();
        let ynear: Balance = ynear.into();
        let max_tokens: Balance = max_tokens.into();
        let mut d = self.get_deposit(&caller);
        assert!(
            ynear > 0 && max_tokens > 0,
            "E2: added liquidity must be >0"
        );
        let (ynear, added_tokens, shares_minted) =
            p.add_liquidity(&caller, ynear, max_tokens, min_shares.into());
        d.remove(&token, added_tokens);
        d.remove_near(ynear);
        d.update_storage(start_storage);
        self.deposits.insert(&caller, &d.into());
        self.set_pool(&token, &p);

        env_log!(
            "Minting {} of shares for {} yNEAR and {} tokens",
            shares_minted,
            ynear,
            added_tokens
        );
        return shares_minted.into();
    }

    /// Redeems `shares` for liquidity stored in this pool with condition of getting at least
    /// `min_ynear` of Near and `min_tokens` of tokens. Shares are not
    /// exchagable between different pools.
    pub fn withdraw_liquidity(
        &mut self,
        token: AccountId,
        shares: U128,
        min_ynear: U128,
        min_tokens: U128,
    ) {
        let start_storage = env::storage_usage();
        let shares: u128 = shares.into();
        let min_ynear: u128 = min_ynear.into();
        let min_tokens: u128 = min_tokens.into();
        assert!(
            shares > 0 && min_ynear > 0 && min_tokens > 0,
            "E2: balance arguments must be > 0"
        );

        let caller = env::predecessor_account_id();
        let mut p = self.get_pool(&token);
        let current_shares = p.shares.get(&caller).unwrap_or(0);
        assert!(
            current_shares >= shares,
            "{}",
            format!(
                "E5: can't withdraw more shares then currently owned ({})",
                current_shares
            )
        );

        let mut d = self.get_deposit(&caller);
        let (ynear, token_amount) = p.withdraw_liquidity(&caller, min_ynear, min_tokens, shares);

        env_log!(
            "Reedeming {:?} shares for {} NEAR and {} tokens",
            shares,
            ynear,
            token_amount,
        );

        d.add(&token, token_amount);
        d.add_near(ynear);
        d.update_storage(start_storage);
        self.deposits.insert(&caller, &d.into());
        self.set_pool(&token, &p);
    }

    /**********************
     AMM functions
    **********************/

    /// Swaps NEAR to `token` and transfers the tokens to the caller.
    /// Caller attaches near tokens he wants to swap to the transacion under a condition of
    /// receving at least `min_tokens` of `token`.
    /// Returns amount of bought tokens.
    #[payable]
    pub fn swap_near_to_token_exact_in(
        &mut self,
        ynear_in: U128,
        token: AccountId,
        min_tokens: U128,
    ) -> U128 {
        let start_storage = env::storage_usage();
        assert_one_yocto();
        let ynear: u128 = ynear_in.into();
        let min_tokens: u128 = min_tokens.into();
        assert!(ynear > 0 && min_tokens > 0, "{}", ERR02_POSITIVE_ARGS);

        let (mut p, tokens_out) = self._price_n2t_in(&token, ynear);
        assert_min_buy(tokens_out, min_tokens);
        let tokens_swap_out = self._swap_n2t(&mut p, ynear, &token, tokens_out);
        self.unsafe_storage_check(start_storage);
        return tokens_swap_out.into();
    }

    /// Swaps `tokens_paid` of `token` to NEAR and transfers NEAR to the caller under acc
    /// condition of receving at least `min_ynear` yocto NEARs.
    /// Preceeding to this transaction, caller has to create sufficient allowance of `token`
    /// for this contract (at least `tokens_paid`).
    /// Returns amount of yNEAR bought.
    #[payable]
    pub fn swap_token_to_near_exact_in(
        &mut self,
        token: AccountId,
        tokens_paid: U128,
        min_ynear: U128,
    ) -> U128 {
        let start_storage = env::storage_usage();
        assert_one_yocto();
        let tokens_paid: u128 = tokens_paid.into();
        let min_ynear: u128 = min_ynear.into();
        assert!(tokens_paid > 0 && min_ynear > 0, "{}", ERR02_POSITIVE_ARGS);

        let mut p = self.get_pool(&token);
        let (near_out, _) = self.calc_out_with_fee(tokens_paid, p.tokens, p.ynear);
        assert_min_buy(near_out, min_ynear);
        let near_swap_out = self._swap_t2n(&mut p, &token, tokens_paid, near_out);
        self.unsafe_storage_check(start_storage);
        return near_swap_out.into();
    }

    /// Swaps two different tokens.
    /// Caller defines the amount of tokens he wants to swap under a condition of
    /// receving at least `min_tokens_out`.
    /// Preceeding to this transaction, caller has to create a sufficient allowance of
    /// `from` token for this contract.
    /// Transaction will panic if a caller doesn't provide enough allowance.
    /// Returns amount tokens bought.
    #[payable]
    pub fn swap_tokens_exact_in(
        &mut self,
        token_in: AccountId,
        tokens_in: U128,
        token_out: AccountId,
        min_tokens_out: U128,
    ) -> U128 {
        let start_storage = env::storage_usage();
        assert_one_yocto();
        let tokens_in: u128 = tokens_in.into();
        let min_tokens_out: u128 = min_tokens_out.into();
        assert!(min_tokens_out > 0 && tokens_in > 0, "{}", ERR02_POSITIVE_ARGS);

        let mut p1 = self.get_pool(&token_in);
        let mut p2 = self.get_pool(&token_out);
        let tokens_out = self._price_swap_tokens_in(&token_in, &token_out, tokens_in);
        assert_min_buy(tokens_out, min_tokens_out);
        let tokens_swap_out = self._swap_tokens(
            &mut p1, &mut p2, &token_in, tokens_in, &token_out, tokens_out,
        );
        self.unsafe_storage_check(start_storage);
        return tokens_swap_out.into();
    }

    /**
    Update storage using deposit update storage function
    start_storage: storage before performing any operation
    Note:
    - This function must be called after performing all the operations to get the
    correct storage
    - This is unsafe when we have other deposit instance around this function call,
    which can overwrite changes here.
    */
    fn unsafe_storage_check(&mut self, start_storage: StorageUsage) {
        let user = env::predecessor_account_id();
        let mut d = self.get_deposit(&user);
        d.update_storage(start_storage);
        self.deposits.insert(&user, &d.into());
    }

    /// Calculates amount of tokens user will recieve when swapping `ynear_in` for `token`
    /// assets
    pub fn price_near_to_token_in(&self, token: AccountId, ynear_in: U128) -> U128 {
        self._price_n2t_in(&token, ynear_in.into()).1.into()
    }

    /// Calculates amount of NEAR user will recieve when swapping `tokens_in` for NEAR.
    pub fn price_token_to_near_in(&self, token: AccountId, tokens_in: U128) -> U128 {
        let tokens_in: u128 = tokens_in.into();
        assert!(tokens_in > 0, "E2: balance arguments must be >0");
        let p = self.get_pool(&token);
        let (out, _) = self.calc_out_with_fee(tokens_in, p.tokens, p.ynear).into();
        return U128(out);
    }

    /// Calculates amount of tokens `to` user will receive when swapping `tokens_in` of `from`
    pub fn price_token_to_token_in(&self, from: AccountId, to: AccountId, tokens_in: U128) -> U128 {
        self._price_swap_tokens_in(&from, &to, tokens_in.into())
            .into()
    }

    /**********************
     Multi Token standard: NEP-MFT
    **********************/

    /// returns resource to more information about the token.
    #[allow(unused)]
    pub fn token_url(&self, token: AccountId) -> String {
        "https://github.com/robert-zaremba/near-clp".to_string()
    }

    /// granularity is the smallest amount of tokens (in the internal denomination) which
    /// may be minted, sent or burned at any time.
    #[allow(unused)]
    pub fn granularity(&self, token: AccountId) -> U128 {
        U128::from(1)
    }

    /// Returns the number of decimals the token uses - e.g. 8, means to divide the token
    /// amount by 100000000 to get its user representation.
    #[allow(unused)]
    pub fn decimals(&self, token: AccountId) -> u8 {
        24
    }

    /// Returns total supply of given subtoken. Implements the NEP-MFT standard.
    pub fn total_supply(&self, token: AccountId) -> U128 {
        match self.pools.get(&token) {
            None => 0.into(),
            Some(p) => p.total_shares.into(),
        }
    }

    /// Returns the `owner` shares balance of a pool identified by the `token`.
    pub fn balance_of(&self, token: AccountId, holder: AccountId) -> U128 {
        self.get_pool(&token)
            .shares
            .get(&holder)
            .unwrap_or(0)
            .into()
    }

    /// Transfer `amount` of LP Shares (Liquidity Provider Shares) of a pool identified
    /// by the `token` (must be a valid AccountID) from the predecessor
    /// to the `recipient` account. Implements the NEP-MFT interface.
    /// If recipient is a smart-contract, then `transfer_call` should be used instead.
    /// `recipient` MUST NOT be a smart-contract.
    /// `msg` is a message for recipient. It might be used to send additional call
    //      instructions.
    /// `memo`: arbitrary data with no specified format used to link the transaction with an
    ///     external data. If referencing a binary data, it should use base64 serialization.
    /// The function panics if the token doesn't refer to any registered pool or the predecessor
    /// doesn't have sufficient amount of shares.
    #[payable]
    pub fn transfer(
        &mut self,
        token: String,
        recipient: AccountId,
        amount: U128,
        msg: String,
        memo: String,
    ) -> bool {
        self._transfer(token, recipient, amount, msg, memo, false)
    }

    /// Transfer `amount` of LP Shares (Liquidity Provider Shares) of a pool identified
    /// by the `token` (must be a valid AccountID) from the predecessor
    /// to the `recipient` contract. Implements the NEP-MFT interface.
    /// `recipient` MUST be a smart contract address.
    /// The recipient contract MUST implement `MFTRecipient` interface.
    /// `msg` is a message sent to the recipient. It might be used to send additional call
    //      instructions.
    /// `memo`: arbitrary data with no specified format used to link the transaction with an
    ///     external event. If referencing a binary data, it should use base64 serialization.
    /// The function panics if the token doesn't refer to any registered pool or the predecessor
    /// doesn't have sufficient amount of shares.
    #[payable]
    pub fn transfer_call(
        &mut self,
        token: String,
        recipient: AccountId,
        amount: U128,
        msg: String,
        memo: String,
    ) -> bool {
        self._transfer(token, recipient, amount, msg, memo, true)
    }

    /**********************
     Debug
    **********************/

    // TODO: remove
    pub fn remove_pool(&mut self, token: AccountId) {
        self.assert_owner();
        if let Some(p) = self.pools.remove(&token) {
            env_log!(
                "killing {} pool and transferring {} to {}",
                token,
                p.ynear,
                &self.owner,
            );
            Promise::new(self.owner.to_string()).transfer(p.ynear);
        }
    }
}
//-------------------------
// END CONTRACT PUBLIC API
//-------------------------

#[cfg(test)]
mod tests {
    use crate::twap::Twap;

    use super::*;
    use near_sdk::{testing_env, MockedBlockchain, VMContext};
    use near_sdk_sim::to_yocto;
    use std::convert::{TryFrom, TryInto};
    use crate::constants::*;

    struct Accounts {
        current: AccountId,
        owner: AccountId,
        predecessor: AccountId,
        token1: AccountId,
        token2: AccountId,
        alice: AccountId,
    }

    struct Ctx {
        accounts: Accounts,
        vm: VMContext,
    }

    impl Ctx {
        fn create_accounts() -> Accounts {
            return Accounts {
                current: "clp".to_string(),
                owner: "clp_owner".to_string(),
                predecessor: "predecessor".to_string(),
                token1: "token1".to_string(),
                token2: "token2".to_string(),
                alice: "alice".to_string(),
            };
        }

        pub fn new(input: Vec<u8>, is_view: bool) -> Self {
            let accounts = Ctx::create_accounts();
            let vm = VMContext {
                current_account_id: accounts.current.clone(),
                signer_account_id: accounts.owner.clone(),
                signer_account_pk: vec![0, 1, 2],
                predecessor_account_id: accounts.predecessor.clone(),
                input,
                block_index: 0,
                block_timestamp: 0,
                account_balance: 0,
                account_locked_balance: 0,
                storage_usage: 0,
                attached_deposit: 0,
                prepaid_gas: MAX_GAS,
                random_seed: vec![0, 1, 2],
                is_view,
                output_data_receivers: vec![],
                epoch_height: 19,
            };
            return Self {
                accounts: accounts,
                vm: vm,
            };
        }
    }

    fn _init(attached_near: Balance) -> (Ctx, NearSwap) {
        let mut ctx = Ctx::new(vec![], false);
        ctx.vm.attached_deposit = attached_near;
        testing_env!(ctx.vm.clone());
        let contract = NearSwap::new("clp_owner".try_into().unwrap());
        return (ctx, contract);
    }
    fn init() -> (Ctx, NearSwap) {
        _init(0)
    }
    fn init_with_storage_deposit() -> (Ctx, NearSwap) {
        _init(NDENOM)
    }
    fn init_with_owner() -> (Ctx, NearSwap) {
        let mut ctx = Ctx::new(vec![], false);
        ctx.vm.attached_deposit = 0;
        testing_env!(ctx.vm.clone());
        let contract = NearSwap::new("predecessor".try_into().unwrap());
        return (ctx, contract);
    }

    fn to_va(a: AccountId) -> ValidAccountId {
        ValidAccountId::try_from(a).unwrap()
    }

    fn account_deposit() -> DepositV1 {
        return DepositV1 {
            ynear: NDENOM,
            storage_used: 84,
            tokens: [("eth".into(), 11)].iter().cloned().collect(),
        };
    }

    #[test]
    fn add_to_whitelist_works() {
        let (ctx, mut c) = init_with_owner();
        let a = ctx.accounts.predecessor.clone();
        // Add to nearswap whitelist
        c.extend_whitelisted_tokens(vec![to_va("token1".into()), to_va("token2".into())]);

        // Add to account deposit list
        let mut d = account_deposit();
        d.add_to_whitelist(&vec![to_va("token1".into()), to_va("token2".into())]);
        c.deposits.insert(&a, &d.into());

        // deposit should work
        c.deposit_token(&a, &"token1".into(), 10);
        let res = c.get_deposit(&a);
        assert_eq!(res.tokens.get(&"token1".to_string()), Some(&10))
    }

    #[test]
    #[should_panic(expected = r#"E23: Token is not whitelisted"#)]
    fn add_to_whitelist_failure_1() {
        let (ctx, mut c) = init_with_owner();
        let a = ctx.accounts.predecessor.clone();
        // Add to nearswap whitelist
        c.extend_whitelisted_tokens(vec![to_va("token1".into()), to_va("token2".into())]);
        let d = account_deposit();
        c.deposits.insert(&a, &d.into());

        // deposit should not work
        // because token not whitelisted in account deposit
        c.deposit_token(&a, &"token1".into(), 10);
    }

    #[test]
    #[should_panic(expected = r#"E23: Token is not whitelisted"#)]
    fn add_to_whitelist_failure_2() {
        let (ctx, mut c) = init_with_owner();
        let a = ctx.accounts.predecessor.clone();

        let d = account_deposit();
        c.deposits.insert(&a, &d.into());

        // deposit should not work
        // because token not whitelisted in global list
        c.deposit_token(&a.clone(), &"token1".into(), 10);
    }

    #[test]
    #[should_panic(expected = r#"E23: Token is not whitelisted"#)]
    fn remove_from_whitelist_works_1() {
        let (ctx, mut c) = init_with_owner();
        let a = ctx.accounts.predecessor.clone();
        c.extend_whitelisted_tokens(vec![to_va("token1".into()), to_va("token2".into())]);
        let mut d = account_deposit();
        d.add_to_whitelist(&vec![to_va("token1".into()), to_va("token2".into())]);
        c.deposits.insert(&a, &d.into());
        c.remove_whitelisted_token(to_va("token1".into()));

        c.deposit_token(&a.clone(), &"token1".into(), 10);
    }

    #[test]
    #[should_panic(expected = r#"E23: Token is not whitelisted"#)]
    fn remove_from_whitelist_works_2() {
        let (ctx, mut c) = init_with_owner();
        let a = ctx.accounts.predecessor.clone();
        c.extend_whitelisted_tokens(vec![to_va("token1".into()), to_va("token2".into())]);
        let mut d = account_deposit();
        d.add_to_whitelist(&vec![to_va("token1".into()), to_va("token2".into())]);

        d.remove_from_whitelist(&to_va("token1".into()));
        c.deposits.insert(&a, &d.into());

        c.deposit_token(&a.clone(), &"token1".into(), 10);
    }

    // TODO - fix this test.
    // #[test]
    // #[should_panic]
    // fn test_new_twice_fails() {
    //     let (ctx, _c) = init();
    //     NearSwap::new(ctx.accounts.current);
    // }

    #[test]
    fn change_owner() {
        let (mut ctx, mut c) = init();

        assert_eq!(&c.owner, &ctx.accounts.owner);

        ctx.vm.predecessor_account_id = ctx.accounts.owner;
        testing_env!(ctx.vm);

        c.change_owner("new_owner_near".try_into().unwrap());
        assert_eq!(c.owner, "new_owner_near");
    }

    #[test]
    #[should_panic(expected = "E22: Only owner can call this function")]
    fn change_owner_other_account() {
        let (_, mut c) = init();
        let owner2: ValidAccountId = "new_owner_near".try_into().unwrap();
        c.change_owner(owner2.clone());
    }

    #[test]
    #[should_panic(expected = "E1: pool already exists")]
    fn create_twice_same_pool_fails() {
        let (ctx, mut c) = init();
        c.create_pool("token1".try_into().unwrap());

        // let's check firstly the pool is there
        let pools = c.list_pools();
        let expected = [ctx.accounts.token1.clone()];
        assert_eq!(pools, expected);

        //
        c.create_pool("token1".try_into().unwrap());
    }

    fn check_and_create_pool(c: &mut NearSwap, token: &AccountId) {
        c.create_pool(token.to_string().try_into().unwrap());
        match c.pool_info(token) {
            None => panic!("Pool for {} token is expected", token),
            Some(p) => assert_eq!(
                p,
                PoolInfo {
                    ynear: 0.into(),
                    tokens: 0.into(),
                    total_shares: 0.into()
                }
            ),
        }
    }

    #[test]
    fn anyone_create_pool() {
        let (ctx, mut c) = init();
        check_and_create_pool(&mut c, &ctx.accounts.token1);
        check_and_create_pool(&mut c, &ctx.accounts.token2);

        let mut pools = c.list_pools();
        let mut expected = [ctx.accounts.token1, ctx.accounts.token2];
        pools.sort();
        expected.sort();
        assert_eq!(pools, expected);
    }

    #[test]
    fn add_liquidity_happy_path() {
        let ynear_deposit = 3 * NDENOM;
        let token_deposit = 1 * NDENOM;

        // attached_near is 1 yocto
        let (ctx, mut c) = _init(1);
        let t = ctx.accounts.token1.clone();
        let a = ctx.accounts.predecessor.clone();

        // in unit tests we can't do cross contract calls, so we can't check token1 updates.
        check_and_create_pool(&mut c, &t);

        let d = DepositV1 {
            ynear: 2 * ynear_deposit + NDENOM,
            storage_used: 10,
            tokens: [(t.clone(), token_deposit * 11)].iter().cloned().collect(),
        };
        c.deposits.insert(&a, &d.into());
        c.add_liquidity(
            t.clone(),
            ynear_deposit.into(),
            token_deposit.into(),
            U128(0),
        );

        let mut p = c.pool_info(&t).expect("Pool should exist");
        let mut expected_pool = PoolInfo {
            ynear: ynear_deposit.into(),
            tokens: token_deposit.into(),
            total_shares: ynear_deposit.into(),
        };
        assert_eq!(p, expected_pool, "pool_info should be correct");
        let a_shares = to_num(c.balance_of(t.clone(), a.clone()));
        assert_eq!(
            a_shares, ynear_deposit,
            "LP should have correct amount of shares"
        );
        assert_eq!(
            to_num(c.total_supply(t.clone())),
            ynear_deposit,
            "Total supply should be correct"
        );

        // total supply of an unknown token must be 0
        assert_eq!(
            to_num(c.total_supply("unknown-token".to_string())),
            0,
            "total supply of other token shouldn't change"
        );

        println!(">> adding liquidity - second time");

        c.add_liquidity(
            t.clone(),
            ynear_deposit.into(),
            (token_deposit * 10).into(),
            U128(0),
        );
        p = c.pool_info(&t).expect("Pool should exist");
        expected_pool = PoolInfo {
            ynear: (ynear_deposit * 2).into(),
            tokens: (token_deposit * 2 + 1).into(), // 1 is added as a minimum token transfer
            total_shares: (ynear_deposit * 2).into(),
        };
        assert_eq!(p, expected_pool, "pool_info should be correct");
        assert_eq!(
            to_num(c.balance_of(t.clone(), a.clone())),
            ynear_deposit * 2,
            "LP should have correct amount of shares"
        );
        assert_eq!(
            to_num(c.total_supply(t.clone())),
            ynear_deposit * 2,
            "Total supply should be correct"
        );
    }

    #[test]
    fn add_liquidity2_happy_path() {
        let ynear_deposit = 30 * NDENOM;
        let token_deposit = 10 * NDENOM;

        // attached_near is 1 yocto
        let (ctx, mut c) = _init(1);
        let t = ctx.accounts.token1.clone();
        let a = ctx.accounts.predecessor.clone();

        let d = DepositV1 {
            ynear: ynear_deposit + NDENOM,
            storage_used: 84,
            tokens: [(t.clone(), token_deposit * 11)].iter().cloned().collect(),
        };
        c.deposits.insert(&a, &d.into());

        let initial_ynear = 30 * NDENOM;
        let mut shares_map = LookupMap::new("123".as_bytes().to_vec());
        shares_map.insert(&a, &initial_ynear);
        let p = Pool {
            ynear: initial_ynear,
            tokens: 10 * NDENOM,
            total_shares: initial_ynear,
            shares: shares_map,
            twap: Twap::new(10),
        };
        c.pools.insert(&t, &p);

        c.add_liquidity(
            t.clone(),
            ynear_deposit.into(),
            (token_deposit * 10).into(),
            U128(0),
        );

        let p_info = c.pool_info(&t).expect("Pool should exist");
        let expected_pool = PoolInfo {
            ynear: (ynear_deposit + p.ynear).into(),
            tokens: (token_deposit + p.tokens + 1).into(), // 1 is added as a minimum token transfer
            total_shares: (ynear_deposit + p.ynear).into(),
        };
        assert_eq!(p_info, expected_pool, "pool_info should be correct");
        let a_shares = c.balance_of(t.clone(), a);
        assert_eq!(
            to_num(a_shares),
            ynear_deposit + p.ynear,
            "LP should have correct amount of shares"
        );
    }

    #[test]
    fn add_liquidity3_happy_path_adjust_ynear() {
        let ynear_deposit = 30 * NDENOM;
        let token_deposit = 10 * NDENOM;

        // attached_near is 1 yocto
        let (ctx, mut c) = _init(1);
        let t = ctx.accounts.token1.clone();
        let a = ctx.accounts.predecessor.clone();

        let d = DepositV1 {
            ynear: ynear_deposit + NDENOM,
            storage_used: 84,
            tokens: [(t.clone(), token_deposit * 11)].iter().cloned().collect(),
        };
        c.deposits.insert(&a, &d.into());

        let initial_ynear = 30 * NDENOM;
        let mut shares_map = LookupMap::new("123".as_bytes().to_vec());
        shares_map.insert(&a, &initial_ynear);
        // Pool Ratio: 3:1
        let p = Pool {
            ynear: initial_ynear,
            tokens: 10 * NDENOM,
            total_shares: initial_ynear,
            shares: shares_map,
            twap: Twap::new(10),
        };
        c.pools.insert(&t, &p);

        let max_tokens = token_deposit / 2;
        // max_tokens passed is less than required therefore ynear to be added
        // in pool will be adjusted
        c.add_liquidity(t.clone(), ynear_deposit.into(), max_tokens.into(), U128(0));

        let adjusted_near = expected_adjusted_near(max_tokens, p.ynear, p.tokens);
        assert!(
            adjusted_near < ynear_deposit,
            "Adjusted near is greater than near deposit"
        );
        let p_info = c.pool_info(&t).expect("Pool should exist");
        let expected_pool = PoolInfo {
            ynear: (adjusted_near + p.ynear).into(),
            tokens: (max_tokens + p.tokens).into(),
            total_shares: (adjusted_near + p.ynear).into(),
        };
        assert_eq!(p_info, expected_pool, "pool_info should be correct");
    }

    fn expected_adjusted_near(max_tokens: u128, ynear_pool: u128, tokens_pool: u128) -> u128 {
        let p_ynear_256 = u256::from(ynear_pool);
        return ((u256::from(max_tokens) * p_ynear_256) / u256::from(tokens_pool) + 1).as_u128();
    }

    #[test]
    fn add_liquidity_min_shares_path() {
        let ynear_deposit = 30 * NDENOM;
        let token_deposit = 10 * NDENOM;

        // attached_near is 1 yocto
        let (ctx, mut c) = _init(1);
        let t = ctx.accounts.token1.clone();
        let a = ctx.accounts.predecessor.clone();

        // in unit tests we can't do cross contract calls, so we can't check token1 updates.
        check_and_create_pool(&mut c, &t);

        let d = DepositV1 {
            ynear: 2 * ynear_deposit + NDENOM,
            storage_used: 10,
            tokens: [(t.clone(), token_deposit * 11)].iter().cloned().collect(),
        };
        c.deposits.insert(&a, &d.into());

        c.add_liquidity(
            t.clone(),
            ynear_deposit.into(),
            token_deposit.into(),
            U128(0),
        );

        let mut p = c.pool_info(&t).expect("Pool should exist");
        let mut expected_pool = PoolInfo {
            ynear: ynear_deposit.into(),
            tokens: token_deposit.into(),
            total_shares: ynear_deposit.into(),
        };
        assert_eq!(p, expected_pool, "pool_info should be correct");
        let a_shares = to_num(c.balance_of(t.clone(), a.clone()));
        assert_eq!(
            a_shares, ynear_deposit,
            "LP should have correct amount of shares"
        );
        assert_eq!(
            to_num(c.total_supply(t.clone())),
            ynear_deposit,
            "Total supply should be correct"
        );

        // total supply of an unknown token must be 0
        assert_eq!(
            to_num(c.total_supply("unknown-token".to_string())),
            0,
            "total supply of other token shouldn't change"
        );

        println!(">> adding liquidity - second time with minted shares");

        let min_shares = to_yocto("30");

        c.add_liquidity(
            t.clone(),
            ynear_deposit.into(),
            (token_deposit * 10).into(),
            min_shares.into(),
        );
        p = c.pool_info(&t).expect("Pool should exist");
        expected_pool = PoolInfo {
            ynear: (ynear_deposit * 2).into(),
            tokens: (token_deposit * 2 + 1).into(), // 1 is added as a minimum token transfer
            total_shares: (ynear_deposit * 2).into(),
        };
        assert_eq!(p, expected_pool, "pool_info should be correct");
        assert_eq!(
            to_num(c.balance_of(t.clone(), a.clone())),
            ynear_deposit * 2,
            "LP should have correct amount of shares"
        );
        assert_eq!(
            to_num(c.total_supply(t.clone())),
            ynear_deposit * 2,
            "Total supply should be correct"
        );
    }

    #[test]
    fn withdraw_happy_path() {
        let (ctx, mut c) = init_with_storage_deposit();
        let a = ctx.accounts.predecessor.clone();
        let t = ctx.accounts.token1.clone();

        let shares_bal = 12 * NDENOM;
        let mut shares_map = LookupMap::new("123".as_bytes().to_vec());
        shares_map.insert(&a, &shares_bal);
        let p = Pool {
            ynear: shares_bal,
            tokens: 3 * NDENOM,
            total_shares: shares_bal,
            shares: shares_map,
            twap: Twap::new(10),
        };
        c.set_pool(&t, &p);

        let d = DepositV1 {
            ynear: NDENOM,
            storage_used: 84,
            tokens: [(t.clone(), 11)].iter().cloned().collect(),
        };
        c.deposits.insert(&a, &d.into());

        let amount = shares_bal / 3;
        let min_v = U128::from(1);
        c.withdraw_liquidity(t.clone(), amount.into(), min_v, min_v);

        let pi = c.pool_info(&t).expect("Pool should exist");
        let expected_pool = PoolInfo {
            ynear: U128::from(shares_bal - amount),
            tokens: U128::from(2 * NDENOM),
            total_shares: U128::from(shares_bal - amount),
        };
        assert_eq!(pi, expected_pool, "pool_info should be correct");
        let acc_shares = c.balance_of(t.clone(), a);
        assert_eq!(
            to_num(acc_shares),
            shares_bal - amount,
            "LP should have correct amount of shares"
        );
        assert_eq!(
            to_num(c.total_supply(t)),
            shares_bal - amount,
            "LP should have correct amount of shares"
        );
    }

    fn prepare_for_withdraw() -> (AccountId, NearSwap) {
        let (ctx, mut c) = init_with_storage_deposit();
        let a = ctx.accounts.predecessor.clone();
        let t = ctx.accounts.token1.clone();

        let shares_bal = 12 * NDENOM;
        let mut shares_map = LookupMap::new("123".as_bytes().to_vec());
        shares_map.insert(&a, &shares_bal);
        let p = Pool {
            ynear: shares_bal,
            tokens: 3 * NDENOM,
            total_shares: shares_bal,
            shares: shares_map,
            twap: Twap::new(10),
        };
        c.set_pool(&t, &p);

        let d = DepositV1 {
            ynear: NDENOM,
            storage_used: 84,
            tokens: [(t.clone(), 11)].iter().cloned().collect(),
        };
        c.deposits.insert(&a, &d.into());
        return (t.clone(), c);
    }

    #[test]
    #[should_panic(expected = r#"E6: redeeming"#)]
    fn withdraw_happy_path_failure_1() {
        let (t, mut c) = prepare_for_withdraw();

        let shares_bal = 12 * NDENOM;
        let amount = shares_bal / 3;
        let min_near = U128::from(10 * NDENOM);
        let min_token = U128::from(1);
        c.withdraw_liquidity(t.clone(), amount.into(), min_near, min_token);
    }

    #[test]
    #[should_panic(expected = r#"E6: redeeming"#)]
    fn withdraw_happy_path_failure_2() {
        let (t, mut c) = prepare_for_withdraw();

        let shares_bal = 12 * NDENOM;
        let amount = shares_bal / 3;
        let min_near = U128::from(1);
        let min_token = U128::from(10 * NDENOM);
        c.withdraw_liquidity(t.clone(), amount.into(), min_near, min_token);
    }

    #[test]
    #[should_panic(expected = "E5: can't withdraw more shares then currently owned")]
    fn withdraw_happy_path_failure_3() {
        let (t, mut c) = prepare_for_withdraw();

        let shares = 24 * NDENOM;
        let min_near = U128::from(1);
        let min_token = U128::from(1);
        c.withdraw_liquidity(t.clone(), shares.into(), min_near, min_token);
    }

    #[test]
    fn shares_transfer() {
        let (ctx, mut c) = init_with_storage_deposit();
        let acc = ctx.accounts.predecessor.clone();
        let t = ctx.accounts.token1.clone();

        let shares_bal = 12 * NDENOM;
        let mut shares_map = LookupMap::new("123".as_bytes().to_vec());
        shares_map.insert(&acc, &shares_bal);
        let p = Pool {
            ynear: shares_bal,
            tokens: 22 * NDENOM,
            total_shares: shares_bal,
            shares: shares_map,
            twap: Twap::new(10),
        };
        c.set_pool(&t, &p);

        let amount = shares_bal / 3;
        let alice = ctx.accounts.alice.clone();
        c.transfer(
            t.clone(),
            alice.clone(),
            amount.into(),
            "msg".to_string(),
            "reference".to_string(),
        );

        // pool_info shouldn't be the same
        let p_info = c.pool_info(&t).expect("Pool should exist");
        assert_eq!(
            p.pool_info(),
            p_info,
            "pool_info shouldn't change after shares transfer"
        );

        // shares balance should be updated
        assert_eq!(
            to_num(c.balance_of(t.clone(), acc)),
            shares_bal - amount,
            "Predecessor shares should be updated"
        );
        assert_eq!(
            to_num(c.balance_of(t.clone(), alice)),
            amount,
            "Predecessor shares should be updated"
        );
    }

    #[test]
    fn calc_price() {
        let (_, c) = init();
        const G: u128 = 1_000_000_000;
        let assert_out = |in_amount, in_bal, out_bal, expected| {
            assert_eq!(c.calc_out_amount(in_amount, in_bal, out_bal), expected)
        };

        // #  test in prices  #

        // ## test same supply ## - we expect y = (x * Y * X) / (x + X)^2
        // `out` is rounded down, that's why in the first test we have 0 out.
        assert_out(1, 10, 10, 0);
        assert_out(1, G, G, 0);
        assert_out(2, G, G, 1);
        assert_out(100, G, G, 99);
        assert_out(1000, G, G, 999);
        assert_out(10_000, G, G, 9999);
        assert_out(20_000, NDENOM, NDENOM, 19999);

        // ## test 2:1 ## - we expect y = (2*x * Y * X) / (2*x + X)^2
        assert_out(1, 2 * G, G, 0);
        assert_out(10_000, 2 * G, G, 4999);
        assert_out(20_000, 2 * NDENOM, NDENOM, 9999);

        // ## test 1:2 ## - we expect (0.5x * Y * X) / (0.5x + X)^2
        assert_out(1, G, 2 * G, 1);
        assert_out(10_000, G, 2 * G, 19999);
        assert_out(20_000, NDENOM, 2 * NDENOM, 39999);

        assert_out(10, 12 * NDENOM, 2400, 0);
    }

    #[test]
    fn calc_price_with_fee() {
        let (_, c) = init();
        const G: u128 = 1_000_000_000;
        let assert_out = |in_amount, in_bal, out_bal| {
            assert_eq!(
                c.calc_out_with_fee(in_amount, in_bal, out_bal).0,
                expected_calc_price_fee(in_amount, in_bal, out_bal)
            )
        };

        // #  test in prices  #

        // ## test same supply ## - we expect y = (x * Y * X) / (x + X)^2
        // where x will be in_amount*(0.997) - (after deducting 3% fee)
        // `out` is rounded down, that's why in the first test we have 0 out.
        assert_out(1, 10, 10);
        assert_out(1, G, G);
        assert_out(2, G, G);
        assert_out(100, G, G);
        assert_out(1000, G, G);
        assert_out(10_000, G, G);
        assert_out(20_000, NDENOM, NDENOM);

        // ## test 2:1 ## - we expect y = (2*x * Y * X) / (2*x + X)^2
        assert_out(1, 2 * G, G);
        assert_out(10_000, 2 * G, G);
        assert_out(20_000, 2 * NDENOM, NDENOM);

        // ## test 1:2 ## - we expect (0.5x * Y * X) / (0.5x + X)^2
        assert_out(1, G, 2 * G);
        assert_out(10_000, G, 2 * G);
        assert_out(20_000, NDENOM, 2 * NDENOM);

        assert_out(10, 12 * NDENOM, 2400);
    }

    #[test]
    fn compare_fee_with_non_fee() {
        // Output tokens got from without fee will always be greater than with fee
        // It could be equal if values are smaller
        let (_, c) = init();
        const G: u128 = 1_000_000_000;
        let x = c.calc_out_amount(1_000_000, G, G);
        let (y, _) = c.calc_out_with_fee(1_000_000, G, G);
        assert!(x > y, "Tokens output incorrect");
    }

    #[allow(non_snake_case)]
    fn expected_calc_price_fee(amount: u128, in_bal: u128, out_bal: u128) -> u128 {
        let x = u256::from(amount - (amount * 3) / 1000);
        let X = u256::from(in_bal);
        let numerator = x * u256::from(out_bal) * X;
        let mut denominator = x + X;
        denominator *= denominator;
        return (numerator / denominator).as_u128();
    }

    #[test]
    fn price_swap() {
        let (_, mut c) = init_with_storage_deposit();
        const G: u128 = 1_000_000_000_000_000_000;
        let t1: AccountId = "token1".to_string();
        let t2: AccountId = "token2".to_string();
        let p1_factor = 4;
        let p1 = Pool {
            // 1:4
            ynear: G,
            tokens: p1_factor * G,
            total_shares: 0,
            shares: LookupMap::new("1".as_bytes().to_vec()),
            twap: Twap::new(10),
        };
        let p2 = Pool {
            // 2:1
            ynear: 2 * G,
            tokens: G,
            total_shares: 0,
            shares: LookupMap::new("2".as_bytes().to_vec()),
            twap: Twap::new(10),
        };
        c.set_pool(&t1, &p1);
        c.set_pool(&t2, &p2);

        let amount: u128 = 1_000_000;
        let mut v = c.price_near_to_token_in(t1.clone(), amount.into());

        let mut v_expected = expected_calc_price_fee(amount, p1.ynear, p1.tokens);
        assert_eq!(to_num(v), v_expected);

        v_expected = expected_calc_price_fee(amount, p1.tokens, p1.ynear);
        v = c.price_token_to_near_in(t1.clone(), amount.into());
        assert_eq!(to_num(v), v_expected);

        /*
         * test  token1 -> token2 swap
         * Pool1 swaps token1 to near that return amount_near = amount_tokens / 4
         * because pool ratio is 1:4
         * Pool2 swaps near to token2 that gives amount_tokens = amount_near / 2
         * because pool ratio is 2:1
         * Therefore final tokens received is approximately amount / 8
         */
        v = c.price_token_to_token_in(t1.clone(), t2.clone(), amount.into());
        assert_close(v, amount / 8, 1000);
    }

    fn to_num(a: U128) -> u128 {
        a.into()
    }

    fn assert_close(a1: U128, a2: u128, margin: u128) {
        let a1 = to_num(a1);
        let diff = if a1 > a2 { a1 - a2 } else { a2 - a1 };
        assert!(
            diff <= margin,
            "{}",
            format!(
                "Expect to be close (margin={}):\n  left: {}\n right: {}\n  diff: {}\n",
                margin, a1, a2, diff
            )
        )
    }
}
