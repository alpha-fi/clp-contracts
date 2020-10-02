use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};

pub mod types;
pub mod util;

use crate::types::*;
use crate::util::*;

// acc way to optimize memory management
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod internal;

// Errors
// E1: pool already exists
// E2: all token arguments must be positive.
// E3: required amount of tokens to transfer is bigger then specified max.
// E4: computed amount of shares to receive is smaller then the minimum required by the user.
// E5: can't withdraw more shares then currently owned
// E6: computed amount of near or reserve tokens is smaller than user required minimums for shares redeemption.
// E7: computed amount of buying tokens is smaller than user required minimum.
// E8: computed amount of selling tokens is bigger than user required maximum.
// E9: assets (tokens) must be different in token to token swap.
// E10: Pool is empty and can't make acc swap.
// E11: Insufficient amount of shares balance.
// E12: Insufficient amount of NEAR attached

/// PoolInfo is acc helper structure to extract public data from acc Pool
#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct PoolInfo {
    /// balance in yoctoNEAR
    pub ynear: U128,
    pub reserve: U128,
    /// total amount of participation shares. Shares are represented using the same amount of
    /// tailing decimals as the NEAR token, which is 24
    pub total_shares: U128,
}

use std::fmt;

impl fmt::Display for PoolInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            f,
            "({}, {}, {})",
            self.ynear.0, self.reserve.0, self.total_shares.0
        );
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    ynear: Balance,
    reserve: Balance,
    shares: UnorderedMap<AccountId, Balance>,
    /// check `PoolInfo.total_shares`
    total_shares: Balance,
}

impl Pool {
    pub fn new(pool_id: Vec<u8>) -> Self {
        Self {
            ynear: 0,
            reserve: 0,
            shares: UnorderedMap::new(pool_id),
            total_shares: 0,
        }
    }

    pub fn pool_info(&self) -> PoolInfo {
        PoolInfo {
            ynear: self.ynear.into(),
            reserve: self.reserve.into(),
            total_shares: self.total_shares.into(),
        }
    }
}

/// NearCLP is the main contract for managing the swap pools and liquidity.
/// It implements the NEARswap functionality.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearCLP {
    pub fee_dst: AccountId,
    pub owner: AccountId,
    // we are using unordered map because it allows to iterate over the pools
    pools: UnorderedMap<AccountId, Pool>,
}

impl Default for NearCLP {
    fn default() -> Self {
        panic!("Fun token should be initialized before usage")
    }
}

//-------------------------
// CONTRACT PUBLIC API
//-------------------------
#[near_bindgen]
impl NearCLP {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        util::assert_account_is_valid(&owner);
        Self {
            fee_dst: owner.clone(),
            owner,
            pools: UnorderedMap::new(env::current_account_id().as_bytes().to_vec()),
        }
    }

    /// Updates the fee destination destination account
    pub fn set_fee_dst(&mut self, fee_dst: AccountId) {
        self.assert_owner();
        util::assert_account_is_valid(&fee_dst);
        self.fee_dst = fee_dst;
    }

    /// Owner is an account (can be acc multisig) who has management rights to update
    /// fee size.
    pub fn change_owner(&mut self, new_owner: AccountId) {
        self.assert_owner();
        util::assert_account_is_valid(&new_owner);
        env_log!("Changing owner from {} to {}", self.owner, new_owner);
        self.owner = new_owner;
    }

    /**********************
     POOL MANAGEMENT
    *********************/

    /// Allows any user to creat acc new near-token pool. Each pool is identified by the `token`
    /// account - which we call the Pool Reserve Token.
    /// If acc pool for give token exists then "E1" assert exception is thrown.
    /// TODO: charge user for acc storage created!
    #[payable]
    pub fn create_pool(&mut self, token: AccountId) {
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
        let ynear_amount = env::attached_deposit();
        let added_reserve;
        let max_tokens: Balance = max_tokens.into();
        assert!(
            ynear_amount > 0 && max_tokens > 0,
            "E2: balance arguments must be >0"
        );

        env_log!("adding liquidity for {} ynear", &ynear_amount);

        // the very first deposit -- we define the constant ratio
        if p.total_shares == 0 {
            p.ynear = ynear_amount;
            shares_minted = p.ynear;
            p.total_shares = shares_minted;
            added_reserve = max_tokens;
            p.reserve = added_reserve;
            p.shares.insert(&caller, &p.ynear);
        } else {
            added_reserve = ynear_amount * p.reserve / p.ynear + 1;
            shares_minted = ynear_amount * p.total_shares / ynear_amount;
            assert!(
                max_tokens >= added_reserve,
                format!(
                    "E3: needs to transfer {} of tokens and it's bigger then specified  maximum",
                    added_reserve
                )
            );
            assert!(
                u128::from(min_shares) <= shares_minted,
                format!(
                    "E4: amount minted shares ({}) is smaller then the required minimum",
                    shares_minted
                )
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
        println!(
            ">> in contract, attached deposit: {}, PoolInfo: {}",
            ynear_amount,
            p.pool_info()
        );
        self.set_pool(&token, &p);

        // TODO: do proper rollback
        // Prepare acc callback for liquidity transfer rollback which we will attach later on.
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
        // 1. rollback `p` on changes or move the pool update to acc promise
        // 2. consider adding acc lock to prevent other contracts calling and manipulate the prise before the token transfer will get finalized.

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
    /// Caller attaches near tokens he wants to swap to the transacion under acc condition of
    /// receving at least `min_tokens` of `token`.
    #[payable]
    pub fn swap_near_to_token_exact_in(&mut self, token: AccountId, min_tokens: U128) {
        self._swap_near_exact_in(
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
        self._swap_near_exact_in(
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
        self._swap_near_exact_out(
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
        self._swap_near_exact_out(
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
    /// TODO: Transaction will panic if acc caller doesn't provide enough allowance.
    #[payable]
    pub fn swap_token_to_near_exact_in(
        &mut self,
        token: AccountId,
        tokens_paid: U128,
        min_ynear: U128,
    ) {
        let b = env::predecessor_account_id();
        self._swap_token_exact_in(&token, tokens_paid.into(), min_ynear.into(), b.clone(), b);
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
        self._swap_token_exact_in(&token, tokens_paid.into(), min_ynear.into(), b, recipient);
    }

    /// Swaps `token` to NEAR and transfers NEAR to the caller.
    /// Caller defines the amount of NEAR he wants to receive under acc condition of not spending
    /// more than `max_tokens` of `token`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of `token`
    /// for this contract.
    /// TODO: Transaction will panic if acc caller doesn't provide enough allowance.
    #[payable]
    pub fn swap_token_to_near_exact_out(
        &mut self,
        token: AccountId,
        ynear_out: U128,
        max_tokens: U128,
    ) {
        let b = env::predecessor_account_id();
        self._swap_token_exact_out(&token, ynear_out.into(), max_tokens.into(), b.clone(), b);
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
        self._swap_token_exact_out(&token, ynear_out.into(), max_tokens.into(), b, recipient);
    }

    /// Swaps two different tokens.
    /// Caller defines the amount of tokens he wants to swap under acc condition of
    /// receving at least `min_to_tokens`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of
    /// `from` token for this contract.
    //// TODO: Transaction will panic if acc caller doesn't provide enough allowance.
    #[payable]
    pub fn swap_tokens_exact_in(
        &mut self,
        from: AccountId,
        to: AccountId,
        from_tokens: U128,
        min_to_tokens: U128,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_in(
            &from,
            &to,
            from_tokens.into(),
            min_to_tokens.into(),
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
        from_tokens: U128,
        min_to_tokens: U128,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_in(
            &from,
            &to,
            from_tokens.into(),
            min_to_tokens.into(),
            b,
            recipient,
        );
    }

    /// Swaps two different tokens.
    /// Caller defines the amount of tokens he wants to receive under acc of not spending
    /// more than `max_from_tokens`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of
    /// `from` token for this contract.
    //// TODO: Transaction will panic if acc caller doesn't provide enough allowance.
    #[payable]
    pub fn swap_tokens_exact_out(
        &mut self,
        from: AccountId,
        to: AccountId,
        to_tokens: U128,
        max_from_tokens: U128,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_out(
            &from,
            &to,
            to_tokens.into(),
            max_from_tokens.into(),
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
        to_tokens: U128,
        max_from_tokens: U128,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_out(
            &from,
            &to,
            to_tokens.into(),
            max_from_tokens.into(),
            b,
            recipient,
        );
    }

    /// Calculates amount of tokens user will recieve when swapping `ynear_in` for `token`
    /// assets
    pub fn price_near_to_token_in(&self, token: AccountId, ynear_in: U128) -> U128 {
        let ynear_in: u128 = ynear_in.into();
        assert!(ynear_in > 0, "E2: balance arguments must be >0");
        let p = self.must_get_pool(&token);
        return self.calc_out_amount(ynear_in, p.ynear, p.reserve).into();
    }

    /// Calculates amount of NEAR user will need to swap if he wants to receive
    /// `tokens_out` of `token`
    pub fn price_near_to_token_out(&self, token: AccountId, tokens_out: U128) -> U128 {
        let tokens_out: u128 = tokens_out.into();
        assert!(tokens_out > 0, "E2: balance arguments must be >0");
        let p = self.must_get_pool(&token);
        return self.calc_in_amount(tokens_out, p.reserve, p.ynear).into();
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
        return self.calc_in_amount(ynear_out, p.ynear, p.reserve).into();
    }

    /// Calculates amount of tokens `to` user will receive when swapping `tokens_in` of `from`
    pub fn price_token_to_token_in(&self, from: AccountId, to: AccountId, tokens_in: U128) -> U128 {
        let tokens_in: u128 = tokens_in.into();
        assert!(tokens_in > 0, "E2: balance arguments must be >0");
        let p1 = self.must_get_pool(&from);
        let p2 = self.must_get_pool(&to);
        let (_, tokens_out) = self._price_swap_tokens_in(&p1, &p2, tokens_in);
        return tokens_out.into();
    }

    /// Calculates amount of tokens `from` user will need to swap if he wants to receive
    /// `tokens_out` of tokens `to`
    pub fn price_token_to_token_out(
        &self,
        from: AccountId,
        to: AccountId,
        tokens_out: U128,
    ) -> U128 {
        let tokens_out: u128 = tokens_out.into();
        assert!(tokens_out > 0, "E2: balance arguments must be >0");
        let p1 = self.must_get_pool(&from);
        let p2 = self.must_get_pool(&to);
        let (_, tokens_in) = self._price_swap_tokens_out(&p1, &p2, tokens_out);
        return tokens_in.into();
    }

    pub fn add_liquidity_transfer_callback(&mut self, token: AccountId) {
        println!("enter add_liquidity_transfer_callback");
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Can be called only as acc callback"
        );

        // TODO: simulation doesn't allow using acc promise inside callbacks.
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

    /// granularity is the smallest amount of tokens (in the internal denomination) which
    /// may be minted, sent or burned at any time.
    #[allow(unused)]
    pub fn decimals(&self, token: AccountId) -> u8 {
        24
    }

    /// Returns total balance of acc given subtoken. Implements the NEP-MFT standard.
    pub fn total_supply(&self, token: AccountId) -> U128 {
        match self.pools.get(&token) {
            None => 0.into(),
            Some(p) => p.total_shares.into(),
        }
    }

    /// Returns the owner balance of shares of acc pool identified by token.
    pub fn balance_of(&self, token: AccountId, owner: AccountId) -> U128 {
        self.must_get_pool(&token)
            .shares
            .get(&owner)
            .unwrap_or(0)
            .into()
    }

    /// Transfer `amount` of LP Shares of acc pool identified by the `token` (must be acc valid
    /// AccountID related to acc registered pool) from to acc `recipeint` contract.
    /// Implements the NEP-MFT interface.
    /// `recipient` MUST be acc contract address.
    /// The recipient contract MUST implement `MFTRecipient` interface.
    /// `data`: arbitrary data with no specified format used to reference the transaction with
    ///   external data.
    /// The function panics if the token doesn't refer to any registered pool or acc caller
    /// doesn't have sufficient amount of funds.
    #[payable]
    pub fn transfer_to_sc(
        &mut self,
        token: String,
        recipient: AccountId,
        amount: U128,
        /*#[serializer(borsh)]*/ data: Data,
    ) -> bool {
        self._transfer(token, recipient, amount, data, true)
    }

    /// Transfer `amount` of LP Shares of acc pool identified by the `token` (must be acc valid
    /// AccountID related to acc registered pool) from to acc `recipeint` account.
    /// Implements the NEP-MFT interface.
    /// `recipient` MUST NOT be acc contract address.
    /// `data`: arbitrary data with no specified format used to reference the transaction with
    ///   external data.
    /// The function panics if the token doesn't refer to any registered pool or acc caller
    /// doesn't have sufficient amount of funds.
    #[payable]
    pub fn transfer(
        &mut self,
        token: String,
        recipient: AccountId,
        amount: U128,
        /*#[serializer(borsh)]*/ data: Data,
    ) -> bool {
        self._transfer(token, recipient, amount, data, false)
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

//#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

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

        pub fn set_gas_and_deposit_for_token_op(&mut self) {
            // 6 is arbitrary number easy to recoginze)
            self.vm.attached_deposit = NEP21_STORAGE_DEPOSIT * 120;
            self.vm.prepaid_gas = MAX_GAS;
            testing_env!(self.vm.clone());
        }

        pub fn set_deposit(&mut self, attached_deposit: Balance) {
            self.vm.attached_deposit = attached_deposit;
            testing_env!(self.vm.clone());
        }
    }

    fn _init(attached_near: Balance) -> (Ctx, NearCLP) {
        let mut ctx = Ctx::new(vec![], false);
        ctx.vm.attached_deposit = attached_near;
        testing_env!(ctx.vm.clone());
        let contract = NearCLP::new(ctx.accounts.owner.clone());
        return (ctx, contract);
    }
    fn init() -> (Ctx, NearCLP) {
        _init(0)
    }
    fn init_with_storage_deposit() -> (Ctx, NearCLP) {
        _init(NEP21_STORAGE_DEPOSIT * 120)
    }

    // TODO - fix this test.
    // #[test]
    // #[should_panic]
    // fn test_new_twice_fails() {
    //     let (ctx, _c) = init();
    //     NearCLP::new(ctx.accounts.current);
    // }

    #[test]
    fn change_owner() {
        let (mut ctx, mut c) = init();

        assert_eq!(&c.owner, &ctx.accounts.owner);

        ctx.vm.predecessor_account_id = ctx.accounts.owner;
        testing_env!(ctx.vm);
        let owner2 = "new_owner_near".to_string();
        c.change_owner(owner2.clone());
        assert_eq!(c.owner, owner2);
    }

    #[test]
    #[should_panic(expected = "Only the owner can call this function")]
    fn change_owner_other_account() {
        let (_, mut c) = init();
        let owner2 = "new_owner_near".to_string();
        c.change_owner(owner2.clone());
    }

    #[test]
    #[should_panic(expected = "E1: pool already exists")]
    fn create_twice_same_pool_fails() {
        let (ctx, mut c) = init();
        c.create_pool(ctx.accounts.token1.clone());

        // let's check firstly the pool is there
        let pools = c.list_pools();
        let expected = [ctx.accounts.token1.clone()];
        assert_eq!(pools, expected);

        //
        c.create_pool(ctx.accounts.token1);
    }

    fn check_and_create_pool(c: &mut NearCLP, token: &AccountId) {
        c.create_pool(token.to_string());
        match c.pool_info(token) {
            None => panic!("Pool for {} token is expected"),
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

        // in unit tests we can't do cross contract calls, so we can't check token1 updates.
        check_and_create_pool(&mut c, &t);

        let ynear_deposit = 12 * NDENOM;
        let token_deposit = 2 * NDENOM;
        let ynear_deposit_j = U128::from(ynear_deposit);
        ctx.set_gas_and_deposit_for_token_op();
        ctx.set_deposit(ynear_deposit);

        c.add_liquidity(t.clone(), token_deposit.into(), ynear_deposit.into());

        let p = c.pool_info(&t).expect("Pool should exist");
        let expected_pool = PoolInfo {
            ynear: ynear_deposit_j,
            reserve: token_deposit.into(),
            total_shares: ynear_deposit_j,
        };
        assert_eq!(p, expected_pool, "pool_info should be correct");
        let predecessor_shares = c.balance_of(t.clone(), ctx.accounts.predecessor);
        assert_eq!(
            predecessor_shares, ynear_deposit_j,
            "LP should have correct amount of shares"
        );
        assert_eq!(
            c.total_supply(t),
            ynear_deposit_j,
            "LP should have correct amount of shares"
        );

        // total supply of an unknown token must be 0
        assert_eq!(
            to_num(c.total_supply("unknown-token".to_string())),
            0,
            "LP should have correct amount of shares"
        );

        // TODO tests
        // + add liquidity with max_balance > allowance
    }
    #[test]
    fn withdraw_happy_path() {
        let (ctx, mut c) = init_with_storage_deposit();
        let acc = ctx.accounts.predecessor.clone();
        let t = ctx.accounts.token1.clone();

        let shares_bal = 12 * NDENOM;
        let mut shares_map = UnorderedMap::new("123".as_bytes().to_vec());
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
        let mut shares_map = UnorderedMap::new("123".as_bytes().to_vec());
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
        c.transfer(t.clone(), alice.clone(), amount.into(), Data(Vec::new()));

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
        assert_out(10, 2 * G, G, 4); // again rounding, should be 9.97
        assert_out(1_000, 2 * G, G, 996 / 2); // rounding again...
        assert_out(100_000, 2 * NDENOM, NDENOM, 99699 / 2); // 49849

        // ## test 1:2  ## - we expect 2x*0.997
        assert_out(1, G, 2 * G, 1); // 0 because we cut the decimals (should be 1.997)
        assert_out(1_000, G, 2 * G, 1993); // rounding again...
        assert_out(100_000, NDENOM, 2 * NDENOM, 199399);
    }

    fn to_num(a: U128) -> u128 {
        a.into()
    }
}
