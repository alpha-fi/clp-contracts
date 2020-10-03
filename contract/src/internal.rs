use crate::*;
use near_sdk::StorageUsage;

// use near_sdk::Gas;

impl NearCLP {
    #[inline]
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

    #[inline]
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
        return r.as_u128() + 1;
    }

    #[inline]
    pub(crate) fn schedule_nep21_tx(
        &mut self,
        token: &AccountId,
        from: AccountId,
        to: AccountId,
        amount: u128,
        // gas: Gas,
    ) -> Promise {
        let gas = env::prepaid_gas();
        assert!(gas >= 20 * TGAS, "Not enough gas");
        ext_nep21::transfer_from(
            from,
            to,
            amount.into(),
            token,
            util::NEP21_STORAGE_DEPOSIT,
            env::prepaid_gas() / 3,
        )
    }

    pub(crate) fn _price_near_to_token_in(
        &self,
        token: &AccountId,
        ynear_in: u128,
    ) -> (Pool, u128) {
        assert!(ynear_in > 0, "E2: balance arguments must be >0");
        let p = self.must_get_pool(&token);
        let out = self.calc_out_amount(ynear_in, p.ynear, p.reserve).into();
        (p, out)
    }

    pub(crate) fn _price_near_to_token_out(
        &self,
        token: &AccountId,
        tokens_out: u128,
    ) -> (Pool, U128) {
        assert!(tokens_out > 0, "E2: balance arguments must be >0");
        let p = self.must_get_pool(&token);
        let in_amount = self.calc_in_amount(tokens_out, p.ynear, p.reserve).into();
        (p, in_amount)
    }

    pub(crate) fn _price_swap_tokens_in(
        &self,
        t_in: &AccountId,
        t_out: &AccountId,
        tokens_in: Balance,
    ) -> (Pool, Pool, Balance, Balance) {
        assert!(tokens_in > 0, "E2: balance arguments must be >0");
        assert_ne!(t_in, t_out, "E9: can't swap same tokens");
        let p_in = self.must_get_pool(t_in);
        let p_out = self.must_get_pool(t_out);
        let near_swap = self.calc_out_amount(tokens_in, p_in.reserve, p_in.ynear);
        let tokens2_out = self.calc_out_amount(near_swap, p_out.ynear, p_out.reserve);
        return (p_in, p_out, near_swap, tokens2_out);
    }

    pub(crate) fn _price_swap_tokens_out(
        &self,
        t_in: &AccountId,
        t_out: &AccountId,
        tokens_out: Balance,
    ) -> (Pool, Pool, Balance, Balance) {
        assert!(tokens_out > 0, "E2: balance arguments must be >0");
        assert_ne!(t_in, t_out, "E9: can't swap same tokens");
        let p_in = self.must_get_pool(&t_in);
        let p_out = self.must_get_pool(&t_out);
        let near_swap = self.calc_in_amount(tokens_out, p_out.ynear, p_out.reserve);
        let tokens1_to_pay = self.calc_in_amount(near_swap, p_in.ynear, p_in.reserve);
        return (p_in, p_out, near_swap, tokens1_to_pay);
    }

    pub(crate) fn _swap_near(
        &mut self,
        p: &mut Pool,
        token: &AccountId,
        near: Balance,
        reserve: Balance,
        recipient: AccountId,
    ) {
        println!(
            "User purchased {} {} for {} yoctoNEAR",
            reserve, token, near
        );
        p.reserve -= reserve;
        p.ynear += near;

        self.schedule_nep21_tx(token, env::current_account_id(), recipient, reserve);
        // TODO: this updated should be done after nep21 transfer
        self.set_pool(token, p);
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
        assert!(min_tokens > 0, "E2: balance arguments must be >0");
        let (mut p, tokens_out) = self._price_near_to_token_in(token, near_paid);
        assert_min_buy(tokens_out, min_tokens);
        self._swap_near(&mut p, &token, near_paid, tokens_out, recipient);
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
        assert!(
            tokens_out > 0 && max_near_paid > 0,
            "E2: balance arguments must be >0"
        );
        let mut p = self.must_get_pool(&token);
        let near_to_pay = self.calc_in_amount(tokens_out, p.ynear, p.reserve);

        if max_near_paid < near_to_pay {
            env::panic(
                format!(
                    "E12: you need {} yNEAR to get {} {}",
                    near_to_pay, tokens_out, &token
                )
                .as_bytes(),
            );
        } else if max_near_paid > near_to_pay {
            Promise::new(buyer).transfer(max_near_paid - near_to_pay);
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
            "User {} purchased {} NEAR tokens for {} reserve tokens to {}",
            buyer, near, reserve, recipient
        );
        p.reserve += reserve;
        p.ynear -= near;

        // firstly get tokens from the buyer, then send NEAR to recipient
        self.schedule_nep21_tx(token, buyer, env::current_account_id(), reserve)
            .then(Promise::new(recipient).transfer(near));
        // TODO - this should be in a promise.
        self.set_pool(&token, p);
    }

    /// Pool sells NEAR for `tokens_paid` reserve tokens. Asserts that a user buys at least
    /// `min_near`.
    pub(crate) fn _swap_token_exact_in(
        &mut self,
        token: &AccountId,
        tokens_paid: Balance,
        min_near: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        assert!(
            tokens_paid > 0 && min_near > 0,
            "E2: balance arguments must be >0"
        );
        let mut p = self.must_get_pool(&token);
        let near_out = self.calc_out_amount(tokens_paid, p.reserve, p.ynear);
        assert_min_buy(near_out, min_near);
        self._swap_reserve(&mut p, token, tokens_paid, near_out, buyer, recipient);
    }

    /// Pool sells `tokens_out` reserve tokens for NEAR tokens. Asserts that a user pays
    /// no more than `max_near_paid`.
    pub(crate) fn _swap_token_exact_out(
        &mut self,
        token: &AccountId,
        near_out: Balance,
        max_tokens_paid: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        assert!(
            near_out > 0 && max_tokens_paid > 0,
            "E2: balance arguments must be >0"
        );
        let mut p = self.must_get_pool(&token);
        let tokens_to_pay = self.calc_in_amount(near_out, p.ynear, p.reserve);
        assert_max_pay(tokens_to_pay, max_tokens_paid);
        self._swap_reserve(&mut p, token, tokens_to_pay, near_out, buyer, recipient);
    }

    pub(crate) fn _swap_tokens(
        &mut self,
        mut p1: Pool,
        mut p2: Pool,
        token1: &AccountId,
        token2: &AccountId,
        token1_in: Balance,
        token2_out: Balance,
        near_swap: Balance,
        buyer: AccountId,
        recipient: AccountId,
    ) {
        println!(
            "User purchased {} {} tokens for {} {} tokens",
            token2_out, token2, token1_in, token1,
        );
        p1.reserve += token1_in;
        p1.ynear -= near_swap;
        p2.reserve -= token2_out;
        p2.ynear += near_swap;

        let caller = env::current_account_id();
        // firstly get tokens from buyer, then send to to the recipient
        self.schedule_nep21_tx(token1, buyer, caller.clone(), token1_in)
            .then(self.schedule_nep21_tx(token2, caller, recipient, token2_out));
        // TODO: make updates after nep21 transfers (together with promise2)
        self.set_pool(&token1, &p1);
        self.set_pool(&token2, &p2);
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
        assert!(min_tokens2 > 0, "E2: balance arguments must be >0");
        let (p1, p2, near_swap, tokens2_out) =
            self._price_swap_tokens_in(token1, token2, tokens1_paid);
        assert_min_buy(tokens2_out, min_tokens2);
        self._swap_tokens(
            p1,
            p2,
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
        assert!(max_tokens1_paid > 0, "E2: balance arguments must be >0");
        let (p1, p2, near_swap, tokens1_to_pay) =
            self._price_swap_tokens_out(token1, token2, tokens2_out);
        env_log!("Buying price is {}", tokens1_to_pay);

        assert_max_pay(tokens1_to_pay, max_tokens1_paid);

        self._swap_tokens(
            p1,
            p2,
            token1,
            token2,
            tokens1_to_pay,
            tokens2_out,
            near_swap,
            buyer,
            recipient,
        )
    }

    /// Helper function of a transfer implementing NEP-MFT standard.
    pub(crate) fn _transfer(
        &mut self,
        token: String,
        recipient: AccountId,
        amount: U128,
        data: Data,
        is_contract: bool,
    ) -> bool {
        let sender = env::predecessor_account_id();
        util::assert_account_is_valid(&recipient);
        let amount_u = u128::from(amount);
        assert!(amount_u > 0, "E2: amount must be >0");
        let mut p = self.must_get_pool(&token);
        let shares = p.shares.get(&sender).unwrap_or(0);
        assert!(
            shares >= amount_u,
            "E11: Insufficient amount of shares balance"
        );
        println!(
            ">>>>> Transferring shares. Sender {}, shares: {}, amount: {}",
            sender, shares, amount.0,
        );
        let initial_storage = env::storage_usage();
        p.shares.insert(&sender, &(shares - amount_u));
        p.shares.insert(
            &recipient,
            &(p.shares.get(&recipient).unwrap_or(0) + amount_u),
        );

        self.refund_storage(initial_storage);
        if is_contract {
            // TODO: We should do it before modifiying local state to avoid exploits.
            ext_mft_rec::on_mft_receive(
                token.clone(),
                sender,
                amount,
                data,
                &recipient,
                0,
                env::prepaid_gas() / 4,
            );
        }
        self.set_pool(&token, &p);
        return true;
    }

    pub(crate) fn refund_storage(&self, initial_storage: StorageUsage) {
        let current_storage = env::storage_usage();
        let attached_deposit = env::attached_deposit();
        let refund_amount = if current_storage > initial_storage {
            let required_deposit =
                Balance::from(current_storage - initial_storage) * STORAGE_BYTE_PRICE;
            assert!(
                required_deposit <= attached_deposit,
                "The required attached deposit is {}, but the given attached deposit is is {}",
                required_deposit,
                attached_deposit,
            );
            attached_deposit - required_deposit
        } else {
            attached_deposit + Balance::from(initial_storage - current_storage) * STORAGE_BYTE_PRICE
        };
        if refund_amount > 0 {
            env::log(format!("Refunding {} tokens for storage", refund_amount).as_bytes());
            Promise::new(env::predecessor_account_id()).transfer(refund_amount);
        }
    }
}

#[inline]
fn assert_max_pay(to_pay: u128, max: u128) {
    assert!(
        to_pay <= max,
        format!(
            "E8: selling {} tokens is bigger than required maximum",
            to_pay
        )
    );
}

#[inline]
fn assert_min_buy(to_buy: u128, min: u128) {
    assert!(
        to_buy >= min,
        format!(
            "E7: buying {} tokens is smaller than required minimum",
            to_buy
        )
    );
}
