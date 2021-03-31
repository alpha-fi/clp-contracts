use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance, Timestamp};
use std::convert::{TryFrom,TryInto};

use std::fmt;
use crate::constants::*;

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Copy)]
pub struct Twap {
    // timestamp
    pub block_timestamp: U64,
    // Number of observations till block_timestamp
    pub num_of_observations: U128,
    // cumulative price of token0 till block_timestamp
    pub price0cumulative: U128,
    // cumulative price of token1 till block_timestamp
    pub price1cumulative: U128,
}

impl Twap {
    /// @notice Transforms a previous observation into a new observation, given the passage of time and the current tick and liquidity values
    /// @dev block_timestamp _must_ be chronologically equal to or greater than last.block_timestamp, safe for 0 or 1 overflows
    /// @param last The specified observation to be transformed
    /// @param block_timestamp The timestamp of the new observation
    /// @param price0 price of first token
    /// @param price1 price of second token
    /// @return Observation The newly populated observation
    pub fn transform(
        last: &Twap,
        block_timestamp: u64,
        price0: u128,
        price1: u128
    ) -> Twap {
        let price0cumu = u128::try_from(last.price0cumulative).unwrap();
        let price1cumu = u128::try_from(last.price1cumulative).unwrap();
        return
            Twap {
                block_timestamp: U64(block_timestamp),
                num_of_observations: U128(u128::try_from(last.num_of_observations).unwrap() + 1),
                price0cumulative: U128(price0cumu + price0),
                price1cumulative: U128(price1cumu + price1),
            };
    }

    /// @return instance of Twap structure
    pub fn new() -> Self {
        return Self {
            block_timestamp: U64(1),
            num_of_observations: U128(0),
            price0cumulative: U128(0),
            price1cumulative: U128(0),
        }
    }
    /// @notice Initialize the oracle array by writing the first slot. Called once for the lifecycle of the observations array
    /// @param observation The stored oracle array
    /// @param time The time of the oracle initialization
    /// @return last updated index
    pub fn initialize(
        observation: &mut Vec<Twap>, time: u64,
        price0: u128, price1: u128
    ) -> usize
    {
        observation.push( Twap {
            block_timestamp: U64(time),
            num_of_observations: U128(1),
            price0cumulative: U128(price0),
            price1cumulative: U128(price1),
        });
        return 0;
    }

    /// @notice Writes an oracle observation to the array
    /// @dev Index represents the most recently written element.
    /// max_length and index must be tracked externally.
    /// @param observation The stored TWAP array
    /// @param index The location of the most recently updated observation
    /// @param block_timestamp The timestamp of the new observation
    /// @param max_length The length of TWAP array
    /// @return updated index
    pub fn write(
        observation: &mut Vec<Twap>,
        index: usize,
        block_timestamp: u64,
        price0: u128,
        price1: u128,
        max_length: usize
    ) -> usize {
        let last: &Twap = &observation[index];

        let updated_index: usize = (index + 1) % max_length;
        if index + 1 >= max_length {
            observation[updated_index] = Twap::transform(last, block_timestamp, price0, price1);
        } else {
            observation.push(Twap::transform(last, block_timestamp, price0, price1));
        }

        return updated_index;
    }

    // Pivoted point binary search
    // Similar to rotated array from a certain pivot point
    // @param timestamp given timestamp
    // @returns index of first timestamp that is greater than or equal to given timestamp
    pub fn binary_search(
        observation: &Vec<Twap>,
        last_updated_index: usize,
        max_length: usize,
        block_timestamp: u64,
        pivoted: bool
    ) -> usize {
        let mut start: usize = 0;
        let mut end: usize = last_updated_index + 1;

        let mut mid: usize;

        while start < end {
            mid = (start + end) / 2;
            if u64::try_from(block_timestamp).unwrap() <= u64::try_from(observation[mid].block_timestamp).unwrap() {
                end = mid;
            } else {
                start = mid + 1;
            }
        }

        if(pivoted && start == 0) {
            let res = start;
            start = last_updated_index + 1;
            end = max_length;

            while start < end {
                mid = (start + end) / 2;
                if u64::try_from(block_timestamp).unwrap() <= u64::try_from(observation[mid].block_timestamp).unwrap() {
                    end = mid;
                } else {
                    start = mid + 1;
                }
            }
            if start == max_length - 1 
                && u64::try_from(observation[start].block_timestamp).unwrap() < u64::try_from(block_timestamp).unwrap() {
                start = res;
            }

            return start;
        }

        return start;
    }

    // function which will calculate mean using str "1min, 5min, 1h, 12h"
    pub fn calculate_mean(
        observation: &Vec<Twap>, time: &str, last_index: usize,
        max_length: usize, pivoted: bool
    ) -> (U128, U128) {
        let timeDiff: u64;
        match time {
            "1min" => timeDiff = 6000_000_0000, // 1 minute in nanoseconds
            "5min" => timeDiff = 3000_0000_0000, // 5 minute in nanoseconds
            "1h" => timeDiff = 3600_000_000_000,
            "12h" => timeDiff = 43200_000_000_000,
            _ => timeDiff = 0
        }
        let req_timestamp = u64::try_from(observation[last_index].block_timestamp).unwrap() - timeDiff;

        let left_index = Twap::binary_search(observation, last_index, max_length, req_timestamp, pivoted);

        //let cmp = u64::try_from(observation[left_index].block_timestamp).unwrap();
        if left_index == 0 {
            let total_observe: u128 = u128::try_from(observation[last_index].num_of_observations).unwrap();
            let price0cumu: u128 = u128::try_from(observation[last_index].price0cumulative).unwrap();
            let price1cumu: u128 = u128::try_from(observation[last_index].price1cumulative).unwrap();
            let mean0: u128 = price0cumu / total_observe;
            let mean1: u128 = price1cumu / total_observe;
            return (U128(mean0), U128(mean1));
        } else {
            let total_observe: u128 = u128::try_from(observation[last_index].num_of_observations).unwrap()
                                    - u128::try_from(observation[left_index - 1].num_of_observations).unwrap();
            let price0cumu: u128 = u128::try_from(observation[last_index].price0cumulative).unwrap()
                                    - u128::try_from(observation[left_index - 1].price0cumulative).unwrap();
            let price1cumu: u128 = u128::try_from(observation[last_index].price1cumulative).unwrap()
                                    - u128::try_from(observation[left_index - 1].price1cumulative).unwrap();
            let mean0: u128 = price0cumu / total_observe;
            let mean1: u128 = price1cumu / total_observe;
            return (U128(mean0), U128(mean1));
        }
    }
}