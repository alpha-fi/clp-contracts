// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use crate::*;

impl NearSwap {
    #[inline]
    pub(crate) fn assert_owner(&self) {
        assert!(
            env::predecessor_account_id() == self.owner,
            "E22: Only owner can call this function"
        );
    }

    #[inline]
    pub(crate) fn get_pool(&self, ref token: &AccountId) -> Pool {
        self.pools
            .get(token)
            .expect("Pool for this token doesn't exist")
    }

    #[inline]
    pub(crate) fn set_pool(&mut self, ref token: &AccountId, pool: &Pool) {
        self.pools.insert(token, pool);
    }

    /// Calculates amout of tokens a user buys for `in_amount` tokens, when a total balance
    /// in the pool is `in_bal` and `out_bal` of paid tokens and buying tokens respectively.
    #[inline]
    pub(crate) fn calc_out_amount(&self, in_amount: u128, in_bal: u128, out_bal: u128) -> u128 {
        // formula: y = (x * Y * X) / (x + X)^2
        let x = u256::from(in_amount);
        let X = u256::from(in_bal);
        let numerator = ( x * u256::from(in_bal) * X);
        let mut denominator = (x + X);
        denominator *= denominator;

        let r = numerator / denominator;
        return r.as_u128();
    }

    /// returns swap out amount and fee.
    pub(crate) fn calc_out_with_fee(&self, mut x: u128,  X: u128, Y: u128) -> (u128, u128) {
        if x == 0 {
            return (0, 0);
        }
        let fee = x*1000/3; // 0.3% x
        x = x - fee;
        (self.calc_out_amount(x, X,  Y), fee)
    }

    pub(crate) fn _price_n2t_in(&self, token: &AccountId, ynear_in: u128) -> (Pool, u128) {
        assert!(ynear_in > 0, "E2: balance arguments must be >0");
        let p = self.get_pool(&token);
        let out = self.calc_out_amount(ynear_in, p.ynear, p.tokens).into();
        (p, out)
    }

    pub(crate) fn _price_swap_tokens_in(
        &self,
        t_in: &AccountId,
        t_out: &AccountId,
        tokens_in: Balance,
    ) -> Balance {
        assert!(tokens_in > 0, "E2: balance arguments must be >0");
        assert_ne!(t_in, t_out, "E9: can't swap same tokens");
        let p_in = self.get_pool(t_in);
        let p_out = self.get_pool(t_out);
        let near_swap = self.calc_out_amount(tokens_in, p_in.tokens, p_in.ynear);
        let tokens2_out = self.calc_out_amount(near_swap, p_out.ynear, p_out.tokens);
        println!(
            "Swapping_in {} {} -> {} ynear -> {} {}",
            tokens_in, t_in, near_swap, tokens2_out, t_out
        );
        return tokens2_out;
    }

    /// Should be at least `min_tokens_out` or swap will fail
    /// (prevents front running and other slippage issues).
    pub(crate) fn _swap_n2t(
        &mut self,
        p: &mut Pool,
        ynear_in: Balance,
        token: &AccountId,
        min_tokens_out: Balance,
    ) -> Balance {
        let in_bal = p.ynear;
        let out_bal = p.tokens;
        let in_amount = ynear_in;

        let (out_amount, fee) = self.calc_out_with_fee(in_amount, in_bal, out_bal);
        assert!(out_amount >= min_tokens_out, ERR25_MIN_AMOUNT);
        println!(
            "User purchased {} {} for {} yNEAR",
            out_amount, token, ynear_in
        );
        
        p.tokens -= out_amount;
        p.ynear += ynear_in;

        let user = env::predecessor_account_id();
        let mut d = self.get_deposit(&user);
        d.remove_near(ynear_in);
        d.add(token, out_amount);

        self.set_pool(token, p);
        self.set_deposit(&user, &d);
        out_amount
    }

    // Should be at least `min_ynear_out` or swap will fail
    // (prevents front running and other slippage issues).
    pub(crate) fn _swap_t2n(
        &mut self,
        p: &mut Pool,
        token: &AccountId,
        token_in: Balance,
        min_ynear_out: Balance,
    ) -> Balance {
        let user = env::predecessor_account_id();

        let in_bal = p.tokens;
        let out_bal = p.ynear;
        let in_amount = token_in;

        let (out_amount, fee) = self.calc_out_with_fee(in_amount, in_bal, out_bal);
        assert!(out_amount >= min_ynear_out, ERR25_MIN_AMOUNT);
        println!(
            "User {} purchased {} NEAR tokens for {} tokens",
            user, out_amount, token_in
        );

        p.tokens += in_amount;
        p.ynear -= out_amount;

        let mut d = self.get_deposit(&user);
        d.remove(token, in_amount);
        d.ynear += out_amount;

        self.set_pool(&token, p);
        self.set_deposit(&user, &d);
        out_amount
    }

    // Should be at least min_amount_out or swap will fail
    // (prevents front running and other slippage issues).
    pub(crate) fn _swap_tokens(
        &mut self,
        p1: &mut Pool,
        p2: &mut Pool,
        token1: &AccountId,
        token1_in: Balance,
        token2: &AccountId,
        min_token2_out: Balance,
    ) -> Balance {
        let user = env::predecessor_account_id();
        let (swap_amount, _) = self.calc_out_with_fee(token1_in, p1.tokens, p1.ynear);
        let (out, _) = self.calc_out_with_fee(swap_amount, p2.ynear, p2.tokens);

        assert!(out >= min_token2_out, ERR25_MIN_AMOUNT);
        println!(
            "User purchased {} {} tokens for {} {} tokens",
            out, token2, token1_in, token1,
        );

        p1.tokens += token1_in;
        p1.ynear -= swap_amount;
        p2.tokens -= out;
        p2.ynear += swap_amount;

        let mut d = self.get_deposit(&user);
        d.remove(token1, token1_in);
        d.add(token2, out);

        self.set_pool(&token1, p1);
        self.set_pool(&token2, p2);
        self.set_deposit(&user, &d);
        out
    }

    /// Helper function for LP shares transfer implementing NEP-MFT standard.
    pub(crate) fn _transfer(
        &mut self,
        token: String,
        recipient: AccountId,
        amount: U128,
        msg: String,
        _memo: String,
        is_contract: bool,
    ) -> bool {
        // TODO: add storage checks
        // let tx_start_storage = env::storage_usage();

        let sender = env::predecessor_account_id();
        util::assert_account_is_valid(&recipient);
        let amount_u = u128::from(amount);
        assert!(amount_u > 0, "E2: amount must be >0");
        let mut p = self.get_pool(&token);
        let shares = p.shares.get(&sender).unwrap_or(0);
        assert!(shares >= amount_u, ERR11_NOT_ENOUGH_SHARES);
        p.shares.insert(&sender, &(shares - amount_u));
        p.shares.insert(
            &recipient,
            &(p.shares.get(&recipient).unwrap_or(0) + amount_u),
        );

        if is_contract {
            // TODO: We should do it before modifiying local state to avoid exploits.
            ext_mft_rec::on_mft_receive(
                token.clone(),
                sender,
                amount,
                msg,
                &recipient,
                0,
                env::prepaid_gas() / 4,
            );
        }
        self.set_pool(&token, &p);
        return true;
    }
}

#[inline]
pub(crate) fn assert_max_pay(to_pay: u128, max: u128) {
    assert!(
        to_pay <= max,
        format!(
            "E8: selling {} tokens is bigger than required maximum",
            to_pay
        )
    );
}

#[inline]
pub(crate) fn assert_min_buy(to_buy: u128, min: u128) {
    assert!(
        to_buy >= min,
        format!(
            "E7: buying {} tokens is smaller than required minimum",
            to_buy
        )
    );
}
