// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance};
use std::convert::{TryFrom,TryInto};

use std::fmt;
use crate::*;
use crate::constants::*;


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

    pub populated: usize,
    pub last_updated_index: usize,
    pub pivoted: bool,
    pub observation: Vec<Twap>,

    pub mean_1min: (U128, U128),
    pub mean_5min: (U128, U128),
    pub mean_1h: (U128, U128),
    pub mean_12h: (U128, U128)
}

impl Pool {
    pub fn new(pool_id: Vec<u8>) -> Self {
        Self {
            ynear: 0,
            reserve: 0,
            shares: LookupMap::new(pool_id),
            total_shares: 0,
            populated: 0,
            last_updated_index: 0,
            pivoted: false,
            observation: Vec::new(),
            mean_1min: (U128(0), U128(0)),
            mean_5min: (U128(0), U128(0)),
            mean_1h: (U128(0), U128(0)),
            mean_12h: (U128(0), U128(0))
        }
    }

    pub fn pool_info(&self) -> PoolInfo {
        PoolInfo {
            ynear: self.ynear.into(),
            reserve: self.reserve.into(),
            total_shares: self.total_shares.into(),
        }
    }

    pub fn log_observation(&mut self) {
        // price0, price1 calculate
        let near = u128::try_from(self.ynear).unwrap();
        let reserve = u128::try_from(self.reserve).unwrap();
        let price0: u128 = near / reserve;
        let price1: u128 = reserve / near;

        // update current index
        if self.populated == 0 {
            self.last_updated_index = Twap::initialize(&mut self.observation, env::block_timestamp(), price0, price1);
            self.populated += 1;
        } else {
            if self.last_updated_index + 1 >= MAX_LENGTH {
                self.pivoted = true;
            }
            self.last_updated_index = Twap::write(
                &mut self.observation, self.last_updated_index,
                env::block_timestamp(), price0, price1,
                MAX_LENGTH
            );
        }

        // update mean
        let mut len: usize;
        if self.pivoted == true {
            len = MAX_LENGTH.into();
        } else {
            len = self.observation.len();
        }

        self.mean_1min = Twap::calculate_mean(
            &self.observation, "1min", self.last_updated_index, len, self.pivoted
        );

        self.mean_5min = Twap::calculate_mean(
            &self.observation, "5min", self.last_updated_index, len, self.pivoted
        );

        self.mean_1h = Twap::calculate_mean(
            &self.observation, "1h", self.last_updated_index, len, self.pivoted
        );

        self.mean_12h = Twap::calculate_mean(
            &self.observation, "12h", self.last_updated_index, len, self.pivoted
        );
    }
}
