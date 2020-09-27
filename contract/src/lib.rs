// use near_sdk::json_types::U128;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};

pub mod util;
use crate::util::*;
//use std::collections::UnorderedMap;

// a way to optimize memory management
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod internal;

// Errors
// "E1" - Pool for this token already exists
// "E2" - all token arguments must be positive.
// "E3" - required amount of tokens to transfer is bigger then specified max.
// "E4" - computed amount of shares to receive is smaller the minimum required by the user.
// "E5" - not enough shares to redeem.
// "E6" - computed amount of near or reserve tokens is smaller than user required minimums for shares redeemption.
// "E7" - computed amount of buying tokens is smaller than user required minimum.
// "E8" - computed amount of selling tokens is bigger than user required maximum.
// "E9" - assets (tokens) must be different in token to token swap.
// "E10" - Pool is empty and can't make a swap.

/// PoolInfo is a helper structure to extract public data from a Pool
#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct PoolInfo {
    /// balance in yoctoNEAR
    pub ynear: Balance,
    pub reserve: Balance,
    /// total amount of participation shares. Shares are represented using the same amount of
    /// tailing decimals as the NEAR token, which is 24
    pub total_shares: Balance,
}

use std::fmt;

impl fmt::Display for PoolInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            f,
            "({}, {}, {})",
            self.ynear, self.reserve, self.total_shares
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
            ynear: self.ynear,
            reserve: self.reserve,
            total_shares: self.total_shares,
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
        util::assert_account(&owner, "Owner");
        Self {
            fee_dst: owner.clone(),
            owner,
            pools: UnorderedMap::new(env::current_account_id().as_bytes().to_vec()),
        }
    }

    pub fn set_fee_dst(&mut self, fee_dst: AccountId) {
        assert!(
            env::predecessor_account_id() == self.owner,
            "Only owner can change fee_dst"
        );
        assert!(
            env::is_valid_account_id(fee_dst.as_bytes()),
            "fee_dst account ID is invalid."
        );
        self.fee_dst = fee_dst;
    }

    /// Owner is an account (can be a multisig) who has management rights to update
    /// fee size.
    pub fn change_owner(&mut self, new_owner: AccountId) {
        self.assert_owner();
        assert!(
            env::is_valid_account_id(new_owner.as_bytes()),
            "fee_dst account ID is invalid."
        );
        env_log!("Changing owner from {} to {}", self.owner, new_owner);
        self.owner = new_owner;
    }

    /**********************
       POOL MANAGEMENT
    **********************/

    #[payable]
    pub fn check_number(&mut self, a: u128, aj: U128, b: Balance) {
        let d = env::attached_deposit();
        env_log!("u128: {}, U128: {:?}, Balance: {}, near: {}", a, aj, b, d);
    }

    /// Allows any user to creat a new near-token pool. Each pool is identified by the `token`
    /// account - which we call the Pool Reserve Token.
    /// If a pool for give token exists then "E1" assert exception is thrown.
    /// TODO: charge user for a storage created!
    #[payable]
    pub fn create_pool(&mut self, token: AccountId) {
        assert!(
            self.pools
                .insert(&token, &Pool::new(token.as_bytes().to_vec()))
                .is_none(),
            "E1"
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
    #[payable]
    pub fn add_liquidity(&mut self, token: AccountId, max_tokens: U128, min_shares: U128) {
        println!(">> ADD LIQUIDITY ARGS, {:?}, {:?}", max_tokens, min_shares);

        let mut p = self.must_get_pool(&token);
        let caller = env::predecessor_account_id();
        let shares_minted;
        let ynear_amount = env::attached_deposit();
        let added_reserve;
        let max_tokens: Balance = max_tokens.into();
        assert!(ynear_amount > 0 && max_tokens > 0, "E2");

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
            assert!(max_tokens >= added_reserve, "E3");
            assert!(u128::from(min_shares) <= shares_minted, "E4");

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
        // Prepare a callback for liquidity transfer rollback which we will attach later on.
        //prepare the callback so we can rollback if the transfer fails (for example: panic_msg: "Not enough balance" })
        let callback_args = format!(r#"{{ "token":"{}" }}"#, token).into();
        let callback = Promise::new(env::current_account_id()).function_call(
            "add_liquidity_transfer_callback".into(),
            callback_args,
            0,
            20 * TGAS,
        );

        //schedule a call to transfer nep21 tokens
        let args: Vec<u8> = format!(
            r#"{{ "owner_id":"{oid}","new_owner_id":"{noid}","amount":"{amount}" }}"#,
            oid = caller,
            noid = env::current_account_id(),
            amount = added_reserve
        )
        .into();

        Promise::new(token) //call the token contract
            .function_call(
                "transfer_from".into(),
                args,
                NEP21_STORAGE_DEPOSIT,
                20 * TGAS,
            )
            .then(callback); //after that, the callback will check success/failure

        // TODO:
        // Handling exception is work-in-progress in NEAR runtime
        // 1. rollback `p` on changes or move the pool update to a promise
        // 2. consider adding a lock to prevent other contracts calling and manipulate the prise before the token transfer will get finalized.
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
        assert!(shares_ > 0 && min_ynear > 0 && min_tokens > 0, "E2");

        let caller = env::predecessor_account_id();
        let mut p = self.must_get_pool(&token);
        let current_shares = p.shares.get(&caller).unwrap_or(0);
        assert!(current_shares >= shares_, "E5");

        let total_shares2 = u256::from(p.total_shares);
        let shares2 = u256::from(shares_);
        let ynear_amount = (shares2 * u256::from(p.ynear) / total_shares2).as_u128();
        let token_amount = (shares2 * u256::from(p.reserve) / total_shares2).as_u128();
        assert!(
            ynear_amount >= min_ynear && token_amount >= min_tokens,
            "E6"
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

        //send near to caller
        let send_near = Promise::new(caller.clone()) // caller is clone because it has to be used later
            .transfer(ynear_amount);
        //send token to caller
        let send_tokens = self.schedule_nep21_tansfer(
            &token,
            env::current_account_id(),
            env::predecessor_account_id(),
            token_amount,
        );
        //schedule  both in parallel
        send_near.and(send_tokens);
        //TODO COMPLEX-CALLBACKS
    }

    /// Returns the owner balance of shares of a pool identified by token.
    pub fn shares_balance_of(&self, token: AccountId, owner: AccountId) -> U128 {
        return self
            .must_get_pool(&token)
            .shares
            .get(&owner)
            .unwrap_or(0)
            .into();
    }

    /**********************
    CLP market functions
    **********************/

    /// Swaps NEAR to `token` and transfers the reserve tokens to the caller.
    /// Caller attaches near tokens he wants to swap to the transacion under a condition of
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

    /// Swaps `tokens_paid` of `token` to NEAR and transfers NEAR to the caller under a
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
    /// Caller defines the amount of NEAR he wants to receive under a condition of not spending
    /// more than `max_tokens` of `token`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of `token`
    /// for this contract.
    /// TODO: Transaction will panic if a caller doesn't provide enough allowance.
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
    /// Caller defines the amount of tokens he wants to swap under a condition of
    /// receving at least `min_to_tokens`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of
    /// `from` token for this contract.
    //// TODO: Transaction will panic if a caller doesn't provide enough allowance.
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
    /// Caller defines the amount of tokens he wants to receive under a of not spending
    /// more than `max_from_tokens`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of
    /// `from` token for this contract.
    //// TODO: Transaction will panic if a caller doesn't provide enough allowance.
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
        assert!(ynear_in > 0, "E2");
        let p = self.must_get_pool(&token);
        return self.calc_out_amount(ynear_in, p.ynear, p.reserve).into();
    }

    /// Calculates amount of NEAR user will need to swap if he wants to receive
    /// `tokens_out` of `token`
    pub fn price_near_to_token_out(&self, token: AccountId, tokens_out: U128) -> U128 {
        let tokens_out: u128 = tokens_out.into();
        assert!(tokens_out > 0, "E2");
        let p = self.must_get_pool(&token);
        return self.calc_in_amount(tokens_out, p.reserve, p.ynear).into();
    }

    /// Calculates amount of NEAR user will recieve when swapping `tokens_in` for NEAR.
    pub fn price_token_to_near_in(&self, token: AccountId, tokens_in: U128) -> U128 {
        let tokens_in: u128 = tokens_in.into();
        assert!(tokens_in > 0, "E2");
        let p = self.must_get_pool(&token);
        return self.calc_out_amount(tokens_in, p.reserve, p.ynear).into();
    }

    /// Calculates amount of tokens user will need to swap if he wants to receive
    /// `tokens_out` of `tokens`
    pub fn price_token_to_near_out(&self, token: AccountId, ynear_out: U128) -> U128 {
        let ynear_out: u128 = ynear_out.into();
        assert!(ynear_out > 0, "E2");
        let p = self.must_get_pool(&token);
        return self.calc_in_amount(ynear_out, p.ynear, p.reserve).into();
    }

    /// Calculates amount of tokens `to` user will receive when swapping `tokens_in` of `from`
    pub fn price_token_to_token_in(&self, from: AccountId, to: AccountId, tokens_in: U128) -> U128 {
        let tokens_in: u128 = tokens_in.into();
        assert!(tokens_in > 0, "E2");
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
        assert!(tokens_out > 0, "E2");
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
}
//-------------------------
// END CONTRACT PUBLIC API
//-------------------------

//#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod unit_tests_fun_token;

//#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    use unit_tests_fun_token::FungibleToken;

    struct Accounts {
        current: AccountId,
        owner: AccountId,
        predecessor: AccountId,
        token1: AccountId,
        token2: AccountId,
    }

    struct Ctx {
        accounts: Accounts,
        vm: VMContext,
        token_supply: u128,
    }

    impl Ctx {
        fn create_accounts() -> Accounts {
            return Accounts {
                current: "clp_near".to_string(),
                owner: "owner_near".to_string(),
                predecessor: "pre_near".to_string(),
                token1: "token1_near".to_string(),
                token2: "token2_near".to_string(),
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
                prepaid_gas: 10u64.pow(18),
                random_seed: vec![0, 1, 2],
                is_view,
                output_data_receivers: vec![],
                epoch_height: 19,
            };
            return Self {
                accounts: accounts,
                vm: vm,
                token_supply: 1_000_000_000_000_000u128,
            };
        }

        pub fn set_deposit_for_token_op(&mut self) {
            let storage_price_per_byte: Balance = NEP21_STORAGE_DEPOSIT;
            self.set_deposit(storage_price_per_byte * 6); // arbitrary number easy to recoginze)
        }

        pub fn set_deposit(&mut self, attached_deposit: Balance) {
            self.vm.attached_deposit = attached_deposit;
            testing_env!(self.vm.clone());
        }
    }

    fn init() -> (Ctx, NearCLP) {
        let ctx = Ctx::new(vec![], false);
        testing_env!(ctx.vm.clone());
        let contract = NearCLP::new(ctx.accounts.owner.clone());
        return (ctx, contract);
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
    #[should_panic(expected = "E1")]
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
                    ynear: 0,
                    reserve: 0,
                    total_shares: 0
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
        let a = ctx.accounts.predecessor.clone();
        let t = ctx.accounts.token1.clone();
        let mut token1 = FungibleToken::new(a.clone(), ctx.token_supply.into());
        check_and_create_pool(&mut c, &t);
        assert_eq!(
            token1.total_supply, ctx.token_supply,
            "Token total supply must be correct"
        );

        let ynear_deposit = NDENOM * 11;
        let token_deposit = 500u128;
        ctx.set_deposit_for_token_op();
        token1.inc_allowance(t.clone(), token_deposit.into());

        ctx.set_deposit(ynear_deposit);
        c.add_liquidity(t.clone(), token_deposit.into(), ynear_deposit.into());

        let p = c.pool_info(&t).expect("Pool should exist");
        println!("Pool info: {}", p);
        assert_eq!(p.ynear, ynear_deposit, "Near balance should be correct");
        assert_eq!(p.reserve, token_deposit, "Token balance should be correct");
    }

    // TODO tests
    // + add liquidity with max_balance > allowance
}
