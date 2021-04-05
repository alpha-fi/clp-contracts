// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use internal::{assert_max_pay, assert_min_buy};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{assert_one_yocto, env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise};

mod constants;
mod deposit;
pub mod errors;
mod ft_token;
mod internal;
pub mod pool;
mod storage_management;
pub mod types;
pub mod util;

use crate::deposit::*;
use crate::errors::*;
use crate::pool::*;
use crate::types::*;
use crate::util::*;
use crate::constants::*;

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
    deposits: LookupMap<AccountId, AccountDeposit>,
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
            pools: UnorderedMap::new("p".into()),
            deposits: LookupMap::new("d".into()),
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
        self.set_deposit(&caller, &d);
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
    /// `min_ynear` of Near and `min_tokens` of tokens. Shares are note
    /// exchengable between different pools.
    pub fn withdraw_liquidity(
        &mut self,
        token: AccountId,
        shares: U128,
        min_ynear: U128,
        min_tokens: U128,
    ) {
        let shares_: u128 = shares.into();
        let min_ynear: u128 = min_ynear.into();
        let min_tokens: u128 = min_tokens.into();
        assert!(
            shares_ > 0 && min_ynear > 0 && min_tokens > 0,
            "E2: balance arguments must be >0"
        );

        let caller = env::predecessor_account_id();
        let mut p = self.get_pool(&token);
        let current_shares = p.shares.get(&caller).unwrap_or(0);
        assert!(
            current_shares >= shares_,
            format!(
                "E5: can't withdraw more shares then currently owned ({})",
                current_shares
            )
        );

        let total_shares2 = u256::from(p.total_shares);
        let shares2 = u256::from(shares_);
        let ynear = (shares2 * u256::from(p.ynear) / total_shares2).as_u128();
        let token_amount = (shares2 * u256::from(p.tokens) / total_shares2).as_u128();
        assert!(
            ynear >= min_ynear && token_amount >= min_tokens,
            format!(
                "E6: redeeming (ynear={}, tokens={}), which is smaller than the required minimum",
                ynear, token_amount
            )
        );

        env_log!(
            "Reedeming {:?} shares for {} NEAR and {} tokens",
            shares,
            ynear,
            token_amount,
        );
        p.shares.insert(&caller, &(current_shares - shares_));
        p.total_shares -= shares_;
        p.tokens -= token_amount;
        p.ynear -= ynear;

        // .then(Promise::new(caller).transfer(ynear));

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
        assert_one_yocto();
        let ynear: u128 = ynear_in.into();
        let min_tokens: u128 = min_tokens.into();
        assert!(ynear > 0 && min_tokens > 0, ERR02_POSITIVE_ARGS);

        let (mut p, tokens_out) = self._price_n2t_in(&token, ynear);
        assert_min_buy(tokens_out, min_tokens);
        self._swap_n2t(&mut p, ynear, &token, tokens_out);
        return tokens_out.into();
    }

    /// Swaps NEAR to `token` and transfers the tokens to the caller.
    /// Caller attaches maximum amount of NEAR he is willing to swap to receive `tokens_out`
    /// of `token` wants to swap to the transacion. Surplus of NEAR tokens will be returned.
    /// Transaction will panic if the caller doesn't attach enough NEAR tokens.
    /// Returns amount of yNEAR paid.
    #[payable]
    pub fn swap_near_to_token_exact_out(
        &mut self,
        max_ynear: U128,
        token: AccountId,
        tokens_out: U128,
    ) -> U128 {
        assert_one_yocto();
        let tokens_out: u128 = tokens_out.into();
        let max_ynear: u128 = max_ynear.into();
        assert!(tokens_out > 0 && max_ynear > 0, ERR02_POSITIVE_ARGS);

        let mut p = self.get_pool(&token);
        let near_to_pay = self.calc_in_amount(tokens_out, p.ynear, p.tokens);
        self._swap_n2t(&mut p, near_to_pay, &token, tokens_out);
        return near_to_pay.into();
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
        assert_one_yocto();
        let tokens_paid: u128 = tokens_paid.into();
        let min_ynear: u128 = min_ynear.into();
        assert!(tokens_paid > 0 && min_ynear > 0, ERR02_POSITIVE_ARGS);

        let mut p = self.get_pool(&token);
        let near_out = self.calc_out_amount(tokens_paid, p.tokens, p.ynear);
        assert_min_buy(near_out, min_ynear);
        self._swap_t2n(&mut p, &token, tokens_paid, near_out);
        return near_out.into();
    }

    /// Swaps `token` to NEAR and transfers NEAR to the caller.
    /// Caller defines the amount of NEAR he wants to receive under a condition of not spending
    /// more than `max_tokens` of `token`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of `token`
    /// for this contract.
    /// Returns amount tokens yNEAR paid.
    #[payable]
    pub fn swap_token_to_near_exact_out(
        &mut self,
        token: AccountId,
        max_tokens: U128,
        ynear_out: U128,
    ) -> U128 {
        assert_one_yocto();
        let max_tokens: u128 = max_tokens.into();
        let ynear_out: u128 = ynear_out.into();
        assert!(ynear_out > 0 && max_tokens > 0, ERR02_POSITIVE_ARGS);

        let mut p = self.get_pool(&token);
        let tokens_to_pay = self.calc_in_amount(ynear_out, p.tokens, p.ynear);
        assert_max_pay(tokens_to_pay, max_tokens);
        self._swap_t2n(&mut p, &token, tokens_to_pay, ynear_out);
        return tokens_to_pay.into();
    }

    /// Swaps two different tokens.
    /// Caller defines the amount of tokens he wants to swap under a condition of
    /// receving at least `min_to_tokens`.
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
        assert_one_yocto();
        let tokens_in: u128 = tokens_in.into();
        let min_tokens_out: u128 = min_tokens_out.into();
        assert!(min_tokens_out > 0 && tokens_in > 0, ERR02_POSITIVE_ARGS);

        let (p1, p2, near_swap, tokens_out) =
            self._price_swap_tokens_in(&token_in, &token_out, tokens_in);
        assert_min_buy(tokens_out, min_tokens_out);
        self._swap_tokens(
            p1, p2, &token_in, tokens_in, &token_out, tokens_out, near_swap,
        );
        return tokens_out.into();
    }

    /// Swaps two different tokens.
    /// Caller defines the amount of tokens he wants to receive under a condiiton
    /// of not spending more than `max_from_tokens`.
    /// Preceeding to this transaction, caller has to create a sufficient allowance of
    /// `from` token for this contract.
    /// Transaction will panic if a caller doesn't provide enough allowance.
    /// Returns amount tokens paid.
    #[payable]
    pub fn swap_tokens_exact_out(
        &mut self,
        token_in: AccountId,
        max_tokens_in: U128,
        token_out: AccountId,
        tokens_out: U128,
    ) -> U128 {
        assert_one_yocto();
        let (tokens_out, max_tokens_in) = (u128::from(tokens_out), u128::from(max_tokens_in));
        assert!(max_tokens_in > 0 && tokens_out > 0, ERR02_POSITIVE_ARGS);

        let (p1, p2, near_swapped, tokens_in_to_pay) =
            self._price_swap_tokens_out(&token_in, &token_out, tokens_out);
        env_log!("Buying price is {}", tokens_in_to_pay);
        assert_max_pay(tokens_in_to_pay, max_tokens_in);
        self._swap_tokens(
            p1,
            p2,
            &token_in,
            tokens_in_to_pay,
            &token_out,
            tokens_out,
            near_swapped,
        );
        return tokens_in_to_pay.into();
    }

    /// Calculates amount of tokens user will recieve when swapping `ynear_in` for `token`
    /// assets
    pub fn price_near_to_token_in(&self, token: AccountId, ynear_in: U128) -> U128 {
        self._price_n2t_in(&token, ynear_in.into()).1.into()
    }

    /// Calculates amount of NEAR user will need to swap if he wants to receive
    /// `tokens_out` of `token`
    pub fn price_near_to_token_out(&self, token: AccountId, tokens_out: U128) -> U128 {
        self._price_n2t_out(&token, tokens_out.into()).1.into()
    }

    /// Calculates amount of NEAR user will recieve when swapping `tokens_in` for NEAR.
    pub fn price_token_to_near_in(&self, token: AccountId, tokens_in: U128) -> U128 {
        let tokens_in: u128 = tokens_in.into();
        assert!(tokens_in > 0, "E2: balance arguments must be >0");
        let p = self.get_pool(&token);
        return self.calc_out_amount(tokens_in, p.tokens, p.ynear).into();
    }

    /// Calculates amount of tokens user will need to swap if he wants to receive
    /// `tokens_out` of `tokens`
    pub fn price_token_to_near_out(&self, token: AccountId, ynear_out: U128) -> U128 {
        let ynear_out: u128 = ynear_out.into();
        assert!(ynear_out > 0, "E2: balance arguments must be >0");
        let p = self.get_pool(&token);
        return self.calc_in_amount(ynear_out, p.tokens, p.ynear).into();
    }

    /// Calculates amount of tokens `to` user will receive when swapping `tokens_in` of `from`
    pub fn price_token_to_token_in(&self, from: AccountId, to: AccountId, tokens_in: U128) -> U128 {
        self._price_swap_tokens_in(&from, &to, tokens_in.into())
            .3
            .into()
    }

    /// Calculates amount of tokens `from` user will need to swap if he wants to receive
    /// `tokens_out` of tokens `to`
    pub fn price_token_to_token_out(
        &self,
        from: AccountId,
        to: AccountId,
        tokens_out: U128,
    ) -> U128 {
        self._price_swap_tokens_out(&from, &to, tokens_out.into())
            .3
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
    use super::*;
    use near_sdk::{testing_env, MockedBlockchain, VMContext};
    use std::convert::TryInto;

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

        pub fn set_deposit(&mut self, attached_deposit: Balance) {
            self.vm.attached_deposit = attached_deposit;
            testing_env!(self.vm.clone());
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
        _init(NEP21_STORAGE_DEPOSIT * 120)
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
        let (mut ctx, mut c) = init();
        let t = ctx.accounts.token1.clone();
        let a = ctx.accounts.predecessor.clone();

        // in unit tests we can't do cross contract calls, so we can't check token1 updates.
        check_and_create_pool(&mut c, &t);

        let ynear_deposit = 30 * NDENOM;
        let token_deposit = 10 * NDENOM;
        let ynear_deposit_with_storage = ynear_deposit + NEP21_STORAGE_DEPOSIT;
        ctx.set_deposit(ynear_deposit_with_storage);

        c.add_liquidity(t.clone(), ynear_deposit.into(), token_deposit.into(), U128(0));

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

        c.add_liquidity(t.clone(), ynear_deposit.into(), (token_deposit * 10).into(), U128(0));
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
        let ynear_deposit = 3 * NDENOM;
        let token_deposit = 1 * NDENOM + 1;
        let ynear_deposit_with_storage = ynear_deposit + NEP21_STORAGE_DEPOSIT;

        let (ctx, mut c) = _init(ynear_deposit_with_storage);
        let t = ctx.accounts.token1.clone();
        let a = ctx.accounts.predecessor.clone();

        let initial_ynear = 30 * NDENOM;
        let mut shares_map = LookupMap::new("123".as_bytes().to_vec());
        shares_map.insert(&a, &initial_ynear);
        let p = Pool {
            ynear: initial_ynear,
            tokens: 10 * NDENOM,
            total_shares: 30 * NDENOM,
            shares: shares_map,
        };
        c.pools.insert(&t, &p);

        c.add_liquidity(t.clone(),token_deposit.into(), ynear_deposit.into(), U128(0));

        let p_info = c.pool_info(&t).expect("Pool should exist");
        let expected_pool = PoolInfo {
            ynear: (ynear_deposit + p.ynear).into(),
            tokens: (token_deposit + p.tokens).into(),
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
    fn withdraw_happy_path() {
        let (ctx, mut c) = init_with_storage_deposit();
        let acc = ctx.accounts.predecessor.clone();
        let t = ctx.accounts.token1.clone();

        let shares_bal = 12 * NDENOM;
        let mut shares_map = LookupMap::new("123".as_bytes().to_vec());
        shares_map.insert(&acc, &shares_bal);
        let p = Pool {
            ynear: shares_bal,
            tokens: 3 * NDENOM,
            total_shares: shares_bal,
            shares: shares_map,
        };
        c.set_pool(&t, &p);

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
        let acc_shares = c.balance_of(t.clone(), acc);
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
        let assert_in = |buy, in_bal, out_bal, expected| {
            assert_eq!(c.calc_in_amount(buy, in_bal, out_bal), expected)
        };

        // #  test in prices  #
        // Note, the output is always +1, because we add 1 at the end.

        // ## test same supply ## - we expect x*1.003 + 1
        assert_in(1, 10, 10, 2);
        assert_in(1, G, G, 2);
        assert_in(2, G, G, 3);
        assert_in(100, G, G, 101);
        // now the 0.3% takes effect
        assert_in(1000, G, G, 1004);
        assert_in(10_000, G, G, 10_031);
        assert_in(20_000, NDENOM, NDENOM, 20_061);

        // ## test 2:1 ## - we expect 2x*1.003 + 1
        assert_in(1, 2 * G, G, 3);
        assert_in(10_000, 2 * G, G, 20_061);
        assert_in(20_000, 2 * NDENOM, NDENOM, 40_121);

        // ## test 1:2 ## - we expect 0.5x*1.003 + 1
        assert_in(1, G, 2 * G, 1);
        assert_in(10_000, G, 2 * G, 5000 + 15 + 1);
        assert_in(20_000, NDENOM, 2 * NDENOM, 10_000 + 31);

        assert_in(10, 12 * NDENOM, 2400, 50360285878556170603862u128);

        // #  test out prices  #
        let assert_out = |sell, in_bal, out_bal, expected| {
            assert_eq!(c.calc_out_amount(sell, in_bal, out_bal), expected)
        };

        // ## test same supply ## - we expect x*0.997
        assert_out(1, G, G, 0); // 0 because we cut the decimals (should be 0.997)
        assert_out(10, G, G, 9); // again rounding, should be 9.97
        assert_out(1_000, G, G, 996); // rounding again...
        assert_out(100_000, NDENOM, NDENOM, 99699);

        // ## test 2:1 ## - we expect 0.5x*0.997
        assert_out(1, 2 * G, G, 0); // 0 because we cut the decimals (should be 0.997)
        assert_out(10, 2 * G, G, 4); // again rounding
        assert_out(1_000, 2 * G, G, 996 / 2); // 996 because we add `in_net` to the denominator
        assert_out(100_000, 2 * NDENOM, NDENOM, 99699 / 2); // 49849

        // ## test 1:2  ## - we expect 2x*0.997
        assert_out(1, G, 2 * G, 1); // 0 because we cut the decimals (should be 1.997)
        assert_out(1_000, G, 2 * G, 1993); // rounding again...
        assert_out(100_000, NDENOM, 2 * NDENOM, 199399);

        // ## test 1:10  ## - we expect 10x*0.997 $net_calc
        assert_out(1_000, G, 10 * G, 9969); //
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
        };
        let p2 = Pool {
            // 2:1
            ynear: 2 * G,
            tokens: G,
            total_shares: 0,
            shares: LookupMap::new("2".as_bytes().to_vec()),
        };
        c.set_pool(&t1, &p1);
        c.set_pool(&t2, &p2);

        let amount: u128 = 1_000_000;
        let amount_net: u128 = 997_000;
        let mut v = c.price_near_to_token_in(t1.clone(), amount.into());
        let v_expected = amount_net * p1.tokens / (p1.ynear + amount_net);
        assert_eq!(to_num(v), v_expected);
        let mut v2 = c.price_near_to_token_out(t1.clone(), v.into());
        assert_eq!(to_num(v2), amount, "price_out(price_in) must be identity");

        // check reverse computation
        v = c.price_token_to_near_in(t1.clone(), (amount * p1_factor).into());
        assert_eq!(to_num(v), v_expected / p1_factor);
        v2 = c.price_token_to_near_out(t1.clone(), v.into());
        assert_close(v2, amount * p1_factor, 10);

        // we can also do a reverse computation by starting from expected output
        v2 = c.price_token_to_near_out(t1.clone(), v_expected.into());
        assert_close(v2, amount * p1_factor * p1_factor, 10);
        v = c.price_token_to_near_in(t1.clone(), v2.into()).into();
        assert_eq!(to_num(v), v_expected);

        /*
         * test  token1 -> token2 swap
         */
        v = c.price_token_to_token_in(t1.clone(), t2.clone(), amount.into());
        v2 = c.price_token_to_token_out(t1.clone(), t2.clone(), v.into());
        assert_close(v2, amount, 10);

        v2 = c.price_token_to_token_out(t2.clone(), t1.clone(), amount.into());
        assert_close(v2, v.into(), amount * 2 / 1_000);
    }

    fn to_num(a: U128) -> u128 {
        a.into()
    }

    fn assert_close(a1: U128, a2: u128, margin: u128) {
        let a1 = to_num(a1);
        let diff = if a1 > a2 { a1 - a2 } else { a2 - a1 };
        assert!(
            diff <= margin,
            format!(
                "Expect to be close (margin={}):\n  left: {}\n right: {}\n  diff: {}\n",
                margin, a1, a2, diff
            )
        )
    }
}
