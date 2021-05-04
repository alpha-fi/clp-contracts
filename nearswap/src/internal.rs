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
        // in_a * out_bal / (in_bal + in_a)  and scaling for fee
        let in_net = u256::from(in_amount) * 997;
        let r: u256 = in_net * u256::from(out_bal) / (u256::from(in_bal) * 1000 + in_net);
        return r.as_u128();
    }

    /// Calculates amout of tokens a user must pay to buy `out_amount` tokens, when a total
    /// balance in the pool is `in_bal` and `out_bal` of paid tokens and buying tokens
    /// respectively.
    #[inline]
    pub(crate) fn calc_in_amount(&self, out_amount: u128, in_bal: u128, out_bal: u128) -> u128 {
        // this is getOutputPrice in Uniswap
        // (in_bal * out_amount * 1000) / (out_bal - out_amount) / 997;
        let numerator = u256::from(in_bal) * u256::from(out_amount) * 1000;
        let r: u256 = numerator / u256::from(out_bal - out_amount) / 997;
        return r.as_u128() + 1;
    }

    pub(crate) fn _price_n2t_in(&self, token: &AccountId, ynear_in: u128) -> (Pool, u128) {
        assert!(ynear_in > 0, "E2: balance arguments must be >0");
        let p = self.get_pool(&token);
        let out = self.calc_out_amount(ynear_in, p.ynear, p.tokens).into();
        (p, out)
    }

    pub(crate) fn _price_n2t_out(&self, token: &AccountId, tokens_out: u128) -> (Pool, U128) {
        assert!(tokens_out > 0, "E2: balance arguments must be >0");
        let p = self.get_pool(&token);
        let in_amount = self.calc_in_amount(tokens_out, p.ynear, p.tokens).into();
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
        let p_in = self.get_pool(t_in);
        let p_out = self.get_pool(t_out);
        let near_swap = self.calc_out_amount(tokens_in, p_in.tokens, p_in.ynear);
        let tokens2_out = self.calc_out_amount(near_swap, p_out.ynear, p_out.tokens);
        println!(
            "Swapping_in {} {} -> {} ynear -> {} {}",
            tokens_in, t_in, near_swap, tokens2_out, t_out
        );
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
        let p_in = self.get_pool(&t_in);
        let p_out = self.get_pool(&t_out);
        let near_swap = self.calc_in_amount(tokens_out, p_out.ynear, p_out.tokens);
        let tokens1_to_pay = self.calc_in_amount(near_swap, p_in.tokens, p_in.ynear);
        println!(
            "Swapping_out {} {} -> {} ynear -> {} {}",
            tokens1_to_pay, t_in, near_swap, tokens_out, t_out
        );
        return (p_in, p_out, near_swap, tokens1_to_pay);
    }

    pub(crate) fn _swap_n2t(
        &mut self,
        p: &mut Pool,
        ynear_in: Balance,
        token: &AccountId,
        tokens_out: Balance,
    ) {
        println!(
            "User purchased {} {} for {} yNEAR",
            tokens_out, token, ynear_in
        );
        p.tokens -= tokens_out;
        p.ynear += ynear_in;

        let user = env::predecessor_account_id();
        let mut d = self.get_deposit(&user);
        d.remove_near(ynear_in);
        d.add(token, tokens_out);

        self.set_pool(token, p);
        self.set_deposit(&user, &d);
    }

    pub(crate) fn _swap_t2n(
        &mut self,
        p: &mut Pool,
        token: &AccountId,
        token_in: Balance,
        ynear_out: Balance,
    ) {
        let user = env::predecessor_account_id();
        println!(
            "User {} purchased {} NEAR tokens for {} tokens",
            user, ynear_out, token_in
        );
        p.tokens += token_in;
        p.ynear -= ynear_out;

        let mut d = self.get_deposit(&user);
        d.remove(token, token_in);
        d.ynear += ynear_out;

        self.set_pool(&token, p);
        self.set_deposit(&user, &d);
    }

    // TODO: use references rather than copying Pool
    pub(crate) fn _swap_tokens(
        &mut self,
        mut p1: Pool,
        mut p2: Pool,
        token1: &AccountId,
        token1_in: Balance,
        token2: &AccountId,
        token2_out: Balance,
        near_swap: Balance,
    ) {
        let user = env::predecessor_account_id();
        println!(
            "User purchased {} {} tokens for {} {} tokens",
            token2_out, token2, token1_in, token1,
        );
        p1.tokens += token1_in;
        p1.ynear -= near_swap;
        p2.tokens -= token2_out;
        p2.ynear += near_swap;

        let mut d = self.get_deposit(&user);
        d.remove(token1, token1_in);
        d.add(token2, token2_out);

        self.set_pool(&token1, &p1);
        self.set_pool(&token2, &p2);
        self.set_deposit(&user, &d);
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
