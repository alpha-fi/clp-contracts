use crate::*;

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
        // in_a * out_bal / (in_bal + in_a)  and scaling for fee
        let in_net = u256::from(in_amount) * 997;
        let r: u256 = in_net * u256::from(out_bal) / (u256::from(in_bal) * 1000 + in_net);
        return r.as_u128();
    }

    /// Calculates amout of tokens a user must pay to buy `out_amount` tokens, when a total
    /// balance in the pool is `in_bal` and `out_bal` of paid tokens and buying tokens
    /// respectively.
    pub(crate) fn calc_in_amount(&self, out_amount: u128, in_bal: u128, out_bal: u128) -> u128 {
        // this is getOutputPrice in Uniswap
        // (in_bal * out_amount * 1000) / (out_bal - out_amount) / 997;
        let numerator = u256::from(in_bal) * u256::from(out_amount) * 1000;
        let r: u256 = numerator / u256::from(out_bal - out_amount) / 997;
        return r.as_u128();
    }

    pub(crate) fn schedule_nep21_tansfer(
        &mut self,
        token: &AccountId,
        from_account: AccountId,
        to_account: AccountId,
        amount: u128,
    ) -> Promise {
        return Promise::new(token.clone()).function_call(
            "transfer_from".into(),
            format!(
                r#"{{
                        "owner_id": "{}",
                        "new_owner_id": "{}",
                        "amount": "{}"
                        }}"#,
                from_account, to_account, amount
            )
            .as_bytes()
            .to_vec(),
            util::NEP21_STORAGE_DEPOSIT, //refundable, required if the fun-contract needs more storage
            util::MAX_GAS / 3,
        );
        //TODO add rollback callback
        // .then(ext_self::add_liquidity_transfer_callback(
        //     env::current_account_id(),
        //     token,
        //     0,
        //     MAX_GAS/3,
        // ));
    }

    pub(crate) fn _swap_near(
        &mut self,
        p: &mut Pool,
        token: &AccountId,
        near: Balance,
        reserve: Balance,
        recipient: AccountId,
    ) {
        // TODO: remove this
        println!(
            "User purchased {} {} for {} YoctoNEAR",
            reserve, token, near
        );
        p.token_bal -= reserve;
        p.near_bal += near;
        self.set_pool(token, p);

        //send the token from CLP account to buyer
        self.schedule_nep21_tansfer(
            token,
            env::current_account_id(),
            env::predecessor_account_id(),
            reserve,
        );
        //TODO callbacks
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
        println!(
            "User purchased {} NEAR tokens for {} reserve tokens",
            near, reserve
        );
        p.token_bal += reserve;
        p.near_bal -= near;
        self.set_pool(&token, p);

        //get the token from buyer into CLP
        let promise = self.schedule_nep21_tansfer(token, buyer, env::current_account_id(), reserve);
        //and in the same batch send NEAR to client
        promise.transfer(near);
        //TODO COMPLEX ROLLBACKS
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
        assert!(tokens_to_pay <= max_tokens_paid, "E8"); //computed amount of selling tokens is bigger than user required maximum.
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
        env_log!(
            "User purchased {} {} tokens for {} {} tokens",
            token2_out, token2, token1_in, token1,
        );
        p1.token_bal += token1_in;
        p1.near_bal -= near_swap;
        p2.token_bal -= token2_out;
        p2.near_bal += near_swap;
        self.set_pool(&token1, p1);
        self.set_pool(&token2, p2);

        //get the token from buyer into CLP
        let promise1 = self.schedule_nep21_tansfer(
            token1,
            buyer.clone(),
            env::current_account_id(),
            token1_in,
        );
        //send the buyer the bougth token
        let promise2 = self.schedule_nep21_tansfer(
            token2,
            env::current_account_id(),
            buyer.clone(),
            token2_out,
        );
        //do both in parallel
        promise1.and(promise2);
        //TODO COMPLEX ROLLBACKS
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
        //env_log!("tokens1_to_pay {} max_tokens1_paid {}",yton(tokens1_to_pay),yton(max_tokens1_paid));
        assert!(tokens1_to_pay <= max_tokens1_paid, "E8"); //computed amount of selling tokens is bigger than user required maximum.

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
