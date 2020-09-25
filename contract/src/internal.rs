use crate::*;
use util::*;

impl NearCLP {
    pub(crate) fn assert_owner(&self) {
        assert!(
            env::predecessor_account_id() == self.owner,
            "Only the owner can call this function"
        );
    }

    pub(crate) fn must_get_pool(&self, ref token: &AccountId) -> Pool {
        match self.pools.get(token) {
            None => env::panic(b"Pool for this token doesn't exist"),
            Some(p) => return p,
        }
    }

    pub(crate) fn set_pool(&mut self, ref token: &AccountId, pool: &Pool) {
        self.pools.insert(token, pool);
    }

    /// Calculates amout of tokens a user buys for `in_amount` tokens, when a total balance
    /// in the pool is `in_bal` and `out_bal` of paid tokens and buying tokens respectively.
    pub(crate) fn calc_out_amount(&self, in_amount: u128, in_bal: u128, out_bal: u128) -> u128 {
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
    pub(crate) fn calc_in_amount(&self, out_amount: u128, in_bal: u128, out_bal: u128) -> u128 {
        // this is getOutputPrice in Uniswap
        let numerator = U256::from(in_bal) * U256::from(out_amount) * U256::from(1000);
        let denominator = U256::from(out_bal - out_amount) * U256::from(997);
        let result = (numerator / denominator + 1).as_u128();
        return result;
    }

    pub(crate) fn _swap_near(
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

        //TO-DO change the callback
        nep21::ext_nep21::transfer(
            recipient,
            reserve.into(),
            token,
            NEP21_STORAGE_DEPOSIT,
            MAX_GAS/3,
        )
        .then(ext_self::add_liquidity_transfer_callback(
            env::current_account_id(),
            token,
            0,
            MAX_GAS/3,
        ));

        //let transfer_args =

        /*
        Promise::new(env::current_account_id())
        .function_call("transfer".as_bytes(), arguments: Vec<u8>, amount: Balance, gas: Gas)
        .call(nep21::ext_nep21::transfer(recipient, reserve.into(), token, 0, SINGLE_CALL_GAS)
        .then(
            ext_status_message::after_nep21_transfer(
                recipient,
                &account_id,
                0,
                CANT_FAIL_GAS,
            ),
            */
    }

    /// Pool sells reserve token for `near_paid` NEAR tokens. Asserts that a user buys at least
    /// `min_tokens` of reserve tokens.
    pub(crate) fn _swap_near_exact_in(
        &mut self,
        token: &AccountId,
        near_paid: Balance,
        min_tokens: Balance,
        recipient: AccountId,
    ) {
        assert!(near_paid > 0 && min_tokens > 0, "E2");
        let mut p = self.must_get_pool(&token);
        // env::log(format!(
        //         "self.calc_out_amount({},{},{})",near_paid, p.near_bal, p.token_bal
        //         ).as_bytes(),);
        let tokens_out = self.calc_out_amount(near_paid, p.near_bal, p.token_bal);
        assert!(tokens_out >= min_tokens, "E7");
        self._swap_near(&mut p, token, near_paid, tokens_out, recipient);
    }

    /// Pool sells `tokens_out` reserve token for NEAR tokens. Asserts that a user pays no more
    /// than `max_near_paid`.
    pub(crate) fn _swap_near_exact_out(
        &mut self,
        token: &AccountId,
        tokens_out: Balance,
        max_near_paid: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        assert!(tokens_out > 0 && max_near_paid > 0, "E2");
        let mut p = self.must_get_pool(&token);
        let near_to_pay = self.calc_in_amount(tokens_out, p.near_bal, p.token_bal);
        // panics if near_to_pay > max_near_paid
        let near_refund = max_near_paid - near_to_pay;
        if near_refund > 0 {
            Promise::new(buyer).transfer(near_refund as u128);
        }
        self._swap_near(&mut p, token, near_to_pay, tokens_out, recipient);
    }

    pub(crate) fn _swap_reserve(
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
                MAX_GAS/3,
            ));
    }

    /// Pool sells NEAR for `tokens_paid` reserve tokens. Asserts that a user buys at least
    /// `min_near`.
    pub(crate) fn _swap_reserve_exact_in(
        &mut self,
        token: &AccountId,
        tokens_paid: Balance,
        min_near: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        assert!(tokens_paid > 0 && min_near > 0, "E2");
        let mut p = self.must_get_pool(&token);
        let near_out = self.calc_out_amount(tokens_paid, p.token_bal, p.near_bal);
        assert!(near_out >= min_near, "E7");
        self._swap_reserve(&mut p, token, tokens_paid, near_out, buyer, recipient);
    }

    /// Pool sells `tokens_out` reserve tokens for NEAR tokens. Asserts that a user pays
    /// no more than `max_near_paid`.
    pub(crate) fn _swap_reserve_exact_out(
        &mut self,
        token: &AccountId,
        near_out: Balance,
        max_tokens_paid: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        assert!(near_out > 0 && max_tokens_paid > 0, "E2");
        let mut p = self.must_get_pool(&token);
        let tokens_to_pay = self.calc_in_amount(near_out, p.near_bal, p.token_bal);
        assert!(tokens_to_pay <= max_tokens_paid, "E8");
        self._swap_reserve(&mut p, token, tokens_to_pay, near_out, buyer, recipient);
    }

    pub(crate) fn _swap_tokens(
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
            MAX_GAS/3,
        )
        .and(nep21::ext_nep21::transfer(
            recipient,
            token2_out.into(),
            token2,
            0,
            MAX_GAS/3,
        ));
    }

    pub(crate) fn _price_swap_tokens_in(
        &self,
        p_in: &Pool,
        p_out: &Pool,
        tokens_in: Balance,
    ) -> (Balance, Balance) {
        let near_swap = self.calc_out_amount(tokens_in, p_in.token_bal, p_in.near_bal);
        let tokens2_out = self.calc_out_amount(near_swap, p_out.near_bal, p_out.token_bal);
        return (near_swap, tokens2_out);
    }

    pub(crate) fn _price_swap_tokens_out(
        &self,
        p_in: &Pool,
        p_out: &Pool,
        tokens_out: Balance,
    ) -> (Balance, Balance) {
        let near_swap = self.calc_in_amount(tokens_out, p_out.token_bal, p_out.near_bal);
        let tokens1_to_pay = self.calc_in_amount(near_swap, p_in.near_bal, p_in.token_bal);
        return (near_swap, tokens1_to_pay);
    }

    pub(crate) fn _swap_tokens_exact_in(
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
        let mut p1 = self.must_get_pool(&token1);
        let mut p2 = self.must_get_pool(&token2);
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

    pub(crate) fn _swap_tokens_exact_out(
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
        let mut p1 = self.must_get_pool(&token1);
        let mut p2 = self.must_get_pool(&token2);
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
