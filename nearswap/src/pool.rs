// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance};

// use std::fmt;

use crate::twap::*;
use crate::*;

#[cfg(test)]
use std::fmt;

/// PoolInfo is a helper structure to extract public data from a Pool
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
// TOOD: #[cfg_attr(feature = "test", derive(Debug, PartialEq))]
pub struct PoolInfo {
    /// balance in yNEAR
    pub ynear: U128,
    pub tokens: U128,
    /// total amount of participation shares. Shares are represented using the same amount of
    /// tailing decimals as the NEAR token, which is 24
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
pub enum Pool {
    V1(PoolV1),
}

impl Pool {
    fn unpack(self) -> PoolV1 {
        match self {
            Pool::V1(a) => a,
        }
    }
}

impl Into<Pool> for PoolV1 {
    fn into(self) -> Pool {
        Pool::V1(self)
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PoolV1 {
    pub ynear: Balance,
    pub tokens: Balance,
    pub shares: LookupMap<AccountId, Balance>,
    /// check `PoolInfo.total_shares`
    pub total_shares: Balance,

    pub twap: Twap,
}

impl PoolV1 {
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

    // TODO: Add unit tests: empty pool, existing pool
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
