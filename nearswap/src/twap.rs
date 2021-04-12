use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::collections::{Vector};
use near_sdk::{env};
use std::convert::{TryFrom,TryInto};

use std::fmt;
use crate::constants::*;
use crate::pool::*;

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Twap {
    // populated is 
    populated: usize,
    last_updated_index: usize,
    pivoted: bool,
    pub observations: Vec<Observation>,

    pub mean_1min: (U128, U128),
    pub mean_5min: (U128, U128),
    pub mean_1h: (U128, U128),
    pub mean_12h: (U128, U128)
}

impl Twap {

    pub fn new() -> Self {
        Self {
            populated: 0,
            last_updated_index: 0,
            pivoted: false,
            observations: Vec::new(),
            mean_1min: (U128(0), U128(0)),
            mean_5min: (U128(0), U128(0)),
            mean_1h: (U128(0), U128(0)),
            mean_12h: (U128(0), U128(0))
        }
    }

    /**
    Initialize the oracle array by writing the first slot.
    Called once for the lifecycle of the observations array.
    Parameters:
    + `time`: The time of the oracle initialization..
    + `price1`:  price of first token
    + `price2`: price of second token.
    */
    pub fn initialize(
        &mut self,
        time: u64,
        price1: u128,
        price2: u128
    ) -> usize
    {
        self.observations.push( Observation {
            block_timestamp: U64(time),
            num_of_observations: U128(1),
            price1_cumulative: U128(price1),
            price2_cumulative: U128(price2),
        });
        return 0;
    }

    /**
    Writes an oracle observation to the array.
    Index represents the most recently written element.
    Parameters:
    + `block_timestamp`: The timestamp of the new observation.
    + `price1`: price of first token.
    + `price2`: price of second token.
    + `max_length`: The maximum length of TWAP array
    */
    pub fn write(
        &mut self,
        block_timestamp: u64,
        price1: u128,
        price2: u128,
        max_length: usize
    ) -> usize {
        let last: &Observation = &self.observations[self.last_updated_index].clone();

        let updated_index: usize = (self.last_updated_index + 1) % max_length;
        if self.last_updated_index + 1 >= max_length {
            self.observations[updated_index] = Observation::transform(last, block_timestamp, price1, price2);
        } else {
            self.observations.push(Observation::transform(last, block_timestamp, price1, price2));
        }

        return updated_index;
    }

    /**
    Pivoted point binary search: searches array which is
    sorted and rotated from a particular point.
    Similar to rotated array from a certain pivot point.
    Parameters:
    + `max_length`: The maximum length of TWAP array.
    + `timestamp`: given timestamp.
    */
    pub fn binary_search(
        &self,
        max_length: usize,
        block_timestamp: u64,
    ) -> usize {
        let mut start: usize = 0;
        let mut end: usize = self.last_updated_index + 1;

        let mut mid: usize;

        while start < end {
            mid = (start + end) / 2;
            if u64::try_from(block_timestamp).unwrap() <= u64::try_from(self.observations[mid].block_timestamp).unwrap() {
                end = mid;
            } else {
                start = mid + 1;
            }
        }

        if self.pivoted && start == 0 {
            let res = start;
            start = self.last_updated_index + 1;
            end = max_length;

            while start < end {
                mid = (start + end) / 2;
                if u64::try_from(block_timestamp).unwrap() <= u64::try_from(self.observations[mid].block_timestamp).unwrap() {
                    end = mid;
                } else {
                    start = mid + 1;
                }
            }
            if start == max_length - 1 
                && u64::try_from(self.observations[start].block_timestamp).unwrap() < u64::try_from(block_timestamp).unwrap() {
                start = res;
            }

            return start;
        }

        return start;
    }

    // function which will calculate mean using str "1min, 5min, 1h, 12h"
    pub fn calculate_mean(
        &self,
        time: &str,
        max_length: usize,
    ) -> (U128, U128) {
        let timeDiff: u64;
        match time {
            "1min" => timeDiff = 6000_000_0000, // 1 minute in nanoseconds
            "5min" => timeDiff = 3000_0000_0000, // 5 minute in nanoseconds
            "1h" => timeDiff = 3600_000_000_000,
            "12h" => timeDiff = 43200_000_000_000,
            _ => timeDiff = 0
        }
        let last_index = self.last_updated_index;
        let req_timestamp = u64::try_from(self.observations[last_index].block_timestamp).unwrap() - timeDiff;

        let left_index = self.binary_search(max_length, req_timestamp);

        if left_index == 0 {
            let total_observe: u128 = u128::try_from(self.observations[last_index].num_of_observations).unwrap();
            let price1cumu: u128 = u128::try_from(self.observations[last_index].price1_cumulative).unwrap();
            let price2cumu: u128 = u128::try_from(self.observations[last_index].price2_cumulative).unwrap();
            let mean1: u128 = price1cumu / total_observe;
            let mean2: u128 = price2cumu / total_observe;
            return (U128(mean1), U128(mean2));
        } else {
            let total_observe: u128 = u128::try_from(self.observations[last_index].num_of_observations).unwrap()
                                    - u128::try_from(self.observations[left_index - 1].num_of_observations).unwrap();
            let price1cumu: u128 = u128::try_from(self.observations[last_index].price1_cumulative).unwrap()
                                    - u128::try_from(self.observations[left_index - 1].price1_cumulative).unwrap();
            let price2cumu: u128 = u128::try_from(self.observations[last_index].price2_cumulative).unwrap()
                                    - u128::try_from(self.observations[left_index - 1].price2_cumulative).unwrap();
            let mean1: u128 = price1cumu / total_observe;
            let mean2: u128 = price2cumu / total_observe;
            return (U128(mean1), U128(mean2));
        }
    }

    pub(crate) fn log_observation(&mut self, pool: PoolInfo) {
        // price1, price2 calculate
        let near = u128::try_from(pool.ynear).unwrap();
        let reserve = u128::try_from(pool.reserve).unwrap();
        let price1: u128 = near / reserve;
        let price2: u128 = reserve / near;

        // update current index
        if self.populated == 0 {
            self.last_updated_index = self.initialize(env::block_timestamp(), price1, price2);
            self.populated += 1;
        } else {
            if self.last_updated_index + 1 >= MAX_LENGTH {
                self.pivoted = true;
            }
            self.last_updated_index = self.write(
                env::block_timestamp(),
                price1,
                price2,
                MAX_LENGTH
            );
        }

        // update mean
        let len: usize;
        if self.pivoted == true {
            len = MAX_LENGTH.into();
        } else {
            len = self.observations.len();
        }

        self.mean_1min = self.calculate_mean("1min", len);

        self.mean_5min = self.calculate_mean("5min", len);

        self.mean_1h = self.calculate_mean("1h", len);

        self.mean_12h = self.calculate_mean("12h", len);
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Copy)]
pub struct Observation {
    // timestamp
    pub block_timestamp: U64,
    // Number of observations till block_timestamp
    pub num_of_observations: U128,
    // cumulative price of token1 till block_timestamp
    pub price1_cumulative: U128,
    // cumulative price of token2 till block_timestamp
    pub price2_cumulative: U128,
}

impl Observation {
    /**
    Transforms a previous observation into a new observation.
    Parameters:
    + `block_timestamp`: _must_ be chronologically equal to or greater than last.block_timestamp.
    + `last`: The specified observation to be transformed.
    + `price1`: price of first token.
    + `price2`: price of second token.
    */
    pub fn transform(
        last: &Observation,
        block_timestamp: u64,
        price1: u128,
        price2: u128
    ) -> Observation {
        let price1cumu = u128::try_from(last.price1_cumulative).unwrap();
        let price2cumu = u128::try_from(last.price2_cumulative).unwrap();
        return
            Observation {
                block_timestamp: U64(block_timestamp),
                num_of_observations: U128(u128::try_from(last.num_of_observations).unwrap() + 1),
                price1_cumulative: U128(price1cumu + price1),
                price2_cumulative: U128(price2cumu + price2),
            };
    }

    /// returns instance of Observation structure
    pub fn new() -> Self {
        return Self {
            block_timestamp: U64(1),
            num_of_observations: U128(0),
            price1_cumulative: U128(0),
            price2_cumulative: U128(0),
        }
    }
}