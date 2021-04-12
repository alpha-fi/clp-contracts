// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance};

use std::fmt;
use crate::*;

/// PoolInfo is a helper structure to extract public data from a Pool
#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct PoolInfo {
    /// balance in yoctoNEAR
    pub ynear: U128,
    pub reserve: U128,
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
            self.ynear.0, self.reserve.0, self.total_shares.0
        );
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    pub ynear: Balance,
    pub reserve: Balance,
    pub shares: LookupMap<AccountId, Balance>,
    /// check `PoolInfo.total_shares`
    pub total_shares: Balance,

    pub twap: Twap,
}

impl Pool {
    pub fn new(pool_id: Vec<u8>) -> Self {
        Self {
            ynear: 0,
            reserve: 0,
            shares: LookupMap::new(pool_id),
            total_shares: 0,
            twap: Twap::new(),
        }
    }

    pub fn pool_info(&self) -> PoolInfo {
        PoolInfo {
            ynear: self.ynear.into(),
            reserve: self.reserve.into(),
            total_shares: self.total_shares.into(),
        }
    }
}
