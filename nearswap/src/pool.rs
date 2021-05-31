// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance};

// use std::fmt;

use crate::twap::*;
use crate::*;

#[cfg(test)]
use std::fmt;

/// PoolInfo is a helper structure to extract public data from a Pool
#[cfg(not(test))]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct PoolInfo {
    /// balance in yNEAR
    pub ynear: U128,
    pub tokens: U128,
    /// total amount of participation shares. Shares are represented using the same amount of
    /// tailing decimals as the NEAR token, which is 24
    pub total_shares: U128,
}

#[cfg(test)]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct PoolInfo {
    pub ynear: U128,
    pub tokens: U128,
    pub total_shares: U128,
}

#[cfg(test)]
impl fmt::Display for PoolInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            f,
            "({}, {}, {})",
            self.ynear.0, self.tokens.0, self.total_shares.0
        );
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    pub ynear: Balance,
    pub tokens: Balance,
    pub shares: LookupMap<AccountId, Balance>,
    /// check `PoolInfo.total_shares`
    pub total_shares: Balance,

    pub twap: Twap,
}

impl Pool {
    pub fn new(pool_id: Vec<u8>) -> Self {
        Self {
            ynear: 0,
            tokens: 0,
            shares: LookupMap::new(pool_id),
            total_shares: 0,
            twap: Twap::new(65535),
        }
    }

    pub fn pool_info(&self) -> PoolInfo {
        PoolInfo {
            ynear: self.ynear.into(),
            tokens: self.tokens.into(),
            total_shares: self.total_shares.into(),
        }
    }

    /**
    Rebalances the pool by assigning new liquidity. It doesn't perform any transfer.
    Liquidiyt must come from the contract deposits.
    Arguments:
     * `caller`: liquidity provider
     * `ynear`: amount of yNEAR to be added to the pool. Will be adjusted by the `max_tokens`
        constraint.
     * `max_tokens`: max amount of tokens to be added to the pool
    Returns: (ynear added, tokens added, shares minted). */
    pub(crate) fn add_liquidity(
        &mut self,
        caller: &AccountId,
        ynear: u128,
        max_tokens: u128,
        min_shares: u128,
    ) -> (u128, u128, u128) {
        let shares_minted;
        let mut added_tokens;
        let added_near;
        // the very first deposit -- we define the constant ratio
        if self.total_shares == 0 {
            self.ynear = ynear;
            shares_minted = self.ynear;
            self.total_shares = shares_minted;
            added_tokens = max_tokens;
            added_near = ynear;
            self.tokens = added_tokens;
            self.shares.insert(caller, &shares_minted);
        } else {
            let ynear_256 = u256::from(ynear);
            let p_ynear_256 = u256::from(self.ynear); // ynear in pool
            added_tokens = (ynear_256 * u256::from(self.tokens) / p_ynear_256 + 1).as_u128();

            // Adjust near according to max_tokens
            if max_tokens < added_tokens {
                added_near = ((u256::from(max_tokens) * p_ynear_256) / u256::from(self.tokens) + 1)
                    .as_u128();
                added_tokens = max_tokens;
                shares_minted = (u256::from(added_near) * u256::from(self.total_shares)
                    / p_ynear_256)
                    .as_u128();
            } else {
                added_near = ynear;
                shares_minted = (ynear_256 * u256::from(self.total_shares) / p_ynear_256).as_u128();
            }
            assert!(
                u128::from(min_shares) <= shares_minted,
                "E4: amount minted shares ({}) is smaller then the required minimum",
                shares_minted
            );
            self.shares.insert(
                caller,
                &(self.shares.get(&caller).unwrap_or(0) + shares_minted),
            );
            self.tokens += added_tokens;
            self.ynear += added_near;
            self.total_shares += shares_minted;
        }
        return (added_near, added_tokens, shares_minted);
    }

    /// Withdraw `shares` for liquidity stored in this pool and transfer them to the caller deposit account. User can require
    /// getting at least `min_ynear` of Near and `min_tokens` of tokens. The function panic if the condition is not met.
    /// Shares are not exchangeable between different pools.
    pub(crate) fn withdraw_liquidity(
        &mut self,
        caller: &AccountId,
        min_ynear: u128,
        min_tokens: u128,
        shares: u128,
    ) -> (u128, u128) {
        let current_shares = self.shares.get(&caller).unwrap_or(0);
        let total_shares2 = u256::from(self.total_shares);
        let shares2 = u256::from(shares);
        let ynear = (shares2 * u256::from(self.ynear) / total_shares2).as_u128();
        let token_amount = (shares2 * u256::from(self.tokens) / total_shares2).as_u128();
        assert!(
            ynear >= min_ynear && token_amount >= min_tokens,
            "{}",
            format!(
                "E6: redeeming (ynear={}, tokens={}), which is smaller than the required minimum",
                ynear, token_amount
            )
        );

        self.shares.insert(caller, &(current_shares - shares));
        self.total_shares -= shares;
        self.tokens -= token_amount;
        self.ynear -= ynear;

        return (ynear, token_amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::test_utils::{VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain};

    fn init_blockchain() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
    }

    fn setup_pool() -> Pool {
        let token = "eth".to_string();
        return Pool::new(token.as_bytes().to_vec());
    }

    fn expected_added_liquidity(
        ynear: u128, max_tokens: u128, pool: &Pool
    ) -> (u128, u128, u128) {
        let ynear_256 = u256::from(ynear);
        let p_ynear_256 = u256::from(pool.ynear);
        let mut added_tokens = (ynear_256 * u256::from(pool.tokens) / p_ynear_256 + 1).as_u128();
        let shares_minted;
        let added_near;

        // Adjust near according to max_tokens
        if max_tokens < added_tokens {
            added_near = ((u256::from(max_tokens) * p_ynear_256) / u256::from(pool.tokens) + 1)
                .as_u128();
            added_tokens = max_tokens;
            shares_minted = (u256::from(added_near) * u256::from(pool.total_shares)
                / p_ynear_256)
                .as_u128();
        } else {
            added_near = ynear;
            shares_minted = (ynear_256 * u256::from(pool.total_shares) / p_ynear_256).as_u128();
        }
        return (added_near, added_tokens, shares_minted);
    }

    fn expected_withdraw(shares: u128, pool: &Pool) -> (u128, u128) {
        let total_shares2 = u256::from(pool.total_shares);
        let shares2 = u256::from(shares);
        let ynear = (shares2 * u256::from(pool.ynear) / total_shares2).as_u128();
        let token_amount = (shares2 * u256::from(pool.tokens) / total_shares2).as_u128();
        return(ynear, token_amount);
    }

    // Empty pool
    #[test]
    fn new_pool() {
        let pool: Pool = setup_pool();

        assert!(pool.ynear == 0, "Pool is not empty");
        assert!(pool.tokens == 0, "Pool is not empty");
        assert!(pool.total_shares == 0, "Pool is not empty");
    }

    // Existing pool
    #[test]
    fn add_liquidity_pool() {
        init_blockchain();

        let caller = "account".to_string();
        let mut pool: Pool = setup_pool();

        pool.add_liquidity(&caller, 100, 200, 0);

        assert!(pool.ynear == 100, "liquidity added is incorrect");
        assert!(pool.tokens == 200, "liquidity added is incorrect");
        assert!(pool.total_shares == 100, "liquidity added is incorrect");

        let (expected_near, expected_tokens, expected_shares) = expected_added_liquidity(200, 400, &pool);
        
        // add liquidity again
        let (near_added, tokens_added, shares_minted) = pool.add_liquidity(&caller, 200, 400, 0);

        assert!(near_added == expected_near, "liquidity added is incorrect");
        assert!(tokens_added == expected_tokens, "liquidity added is incorrect");
        assert!(shares_minted == expected_shares, "liquidity added is incorrect");

        let (expected_near2, expected_tokens2, expected_shares2) = expected_added_liquidity(100, 100, &pool);
        // add liquidity again with ratio 1:1(100:100)
        let (near_added2, tokens_added2, shares_minted2) = pool.add_liquidity(&caller, 100, 100, 0);

        assert!(near_added2 == expected_near2, "liquidity added is incorrect");
        assert!(tokens_added2 == expected_tokens2, "liquidity added is incorrect");
        assert!(shares_minted2 == expected_shares2, "liquidity added is incorrect");
    }

    #[test]
    fn withdraw_liquidity_pool() {
        init_blockchain();

        let caller = "account".to_string();
        let mut pool: Pool = setup_pool();

        pool.add_liquidity(&caller, 100, 200, 0);

        let (near_before, tokens_before, shares_before) = (pool.ynear, pool.tokens, pool.total_shares);
        let (expected_near, expected_tokens) = expected_withdraw(50, &pool);

        pool.withdraw_liquidity(&caller, 0, 0, 50);

        // withdraw shares
        assert!(pool.ynear == near_before - expected_near, "liquidity removed is incorrect");
        assert!(pool.tokens == tokens_before - expected_tokens, "liquidity removed is incorrect");
        assert!(pool.total_shares == shares_before - 50, "liquidity removed is incorrect");
    }
}