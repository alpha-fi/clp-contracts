// use near_sdk::json_types::U128;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Promise, PromiseResult};
use uint::construct_uint;
//use std::collections::UnorderedMap;

// a way to optimize memory management
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod internal;
mod nep21;
mod util;

// Prepaid gas for making a single simple call.
const SINGLE_CALL_GAS: u64 = 200_000_000_000_000;
const TEN_NEAR: u128 = 10_000_000_000_000_000_000_000_000;

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

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

/// Interface for the contract itself.
#[ext_contract(ext_self)]
pub trait SelfContract {
    /// callback to check the result of the add_liquidity action
    fn add_liquidity_transfer_callback(&mut self, token: AccountId);
}

/// PoolInfo is a helper structure to extract public data from a Pool
#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct PoolInfo {
    pub near_bal: Balance,
    pub token_bal: Balance,
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
            self.near_bal, self.token_bal, self.total_shares
        );
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    near_bal: Balance,
    token_bal: Balance,
    shares: UnorderedMap<AccountId, Balance>,
    /// check `PoolInfo.total_shares`
    total_shares: Balance,
}

impl Pool {
    pub fn new(pool_id: Vec<u8>) -> Self {
        Self {
            near_bal: 0,
            token_bal: 0,
            shares: UnorderedMap::new(pool_id),
            total_shares: 0,
        }
    }

    pub fn pool_info(&self) -> PoolInfo {
        PoolInfo {
            near_bal: self.near_bal,
            token_bal: self.token_bal,
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
        self.owner = new_owner;
    }

    /**********************
       POOL MANAGEMENT
    **********************/

    /// Allows any user to creat a new near-token pool. Each pool is identified by the `token`
    /// account - which we call the Pool Reserve Token.
    /// If a pool for give token exists then "E1" assert exception is thrown.
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
    /// The supplied funds must preserver current ratio of the liquidity pool.
    #[payable]
    pub fn add_liquidity(
        &mut self,
        token: AccountId,
        max_token_amount: Balance,
        min_shares_amont: Balance,
    ) {
        let mut p = self.get_pool(&token);
        let caller = env::predecessor_account_id();
        let shares_minted;
        let near_amount = env::attached_deposit();
        let computed_token_amount;
        assert!(near_amount > 0 && max_token_amount > 0, "E2");

        // the very first deposit -- we define the constant ratio
        if p.total_shares == 0 {
            env::log(b"Creating a first deposit");
            p.near_bal = near_amount;
            shares_minted = p.near_bal;
            p.total_shares = shares_minted;
            computed_token_amount = max_token_amount;
            p.token_bal = computed_token_amount;
            p.shares.insert(&caller, &p.near_bal);
        } else {
            computed_token_amount = near_amount * p.token_bal / p.near_bal + 1;
            shares_minted = near_amount * p.total_shares / near_amount;
            assert!(max_token_amount >= computed_token_amount, "E3");
            assert!(min_shares_amont <= shares_minted, "E4");

            p.shares.insert(
                &caller,
                &(p.shares.get(&caller).unwrap_or(0) + shares_minted),
            );
            p.token_bal += computed_token_amount;
            p.near_bal += near_amount;
            p.total_shares += shares_minted;
        }

        env::log(
            format!(
                "Minting {} of shares for {} NEAR and {} reserve tokens",
                shares_minted, near_amount, computed_token_amount
            )
            .as_bytes(),
        );
        println!(
            ">> in contract, attached deposit: {}, PoolInfo: {}",
            near_amount,
            p.pool_info()
        );

        self.set_pool(&token, &p);

        // Prepare a callback for liquidity transfer which we will attach later on.
        let callback_args = format!(r#"{{ "token":"{tok}" }}"#, tok = token).into();
        let callback = Promise::new(env::current_account_id()).function_call(
            "add_liquidity_transfer_callback".into(),
            callback_args,
            0,
            SINGLE_CALL_GAS / 2,
        );
        //let callback=ext_self::add_liquidity_transfer_callback(env::current_account_id(),&token,0,SINGLE_CALL_GAS/2);
        //let callback_args="{}".into();
        //let callback=Promise::new(token.clone()).function_call("get_total_supply".into(), callback_args, 0, SINGLE_CALL_GAS/2);

        let args: Vec<u8> = format!(
            r#"{{ "owner_id":"{oid}","new_owner_id":"{noid}","amount":"{amount}" }}"#,
            oid = caller,
            noid = env::current_account_id(),
            amount = computed_token_amount
        )
        .into();

        Promise::new(token) //call the token contract
            .function_call("transfer_from".into(), args, 0, SINGLE_CALL_GAS / 2)
            .then(callback);

        /*
        REPLACED BY CODE ABOVE
        nep21::ext_nep21::transfer_from(
            caller,
            env::current_account_id(),
            token_amount.into(),
            &token,
            0,
            SINGLE_CALL_GAS,
        );
        */
        // TODO:
        // Handling exception is work-in-progress in NEAR runtime
        // 1. rollback `p` on changes or move the pool update to a promise
        // 2. consider adding a lock to prevent other contracts calling and manipulate the prise before the token transfer will get finalized.
    }

    /// Redeems `shares` for liquidity stored in this pool with condition of getting at least
    /// `min_near` of Near and `min_tokens` of reserve. Shares are note exchengable between
    /// different pools.
    pub fn withdraw_liquidity(
        &mut self,
        token: AccountId,
        shares: Balance,
        min_near: Balance,
        min_tokens: Balance,
    ) {
        assert!(shares > 0 && min_near > 0 && min_tokens > 0, "E2");
        let caller = env::predecessor_account_id();
        let mut p = self.get_pool(&token);
        let current_shares = p.shares.get(&caller).unwrap_or(0);
        assert!(current_shares >= shares, "E5");

        let near_amount = shares * p.near_bal / p.total_shares;
        let token_amount = shares * p.token_bal / p.total_shares;
        assert!(near_amount >= min_near && token_amount >= min_tokens, "E6");

        env::log(
            format!(
                "Reedeming {} shares for {} NEAR and {} reserve tokens",
                shares, near_amount, token_amount
            )
            .as_bytes(),
        );
        p.shares.insert(&caller, &(current_shares - shares));
        p.total_shares -= shares;
        p.token_bal -= token_amount;
        p.near_bal -= near_amount;

        Promise::new(caller.clone()) // caller is clone because it has to be used later
            .transfer(near_amount as u128)
            .and(nep21::ext_nep21::transfer(
                caller,
                token_amount.into(),
                &token,
                0,
                SINGLE_CALL_GAS,
            ));
    }

    /// Returns the owner balance of shares of a pool identified by token.
    pub fn shares_balance_of(&self, token: AccountId, owner: AccountId) -> Balance {
        return self.get_pool(&token).shares.get(&owner).unwrap_or(0);
    }

    /**********************
    CLP market functions
    **********************/

    /// Swaps NEAR to `token` and transfers the reserve tokens to the caller.
    /// Caller attaches near tokens he wants to swap to the transacion under a condition of
    /// receving at least `min_tokens`.
    #[payable]
    pub fn swap_near_to_reserve_exact_in(&mut self, token: AccountId, min_tokens: Balance) {
        self._swap_near_exact_in(
            &token,
            env::attached_deposit(),
            min_tokens,
            env::predecessor_account_id(),
        );
    }

    /// Same as `swap_near_to_reserve_exact_in`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    #[payable]
    pub fn swap_near_to_reserve_exact_in_xfr(
        &mut self,
        token: AccountId,
        min_tokens: Balance,
        recipient: AccountId,
    ) {
        self._swap_near_exact_in(&token, env::attached_deposit(), min_tokens, recipient);
    }

    /// Swaps NEAR to `token` and transfers the reserve tokens to the caller.
    /// Caller attaches maximum amount of NEAR he is willing to swap to receive `tokens_out`
    /// of `token` wants to swap to the transacion. Surplus of NEAR tokens will be returned.
    /// Transaction will panic if the caller doesn't attach enough NEAR tokens.
    #[payable]
    pub fn swap_near_to_reserve_exact_out(&mut self, token: AccountId, tokens_out: Balance) {
        let b = env::predecessor_account_id();
        self._swap_near_exact_out(&token, tokens_out, env::attached_deposit(), b.clone(), b);
    }

    /// Same as `swap_near_to_reserve_exact_out`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    #[payable]
    pub fn swap_near_to_reserve_exact_out_xfr(
        &mut self,
        token: AccountId,
        tokens_out: Balance,
        recipient: AccountId,
    ) {
        self._swap_near_exact_out(
            &token,
            tokens_out,
            env::attached_deposit(),
            env::predecessor_account_id(),
            recipient,
        );
    }

    /// Swaps `token` to NEAR and transfers NEAR to the caller under a condition of
    /// receving at least `min_near`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of `token`
    /// for this contract.
    /// TODO: Transaction will panic if a caller doesn't provide enough allowance.
    #[payable]
    pub fn swap_reserve_to_near_exact_in(
        &mut self,
        token: AccountId,
        tokens_paid: Balance,
        min_near: Balance,
    ) {
        let b = env::predecessor_account_id();
        self._swap_reserve_exact_in(&token, tokens_paid, min_near, b.clone(), b);
    }

    /// Same as `swap_reserve_to_near_exact_in`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    #[payable]
    pub fn swap_reserve_to_near_exact_in_xfr(
        &mut self,
        token: AccountId,
        tokens_paid: Balance,
        min_near: Balance,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_reserve_exact_in(&token, tokens_paid, min_near, b, recipient);
    }

    /// Swaps `token` to NEAR and transfers NEAR to the caller.
    /// Caller defines the amount of NEAR he wants to receive under a condition of not spending
    /// more than `max_tokens` of `token`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of `token`
    /// for this contract.
    /// TODO: Transaction will panic if a caller doesn't provide enough allowance.
    pub fn swap_reserve_to_near_exact_out(
        &mut self,
        token: AccountId,
        near_out: Balance,
        max_tokens: Balance,
    ) {
        let b = env::predecessor_account_id();
        self._swap_reserve_exact_out(&token, near_out, max_tokens, b.clone(), b);
    }

    /// Same as `swap_reserve_to_near_exact_out`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    pub fn swap_reserve_to_near_exact_out_xfr(
        &mut self,
        token: AccountId,
        near_out: Balance,
        max_tokens: Balance,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_reserve_exact_out(&token, near_out, max_tokens, b, recipient);
    }

    /// Swaps two different tokens.
    /// Caller defines the amount of tokens he wants to swap under a condition of
    /// receving at least `min_tokens_to`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of
    /// `token_from` for this contract.
    //// TODO: Transaction will panic if a caller doesn't provide enough allowance.
    pub fn swap_tokens_exact_in(
        &mut self,
        from: AccountId,
        to: AccountId,
        tokens_from: Balance,
        min_tokens_to: Balance,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_in(&from, &to, tokens_from, min_tokens_to, b.clone(), b);
    }

    /// Same as `swap_tokens_exact_in`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    pub fn swap_tokens_exact_in_xfr(
        &mut self,
        from: AccountId,
        to: AccountId,
        tokens_from: Balance,
        min_tokens_to: Balance,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_in(&from, &to, tokens_from, min_tokens_to, b, recipient);
    }

    /// Swaps two different tokens.
    /// Caller defines the amount of tokens he wants to receive under a of not spending
    /// more than `max_tokens_from`.
    /// Preceeding to this transaction, caller has to create sufficient allowance of
    /// `token_from` for this contract.
    //// TODO: Transaction will panic if a caller doesn't provide enough allowance.
    pub fn swap_tokens_exact_out(
        &mut self,
        from: AccountId,
        to: AccountId,
        tokens_to: Balance,
        max_tokens_from: Balance,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_out(&from, &to, tokens_to, max_tokens_from, b.clone(), b);
    }

    /// Same as `swap_tokens_exact_out`, but user additionly specifies the `recipient`
    /// who will receive the tokens after the swap.
    pub fn swap_tokens_exact_out_xfr(
        &mut self,
        from: AccountId,
        to: AccountId,
        tokens_to: Balance,
        max_tokens_from: Balance,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_out(&from, &to, tokens_to, max_tokens_from, b, recipient);
    }

    /// Calculates amount of tokens user will recieve when swapping `near_in` for `token`
    /// assets
    pub fn price_near_to_token_in(&self, token: AccountId, near_in: Balance) -> Balance {
        assert!(near_in > 0, "E2");
        let p = self.get_pool(&token);
        return self.calc_out_amount(near_in, p.near_bal, p.token_bal);
    }

    /// Calculates amount of NEAR user will need to swap if he wants to receive
    /// `tokens_out` of `tokens`
    pub fn price_near_to_token_out(&self, token: AccountId, tokens_out: Balance) -> Balance {
        assert!(tokens_out > 0, "E2");
        let p = self.get_pool(&token);
        return self.calc_in_amount(tokens_out, p.token_bal, p.near_bal);
    }

    /// Calculates amount of NEAR user will recieve when swapping `tokens_in` for NEAR.
    pub fn price_token_to_near_in(&self, token: AccountId, tokens_in: Balance) -> Balance {
        assert!(tokens_in > 0, "E2");
        let p = self.get_pool(&token);
        return self.calc_out_amount(tokens_in, p.token_bal, p.near_bal);
    }

    /// Calculates amount of tokens user will need to swap if he wants to receive
    /// `tokens_out` of `tokens`
    pub fn price_token_to_near_out(&self, token: AccountId, near_out: Balance) -> Balance {
        assert!(near_out > 0, "E2");
        let p = self.get_pool(&token);
        return self.calc_in_amount(near_out, p.near_bal, p.token_bal);
    }

    /// Calculates amount of tokens `to` user will receive when swapping `tokens_in` of `from`
    pub fn price_token_to_token_in(
        &self,
        from: AccountId,
        to: AccountId,
        tokens_in: Balance,
    ) -> Balance {
        assert!(tokens_in > 0, "E2");
        let p1 = self.get_pool(&from);
        let p2 = self.get_pool(&to);
        let (_, tokens_out) = self._price_swap_tokens_in(&p1, &p2, tokens_in);
        return tokens_out;
    }

    /// Calculates amount of tokens `from` user will need to swap if he wants to receive
    /// `tokens_out` of tokens `to`
    pub fn price_token_to_token_out(
        &self,
        from: AccountId,
        to: AccountId,
        tokens_out: Balance,
    ) -> Balance {
        assert!(tokens_out > 0, "E2");
        let p1 = self.get_pool(&from);
        let p2 = self.get_pool(&to);
        let (_, tokens_in) = self._price_swap_tokens_out(&p1, &p2, tokens_out);
        return tokens_in;
    }

    pub fn add_liquidity_transfer_callback(&mut self, token: AccountId) {
        env::log(format!("enter add_liquidity_transfer_callback").as_bytes());

        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Can be called only as a callback"
        );

        assert_eq!(
            env::promise_results_count(),
            1,
            "Contract expected a result on the callback"
        );
        let action_succeeded = match env::promise_result(0) {
            PromiseResult::Successful(_) => true,
            _ => false,
        };

        //simulation do not allows for promises inside callbacks
        //for now just log result

        env::log(format!("PromiseResult  transfer succeeded:{}", action_succeeded).as_bytes());

        if !action_succeeded {
            env::log(
                format!(
                    "from add_liquidity_transfer_callback, token:{} transfer FAILED!",
                    token
                )
                .as_bytes(),
            );
            panic!("callback");
            //TO-DO ROLLBACK add_liquidity
        }

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

        pub fn set_vmc_with_token_op_deposit(&mut self) {
            let storage_price_per_byte: Balance = 100000000000000000000;
            self.set_vmc_deposit(storage_price_per_byte * 670); // arbitrary number easy to recoginze)
        }

        pub fn set_vmc_deposit(&mut self, attached_deposit: Balance) {
            self.vm.attached_deposit = attached_deposit;
            testing_env!(self.vm.clone());
        }

        // pub fn accounts_c(self) -> Accounts {
        //     return self.accounts;
        // }
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
                    near_bal: 0,
                    token_bal: 0,
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

        let near_deposit = 3000u128;
        let token_deposit = 500u128;
        ctx.set_vmc_with_token_op_deposit();
        token1.inc_allowance(t.clone(), token_deposit.into());

        ctx.set_vmc_deposit(near_deposit);
        let max_token_deposit = token_deposit;
        let min_shares_required = near_deposit;
        c.add_liquidity(t.clone(), max_token_deposit, min_shares_required);

        let p = c.pool_info(&t).expect("Pool should exist");
        assert_eq!(p.near_bal, near_deposit, "Near balance should be correct");
        assert_eq!(
            p.token_bal, token_deposit,
            "Token balance should be correct"
        );
    }

    // TODO tests
    // + add liquidity with max_balance > allowance
}
