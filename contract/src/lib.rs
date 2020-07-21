// use borsh::{BorshDeserialize, BorshSerialize};
// use near_sdk::json_types::U128;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
//use std::collections::UnorderedMap;

// a way to optimize memory management
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod nep21;
mod util;

// Prepaid gas for making a single simple call.
const SINGLE_CALL_GAS: u64 = 200_000_000_000_000;

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

// Pool structure
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    near_bal: Balance,
    token_bal: Balance,
    shares: UnorderedMap<AccountId, Balance>,
    /// total amount of participation shares. Shares are represented using the same amount of
    /// tailing decimals as the NEAR token, which is 24
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
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct NearCLP {
    pub fee_dst: AccountId,
    pub owner: AccountId,
    // we are using unordered map because it allows to iterate over the pools
    pools: UnorderedMap<AccountId, Pool>,
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
        if let Some(_) = self.pools.get(&token) {
            env::panic(b"E1");
        }
        self.pools
            .insert(&token, &Pool::new(token.as_bytes().to_vec()));
    }

    // Increases Near and the Reserve token liquidity.
    // The supplied funds must preserver current ratio of the liquidity pool.
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
            env::log(b"Creating a frist deposit");
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
        self.set_pool(&token, &p);
        nep21::ext_nep21::transfer_from(
            caller,
            env::current_account_id(),
            token_amount.into(),
            &token,
            0,
            SINGLE_CALL_GAS,
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
                SINGLE_CALL_GAS,
            ));
    }

    pub fn shares_balance_of(&self, token: AccountId, owner: AccountId) -> Balance {
        return self.get_pool(&token).shares.get(&owner).unwrap_or(0);
    }

    /**********************
    CLP market functions
    **********************/

    #[payable]
    pub fn swap_near_to_reserve_exact_in(&mut self, token: AccountId, min_tokens: Balance) {
        self._swap_near_exact_in(
            &token,
            env::attached_deposit(),
            min_tokens,
            env::predecessor_account_id(),
        );
    }

    /// swaps NEAR tokens to reserve tokens and transfers reserve tokens to a given recipient.
    #[payable]
    pub fn swap_near_to_reserve_exact_in_xfr(
        &mut self,
        token: AccountId,
        min_tokens: Balance,
        recipient: AccountId,
    ) {
        self._swap_near_exact_in(&token, env::attached_deposit(), min_tokens, recipient);
    }

    #[payable]
    pub fn swap_near_to_reserve_exact_out(&mut self, token: AccountId, tokens_out: Balance) {
        let b = env::predecessor_account_id();
        self._swap_near_exact_out(&token, tokens_out, env::attached_deposit(), b.clone(), b);
    }

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

    pub fn swap_reserve_to_near_exact_out(
        &mut self,
        token: AccountId,
        near_out: Balance,
        max_tokens: Balance,
    ) {
        let b = env::predecessor_account_id();
        self._swap_reserve_exact_out(&token, near_out, max_tokens, b.clone(), b);
    }

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

    pub fn swap_tokens_exact_out(
        &mut self,
        from: AccountId,
        to: AccountId,
        tokens_from: Balance,
        max_tokens_to: Balance,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_out(&from, &to, tokens_from, max_tokens_to, b.clone(), b);
    }

    pub fn swap_tokens_exact_out_xfr(
        &mut self,
        from: AccountId,
        to: AccountId,
        tokens_from: Balance,
        max_tokens_to: Balance,
        recipient: AccountId,
    ) {
        let b = env::predecessor_account_id();
        self._swap_tokens_exact_out(&from, &to, tokens_from, max_tokens_to, b, recipient);
    }

    /// Calculates the amount of tokens user will recieve when swapping `near_paid` for `token`
    /// assets
    pub fn price_near_to_token_in(&self, token: AccountId, near_paid: Balance) -> Balance {
        assert!(near_paid > 0, "E2");
        let p = self.get_pool(&token);
        return self.calc_out_amount(near_paid, p.near_bal, p.token_bal);
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
    fn calc_out_amount(&self, in_amount: Balance, in_bal: Balance, out_bal: Balance) -> Balance {
        // this is getInputPrice in Uniswap
        let in_net = in_amount * 997;
        return in_net * out_bal / (in_bal * 1000 + in_net);
    }

    /// Calculates amout of tokens a user must pay to buy `out_amount` tokens, when a total
    /// balance in the pool is `in_bal` and `out_bal` of paid tokens and buying tokens
    /// respectively.
    fn calc_in_amount(&self, out_amount: Balance, in_bal: Balance, out_bal: Balance) -> Balance {
        // this is getOutputPrice in Uniswap
        return (in_bal * out_amount * 1000) / (out_bal - out_amount) / 997;
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
                "User purchased {} reserve tokens for {} NEAR",
                reserve, near
            )
            .as_bytes(),
        );
        p.token_bal -= reserve;
        p.near_bal += near;
        self.set_pool(token, p);
        nep21::ext_nep21::transfer(recipient, reserve.into(), token, 0, SINGLE_CALL_GAS);
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
            .transfer(near as u128)
            .and(nep21::ext_nep21::transfer_from(
                buyer,
                env::current_account_id(),
                reserve.into(),
                token,
                0,
                SINGLE_CALL_GAS,
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
            SINGLE_CALL_GAS,
        )
        .and(nep21::ext_nep21::transfer(
            recipient,
            token2_out.into(),
            token2,
            0,
            SINGLE_CALL_GAS,
        ));
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
        let near_swap = self.calc_out_amount(tokens1_paid, p1.token_bal, p1.near_bal);
        let tokens2_out = self.calc_out_amount(near_swap, p2.near_bal, p2.token_bal);
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
        let near_swap = self.calc_in_amount(tokens2_out, p2.token_bal, p2.near_bal);
        let tokens1_to_pay = self.calc_in_amount(near_swap, p1.near_bal, p1.token_bal);
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
mod tests {
    /*
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
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
        }
    }

    #[test]
    fn set_get_message() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Welcome::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(
            "howdy".to_string(),
            contract.get_greeting("bob_near".to_string())
        );
    }

    #[test]
    fn get_nonexistent_message() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = Welcome::default();
        assert_eq!(
            "Hello".to_string(),
            contract.get_greeting("francis.near".to_string())
        );
    }
    */
}
