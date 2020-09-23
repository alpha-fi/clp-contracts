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

mod nep21;
mod util;

// Prepaid gas costs. TODO: we need to adjust this value properly.
const MAX_GAS: u64 = 200_000_000_000_000; // 100T gas
const TX_NEP21_GAS: u64 = 20_000_000_000_000; // 20T gas

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

fn yton(near_amount: Balance) -> Balance {
    return near_amount / 10u128.pow(24);
}

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

/// Interface for the contract itself.
#[ext_contract(ext_self)]
pub trait SelfContract {
    /// A callback to check the result of the nep21-trasnfer
    fn nep21_transfer_callback(&mut self);
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

    pub fn create_pool(&mut self, token: AccountId) {
        assert!(
            self.pools
                .insert(&token, &Pool::new(token.as_bytes().to_vec()))
                .is_none(),
            "E1"
        );
    }

    /// Extracts public information about a `token` CLP.
    pub fn pool_info(&self, token: AccountId) -> Option<PoolInfo> {
        match self.pools.get(&token) {
            None => None,
            Some(p) => Some(p.pool_info()),
        }
    }

    // Increases Near and the Reserve token liquidity.
    // The supplied funds must preserver current ratio of the liquidity pool.
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
        let token_amount = max_token_amount;
        assert!(near_amount > 0 && max_token_amount > 0, "E2");

        // the very first deposit -- we define the constant ratio
        if p.total_shares == 0 {
            env::log(b"Creating a first deposit");
            p.near_bal = near_amount;
            shares_minted = p.near_bal;
            p.total_shares = shares_minted;

            p.token_bal = token_amount;
            p.shares.insert(&caller, &p.near_bal);
        } else {
            let token_amount = near_amount * p.token_bal / p.near_bal + 1;
            shares_minted = near_amount * p.total_shares / near_amount;
            assert!(max_token_amount >= token_amount, "E3");
            assert!(min_shares_amont <= shares_minted, "E4");

            p.shares.insert(
                &caller,
                &(p.shares.get(&caller).unwrap_or(0) + shares_minted),
            );
            p.token_bal += token_amount;
            p.near_bal += near_amount;
            p.total_shares += shares_minted;
        }

        env::log(
            format!(
                "Minting {} of shares for {} NEAR and {} reserve tokens",
                shares_minted, near_amount, token_amount
            )
            .as_bytes(),
        );
        println!(
            ">> in contract, attached deposit: {}, PoolInfo: {}",
            near_amount,
            p.pool_info()
        );

        self.set_pool(&token, &p);
        nep21::ext_nep21::transfer_from(
            caller,
            env::current_account_id(),
            token_amount.into(),
            &token,
            0,
            TX_NEP21_GAS,
        );
        // TODO:
        // Handling exception is work-in-progress in NEAR runtime
        // 1. rollback `p` on changes or move the pool update to a promise
        // 2. consider adding a lock to prevent other contracts calling and manipulate the prise before the token transfer will get finalized.
    }

    // Redeems `shares` for liquidity stored in this pool with condition of getting at least
    // `min_near` tokens and `min_tokens` of reserve. Shares are note exchengable between
    // different pools
    pub fn remove_liquidity(
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
                TX_NEP21_GAS,
            ));
    }

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
}

impl NearCLP {
    fn assert_owner(&self) {
        assert!(
            env::predecessor_account_id() == self.owner,
            "Only current owner can change owner"
        );
    }

    fn get_pool(&self, ref token: &AccountId) -> Pool {
        match self.pools.get(token) {
            None => env::panic(b"Pool for this token doesn't exist"),
            Some(p) => return p,
        }
    }

    fn set_pool(&mut self, ref token: &AccountId, pool: &Pool) {
        self.pools.insert(token, pool);
    }

    /// Calculates amout of tokens a user buys for `in_amount` tokens, when a total balance
    /// in the pool is `in_bal` and `out_bal` of paid tokens and buying tokens respectively.
    fn calc_out_amount(&self, in_amount: u128, in_bal: u128, out_bal: u128) -> u128 {
        // this is getInputPrice in Uniswap
        env::log(
            format!(
                "in_amount {} out_bal {} in_bal {}",
                yton(in_amount),
                yton(out_bal),
                yton(in_bal)
            )
            .as_bytes(),
        );
        let in_with_fee = U256::from(in_amount * 997);
        let numerator = in_with_fee * U256::from(out_bal);
        let denominator = U256::from(in_bal) * U256::from(1000) + in_with_fee;
        let result = (numerator / denominator).as_u128();
        env::log(format!("return {}", result).as_bytes());
        return result;
    }

    /// Calculates amout of tokens a user must pay to buy `out_amount` tokens, when a total
    /// balance in the pool is `in_bal` and `out_bal` of paid tokens and buying tokens
    /// respectively.
    fn calc_in_amount(&self, out_amount: u128, in_bal: u128, out_bal: u128) -> u128 {
        // this is getOutputPrice in Uniswap
        let numerator = U256::from(in_bal) * U256::from(out_amount) * U256::from(1000);
        let denominator = U256::from(out_bal - out_amount) * U256::from(997);
        let result = (numerator / denominator + 1).as_u128();
        return result;
    }

    fn _swap_near(
        &mut self,
        p: &mut Pool,
        token: &AccountId,
        near: Balance,
        reserve: Balance,
        recipient: AccountId,
    ) {
        env::log(
            format!(
                "User purchased {} {} for {} YoctoNEAR",
                reserve, token, near
            )
            .as_bytes(),
        );
        p.token_bal -= reserve;
        p.near_bal += near;
        self.set_pool(token, p);

        nep21::ext_nep21::transfer(recipient, reserve.into(), token, 0, TX_NEP21_GAS).then(
            ext_self::nep21_transfer_callback(&env::current_account_id(), 0, MAX_GAS),
        );

        //let transfer_args =

        /*
        Promise::new(env::current_account_id())
        .function_call("transfer".as_bytes(), arguments: Vec<u8>, amount: Balance, gas: Gas)
        .call(nep21::ext_nep21::transfer(recipient, reserve.into(), token, 0, MAX_GAS)
        .then(
            ext_status_message::nep21_transfer_callback(
                recipient,
                &account_id,
                0,
                CANT_FAIL_GAS,
            ),
            */
    }

    pub fn nep21_transfer_callback(&mut self) {
        env::log(format!("enter nep21_transfer_callback").as_bytes());

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

        env::log(format!("PromiseResult  trasnfer succeeded {}", action_succeeded).as_bytes());

        // If the stake action failed and the current locked amount is positive, then the contract has to unstake.
        /*if !stake_action_succeeded && env::account_locked_balance() > 0 {
            Promise::new(env::current_account_id()).stake(0, self.stake_public_key.clone());
        }
        */
    }

    /// Pool sells reserve token for `near_paid` NEAR tokens. Asserts that a user buys at least
    /// `min_tokens` of reserve tokens.
    fn _swap_near_exact_in(
        &mut self,
        token: &AccountId,
        near_paid: Balance,
        min_tokens: Balance,
        recipient: AccountId,
    ) {
        assert!(near_paid > 0 && min_tokens > 0, "E2");
        let mut p = self.get_pool(&token);
        // env::log(format!(
        //         "self.calc_out_amount({},{},{})",near_paid, p.near_bal, p.token_bal
        //         ).as_bytes(),);
        let tokens_out = self.calc_out_amount(near_paid, p.near_bal, p.token_bal);
        assert!(tokens_out >= min_tokens, "E7");
        self._swap_near(&mut p, token, near_paid, tokens_out, recipient);
    }

    /// Pool sells `tokens_out` reserve token for NEAR tokens. Asserts that a user pays no more
    /// than `max_near_paid`.
    fn _swap_near_exact_out(
        &mut self,
        token: &AccountId,
        tokens_out: Balance,
        max_near_paid: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        assert!(tokens_out > 0 && max_near_paid > 0, "E2");
        let mut p = self.get_pool(&token);
        let near_to_pay = self.calc_in_amount(tokens_out, p.near_bal, p.token_bal);
        // panics if near_to_pay > max_near_paid
        let near_refund = max_near_paid - near_to_pay;
        if near_refund > 0 {
            Promise::new(buyer).transfer(near_refund as u128);
        }
        self._swap_near(&mut p, token, near_to_pay, tokens_out, recipient);
    }

    fn _swap_reserve(
        &mut self,
        p: &mut Pool,
        token: &AccountId,
        near: Balance,
        reserve: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        env::log(
            format!(
                "User purchased {} NEAR tokens for {} reserve tokens",
                near, reserve
            )
            .as_bytes(),
        );
        p.token_bal += reserve;
        p.near_bal -= near;
        self.set_pool(&token, p);
        Promise::new(recipient)
            .transfer(near)
            .and(nep21::ext_nep21::transfer_from(
                buyer,
                env::current_account_id(),
                reserve.into(),
                token,
                0,
                TX_NEP21_GAS,
            ));
    }

    /// Pool sells NEAR for `tokens_paid` reserve tokens. Asserts that a user buys at least
    /// `min_near`.
    fn _swap_reserve_exact_in(
        &mut self,
        token: &AccountId,
        tokens_paid: Balance,
        min_near: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        assert!(tokens_paid > 0 && min_near > 0, "E2");
        let mut p = self.get_pool(&token);
        let near_out = self.calc_out_amount(tokens_paid, p.token_bal, p.near_bal);
        assert!(near_out >= min_near, "E7");
        self._swap_reserve(&mut p, token, tokens_paid, near_out, buyer, recipient);
    }

    /// Pool sells `tokens_out` reserve tokens for NEAR tokens. Asserts that a user pays
    /// no more than `max_near_paid`.
    fn _swap_reserve_exact_out(
        &mut self,
        token: &AccountId,
        near_out: Balance,
        max_tokens_paid: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        assert!(near_out > 0 && max_tokens_paid > 0, "E2");
        let mut p = self.get_pool(&token);
        let tokens_to_pay = self.calc_in_amount(near_out, p.near_bal, p.token_bal);
        assert!(tokens_to_pay <= max_tokens_paid, "E8");
        self._swap_reserve(&mut p, token, tokens_to_pay, near_out, buyer, recipient);
    }

    fn _swap_tokens(
        &mut self,
        p1: &mut Pool,
        p2: &mut Pool,
        token1: &AccountId,
        token2: &AccountId,
        token1_in: Balance,
        token2_out: Balance,
        near_swap: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        env::log(
            format!(
                "User purchased {} {} tokens for {} {} tokens",
                token2_out, token2, token1_in, token1
            )
            .as_bytes(),
        );
        p1.token_bal += token1_in;
        p1.near_bal -= near_swap;
        p2.token_bal -= token2_out;
        p2.near_bal += near_swap;
        self.set_pool(&token1, p1);
        self.set_pool(&token2, p1);
        nep21::ext_nep21::transfer_from(
            buyer,
            env::current_account_id(),
            token1_in.into(),
            token1,
            0,
            TX_NEP21_GAS,
        )
        .and(nep21::ext_nep21::transfer(
            recipient,
            token2_out.into(),
            token2,
            0,
            TX_NEP21_GAS,
        ));
    }

    fn _price_swap_tokens_in(
        &self,
        p_in: &Pool,
        p_out: &Pool,
        tokens_in: Balance,
    ) -> (Balance, Balance) {
        let near_swap = self.calc_out_amount(tokens_in, p_in.token_bal, p_in.near_bal);
        let tokens2_out = self.calc_out_amount(near_swap, p_out.near_bal, p_out.token_bal);
        return (near_swap, tokens2_out);
    }

    fn _price_swap_tokens_out(
        &self,
        p_in: &Pool,
        p_out: &Pool,
        tokens_out: Balance,
    ) -> (Balance, Balance) {
        let near_swap = self.calc_in_amount(tokens_out, p_out.token_bal, p_out.near_bal);
        let tokens1_to_pay = self.calc_in_amount(near_swap, p_in.near_bal, p_in.token_bal);
        return (near_swap, tokens1_to_pay);
    }

    fn _swap_tokens_exact_in(
        &mut self,
        token1: &AccountId,
        token2: &AccountId,
        tokens1_paid: Balance,
        min_tokens2: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        assert!(tokens1_paid > 0 && min_tokens2 > 0, "E2");
        assert_ne!(token1, token2, "E9");
        let mut p1 = self.get_pool(&token1);
        let mut p2 = self.get_pool(&token2);
        let (near_swap, tokens2_out) = self._price_swap_tokens_in(&p1, &p2, tokens1_paid);
        assert!(tokens2_out >= min_tokens2, "E7");

        self._swap_tokens(
            &mut p1,
            &mut p2,
            token1,
            token2,
            tokens1_paid,
            tokens2_out,
            near_swap,
            buyer,
            recipient,
        )
    }

    fn _swap_tokens_exact_out(
        &mut self,
        token1: &AccountId,
        token2: &AccountId,
        tokens2_out: Balance,
        max_tokens1_paid: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        assert!(tokens2_out > 0 && max_tokens1_paid > 0, "E2");
        assert_ne!(token1, token2, "E9");
        let mut p1 = self.get_pool(&token1);
        let mut p2 = self.get_pool(&token2);
        let (near_swap, tokens1_to_pay) = self._price_swap_tokens_out(&p1, &p2, tokens2_out);
        assert!(tokens1_to_pay >= max_tokens1_paid, "E8");

        self._swap_tokens(
            &mut p1,
            &mut p2,
            token1,
            token2,
            tokens1_to_pay,
            tokens2_out,
            near_swap,
            buyer,
            recipient,
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod token;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    use token::FungibleToken;

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
    #[should_panic(expected = "Only current owner can change owner")]
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
        c.create_pool(ctx.accounts.token1);
    }

    fn check_and_create_pool(c: &mut NearCLP, token: AccountId) {
        c.create_pool(token.clone());
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
        check_and_create_pool(&mut c, ctx.accounts.token1);
        check_and_create_pool(&mut c, ctx.accounts.token2);
    }

    #[test]
    fn add_liquidity_happy_path() {
        let (mut ctx, mut c) = init();
        let a = ctx.accounts.predecessor.clone();
        let t = ctx.accounts.token1.clone();
        let mut token1 = FungibleToken::new(a.clone(), ctx.token_supply.into());
        check_and_create_pool(&mut c, t.clone());
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

        let p = c.pool_info(t.clone()).expect("Pool should exist");
        assert_eq!(p.near_bal, near_deposit, "Near balance should be correct");
        assert_eq!(
            p.token_bal, token_deposit,
            "Token balance should be correct"
        );
    }

    // TODO tests
    // + add liquidity with max_balance > allowance
}
