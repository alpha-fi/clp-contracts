// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance};

// use std::fmt;

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
}

impl Pool {
    pub fn new(pool_id: Vec<u8>) -> Self {
        Self {
            ynear: 0,
            tokens: 0,
            shares: LookupMap::new(pool_id),
            total_shares: 0,
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
        let added_tokens;
        // the very first deposit -- we define the constant ratio
        if self.total_shares == 0 {
            self.ynear = ynear;
            shares_minted = self.ynear;
            self.total_shares = shares_minted;
            added_tokens = max_tokens;
            self.tokens = added_tokens;
            self.shares.insert(caller, &shares_minted);
        } else {
            let ynear_256 = u256::from(ynear);
            let p_ynear_256 = u256::from(self.ynear); // ynear in pool
            added_tokens = (ynear_256 * u256::from(self.tokens) / p_ynear_256 + 1).as_u128();
            shares_minted = (ynear_256 * u256::from(self.total_shares) / p_ynear_256).as_u128();
            // TODO: adjust ynear to max_tokens instead of panicing
            assert!(
                max_tokens >= added_tokens,
                "E3: needs to transfer {} of tokens and it's bigger then specified  maximum={}",
                added_tokens,
                max_tokens
            );
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
            self.ynear += ynear;
            self.total_shares += shares_minted;
        }
        return (ynear, added_tokens, shares_minted);
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
    ) -> (u128, u128, u128) {
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

        return (shares, ynear, token_amount);
    }
}
