// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise};

mod deposit;
pub mod errors;
mod ft_token;
pub mod pool;
pub mod types;
pub mod util;
mod storage_management;
mod constants;

use crate::deposit::*;
use crate::errors::*;
use crate::pool::*;
use crate::types::*;
use crate::util::*;

mod internal;

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
    /// account - which we call the Pool Reserve Token.
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

    /// Returns list of pools identified as their reserve token AccountId.
    pub fn list_pools(&self) -> Vec<AccountId> {
        return self.pools.keys().collect();
    }

    /// Increases Near and the Reserve token liquidity.
    /// The supplied funds must preserve current ratio of the liquidity pool.
    /// Returns amount of LP Shares is minted for the user.
    #[payable]
    pub fn add_liquidity(&mut self, token: AccountId, max_tokens: U128, min_shares: U128) -> U128 {
        let mut p = self.must_get_pool(&token);
        let caller = env::predecessor_account_id();
        let shares_minted;
        let ynear_amount = env::attached_deposit() - NEP21_STORAGE_DEPOSIT;
        let added_reserve;
        let max_tokens: Balance = max_tokens.into();
        assert!(
            ynear_amount > 0 && max_tokens > 0,
            "E2: balance arguments must be >0"
        );

        // the very first deposit -- we define the constant ratio
        if p.total_shares == 0 {
            p.ynear = ynear_amount;
            shares_minted = p.ynear;
            p.total_shares = shares_minted;
            added_reserve = max_tokens;
            p.reserve = added_reserve;
            p.shares.insert(&caller, &shares_minted);
        } else {
            let ynear_256 = u256::from(ynear_amount);
            let p_ynear_256 = u256::from(p.ynear);
            added_reserve = (ynear_256 * u256::from(p.reserve) / p_ynear_256 + 1).as_u128();
            shares_minted = (ynear_256 * u256::from(p.total_shares) / p_ynear_256).as_u128();
            env_log!(
                "shares_to_mint={}, min_shares={}; attached_ynear={}, added_reserve={}, max_added_reserve={}",
                shares_minted,
                min_shares.0,
                ynear_amount,
                added_reserve, max_tokens
            );
            assert!(
                max_tokens >= added_reserve,
                "E3: needs to transfer {} of tokens and it's bigger then specified  maximum={}",
                added_reserve,
                max_tokens
            );
            assert!(
                u128::from(min_shares) <= shares_minted,
                "E4: amount minted shares ({}) is smaller then the required minimum",
                shares_minted
            );
            p.shares.insert(
                &caller,
                &(p.shares.get(&caller).unwrap_or(0) + shares_minted),
            );
            p.reserve += added_reserve;
            p.ynear += ynear_amount;
            p.total_shares += shares_minted;
        }

        env_log!(
            "Minting {} of shares for {} yNEAR and {} reserve tokens",
            shares_minted,
            ynear_amount,
            added_reserve
        );
        self.set_pool(&token, &p);

        // TODO: do proper rollback
        // Prepare a callback for liquidity transfer rollback which we will attach later on.
        let callback_args = format!(r#"{{ "token":"{}" }}"#, token).into();
        let callback = Promise::new(env::current_account_id()).function_call(
            "add_liquidity_transfer_callback".into(),
            callback_args,
            0,
            5 * TGAS,
        );
        self.schedule_nep21_tx(&token, caller, env::current_account_id(), added_reserve)
            .then(callback); //after that, the callback will check success/failure

        // TODO:
        // Handling exception is work-in-progress in NEAR runtime
        // 1. rollback `p` on changes or move the pool update to a promise
        // 2. consider adding a lock to prevent other contracts calling and manipulate the prise before the token transfer will get finalized.

        return shares_minted.into();
    }

    /// Redeems `shares` for liquidity stored in this pool with condition of getting at least
    /// `min_ynear` of Near and `min_tokens` of reserve tokens (`token`). Shares are note
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
        let mut p = self.must_get_pool(&token);
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
        let ynear_amount = (shares2 * u256::from(p.ynear) / total_shares2).as_u128();
        let token_amount = (shares2 * u256::from(p.reserve) / total_shares2).as_u128();
        assert!(
            ynear_amount >= min_ynear && token_amount >= min_tokens,
            format!(
                "E6: redeeming (ynear={}, tokens={}), which is smaller than the required minimum",
                ynear_amount, token_amount
            )
        );

        env_log!(
            "Reedeming {:?} shares for {} NEAR and {} reserve tokens",
            shares,
            ynear_amount,
            token_amount,
        );
        p.shares.insert(&caller, &(current_shares - shares_));
        p.total_shares -= shares_;
        p.reserve -= token_amount;
        p.ynear -= ynear_amount;

        // let prepaid_gas = env::prepaid_gas();
        self.schedule_nep21_tx(
            &token,
            env::current_account_id(),
            caller.clone(),
            token_amount,
        )
        .then(Promise::new(caller).transfer(ynear_amount));

        //TODO COMPLEX-CALLBACKS
        self.set_pool(&token, &p);
    }

    /**********************
     CLP market functions
    **********************/

    /// Swaps NEAR to `token` and transfers the reserve tokens to the caller.
    /// Caller attaches near tokens he wants to swap to the transacion under a condition of
    /// receving at least `min_tokens` of `token`.
    #[payable]
    pub fn swap_near_to_token_exact_in(&mut self, token: AccountId, min_tokens: U128) {
        self._swap_n2t_exact_in(
            &token,
            env::attached_deposit(),
            min_tokens.into(),
            env::predecessor_account_id(),
        );
    }

    /// Same as `swap_near_to_token_exact_in`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    #[payable]
    pub fn swap_near_to_token_exact_in_xfr(
        &mut self,
        token: AccountId,
        min_tokens: U128,
        recipient: AccountId,
    ) {
        self._swap_n2t_exact_in(
            &token,
            env::attached_deposit(),
            min_tokens.into(),
            recipient,
        );
    }

    /// Swaps NEAR to `token` and transfers the reserve tokens to the caller.
    /// Caller attaches maximum amount of NEAR he is willing to swap to receive `tokens_out`
    /// of `token` wants to swap to the transacion. Surplus of NEAR tokens will be returned.
    /// Transaction will panic if the caller doesn't attach enough NEAR tokens.
    #[payable]
    pub fn swap_near_to_token_exact_out(&mut self, token: AccountId, tokens_out: U128) {
        let b = env::predecessor_account_id();
        self._swap_n2t_exact_out(
            &token,
            tokens_out.into(),
            env::attached_deposit(),
            b.clone(),
            b,
        );
    }

    /// Same as `swap_near_to_token_exact_out`, but user additionly specifies the `recipient`
    /// who will receive the reserve tokens after the swap.
    #[payable]
    pub fn swap_near_to_token_exact_out_xfr(
        &mut self,
        token: AccountId,
        tokens_out: U128,
        recipient: AccountId,
    ) {
        self._swap_n2t_exact_out(
            &token,
            tokens_out.into(),
            env::attached_deposit(),
            env::predecessor_account_id(),
            recipient,
        );
    }

    /// Swaps `tokens_paid` of `token` to NEAR and transfers NEAR to the caller under acc
    /// condition of receving at least `min_ynear` yocto NEARs.
    /// Preceeding to this transaction, caller has to create sufficient allowance of `token`
    /// for this contract (at least `tokens_paid`).
    /// TODO: Transaction will panic if a caller doesn't provide enough allowance.
    #[payable]
    pub fn swap_token_to_near_exact_in(
        &mut self,
        token: AccountId,
        tokens_paid: U128,
        min_ynear: U128,
    ) {
        let b = env::predecessor_account_id();
        self._swap_t2n_exact_in(&token, tokens_paid.into(), min_ynear.into(), b.clone(), b);
    }

    /// Same as `swap_token_to_near_exact_in`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    #[payable]
    pub fn swap_token_to_near_exact_in_xfr(
        &mut self,
        token: AccountId,
        tokens_paid: U128,
        min_ynear: U128,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_t2n_exact_in(&token, tokens_paid.into(), min_ynear.into(), b, recipient);
    }

    /// Swaps `token` to NEAR and transfers NEAR to the caller.
    /// Caller defines the amount of NEAR he wants to receive under a condition of not spending
    /// more than `max_tokens` of `token`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of `token`
    /// for this contract.
    /// TODO: Transaction will panic if a caller doesn't provide enough allowance.
    #[payable]
    pub fn swap_token_to_near_exact_out(
        &mut self,
        token: AccountId,
        ynear_out: U128,
        max_tokens: U128,
    ) {
        let b = env::predecessor_account_id();
        self._swap_t2n_exact_out(&token, ynear_out.into(), max_tokens.into(), b.clone(), b);
    }

    /// Same as `swap_token_to_near_exact_out`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    #[payable]
    pub fn swap_token_to_near_exact_out_xfr(
        &mut self,
        token: AccountId,
        ynear_out: U128,
        max_tokens: U128,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_t2n_exact_out(&token, ynear_out.into(), max_tokens.into(), b, recipient);
    }

    /// Swaps two different tokens.
    /// Caller defines the amount of tokens he wants to swap under a condition of
    /// receving at least `min_to_tokens`.
    /// Preceeding to this transaction, caller has to create a sufficient allowance of
    /// `from` token for this contract.
    /// Transaction will panic if a caller doesn't provide enough allowance.
    #[payable]
    pub fn swap_tokens_exact_in(
        &mut self,
        from: AccountId,
        to: AccountId,
        tokens_in: U128,
        min_tokens_out: U128,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_in(
            &from,
            &to,
            tokens_in.into(),
            min_tokens_out.into(),
            b.clone(),
            b,
        );
    }

    /// Same as `swap_tokens_exact_in`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    #[payable]
    pub fn swap_tokens_exact_in_xfr(
        &mut self,
        from: AccountId,
        to: AccountId,
        tokens_in: U128,
        min_tokens_out: U128,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_in(
            &from,
            &to,
            tokens_in.into(),
            min_tokens_out.into(),
            b,
            recipient,
        );
    }

    /// Swaps two different tokens.
    /// Caller defines the amount of tokens he wants to receive under a condiiton
    /// of not spending more than `max_from_tokens`.
    /// Preceeding to this transaction, caller has to create a sufficient allowance of
    /// `from` token for this contract.
    /// Transaction will panic if a caller doesn't provide enough allowance.
    #[payable]
    pub fn swap_tokens_exact_out(
        &mut self,
        from: AccountId,
        to: AccountId,
        tokens_out: U128,
        max_tokens_in: U128,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_out(
            &from,
            &to,
            tokens_out.into(),
            max_tokens_in.into(),
            b.clone(),
            b,
        );
    }

    /// Same as `swap_tokens_exact_out`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    #[payable]
    pub fn swap_tokens_exact_out_xfr(
        &mut self,
        from: AccountId,
        to: AccountId,
        tokens_out: U128,
        max_tokens_in: U128,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_out(
            &from,
            &to,
            tokens_out.into(),
            max_tokens_in.into(),
            b,
            recipient,
        );
    }

    /// Calculates amount of tokens user will recieve when swapping `ynear_in` for `token`
    /// assets
    pub fn price_near_to_token_in(&self, token: AccountId, ynear_in: U128) -> U128 {
        self._price_near_to_token_in(&token, ynear_in.into())
            .1
            .into()
    }

    /// Calculates amount of NEAR user will need to swap if he wants to receive
    /// `tokens_out` of `token`
    pub fn price_near_to_token_out(&self, token: AccountId, tokens_out: U128) -> U128 {
        self._price_near_to_token_out(&token, tokens_out.into())
            .1
            .into()
    }

    /// Calculates amount of NEAR user will recieve when swapping `tokens_in` for NEAR.
    pub fn price_token_to_near_in(&self, token: AccountId, tokens_in: U128) -> U128 {
        let tokens_in: u128 = tokens_in.into();
        assert!(tokens_in > 0, "E2: balance arguments must be >0");
        let p = self.must_get_pool(&token);
        return self.calc_out_amount(tokens_in, p.reserve, p.ynear).into();
    }

    /// Calculates amount of tokens user will need to swap if he wants to receive
    /// `tokens_out` of `tokens`
    pub fn price_token_to_near_out(&self, token: AccountId, ynear_out: U128) -> U128 {
        let ynear_out: u128 = ynear_out.into();
        assert!(ynear_out > 0, "E2: balance arguments must be >0");
        let p = self.must_get_pool(&token);
        return self.calc_in_amount(ynear_out, p.reserve, p.ynear).into();
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

    pub fn add_liquidity_transfer_callback(&mut self, token: AccountId) {
        println!("enter add_liquidity_transfer_callback");
        // TODO: handle refund from nep21
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Can be called only as a callback"
        );

        // TODO: simulation doesn't allow using a promise inside callbacks.
        // For now we just log result
        if !is_promise_success() {
            env_log!(
                "add_liquidity_transfer_callback: token {} transfer FAILED!",
                token
            );
            panic!("callback");
            //TODO ROLLBACK add_liquidity
        }
        println!("PromiseResult  transfer succeeded");

        // If the stake action failed and the current locked amount is positive, then the contract has to unstake.
        /*if !stake_action_succeeded && env::account_locked_balance() > 0 {
            Promise::new(env::current_account_id()).stake(0, self.stake_public_key.clone());
        }
         */
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
        self.must_get_pool(&token)
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
                    reserve: 0.into(),
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

        c.add_liquidity(t.clone(), token_deposit.into(), ynear_deposit.into());

        let mut p = c.pool_info(&t).expect("Pool should exist");
        let mut expected_pool = PoolInfo {
            ynear: ynear_deposit.into(),
            reserve: token_deposit.into(),
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

        c.add_liquidity(t.clone(), (token_deposit * 10).into(), ynear_deposit.into());
        p = c.pool_info(&t).expect("Pool should exist");
        expected_pool = PoolInfo {
            ynear: (ynear_deposit * 2).into(),
            reserve: (token_deposit * 2 + 1).into(), // 1 is added as a minimum token transfer
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
            reserve: 10 * NDENOM,
            total_shares: 30 * NDENOM,
            shares: shares_map,
        };
        c.pools.insert(&t, &p);

        c.add_liquidity(t.clone(), token_deposit.into(), ynear_deposit.into());

        let p_info = c.pool_info(&t).expect("Pool should exist");
        let expected_pool = PoolInfo {
            ynear: (ynear_deposit + p.ynear).into(),
            reserve: (token_deposit + p.reserve).into(),
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
            reserve: 3 * NDENOM,
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
            reserve: U128::from(2 * NDENOM),
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
            reserve: 22 * NDENOM,
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
            reserve: p1_factor * G,
            total_shares: 0,
            shares: LookupMap::new("1".as_bytes().to_vec()),
        };
        let p2 = Pool {
            // 2:1
            ynear: 2 * G,
            reserve: G,
            total_shares: 0,
            shares: LookupMap::new("2".as_bytes().to_vec()),
        };
        c.set_pool(&t1, &p1);
        c.set_pool(&t2, &p2);

        let amount: u128 = 1_000_000;
        let amount_net: u128 = 997_000;
        let mut v = c.price_near_to_token_in(t1.clone(), amount.into());
        let v_expected = amount_net * p1.reserve / (p1.ynear + amount_net);
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
