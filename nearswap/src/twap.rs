use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, WrappedTimestamp};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance, Timestamp};
use std::convert::{TryFrom,TryInto};

use std::fmt;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Twap {
    // timestamp
    pub block_timestamp: WrappedTimestamp,
    // Number of observations till block_timestamp
    pub num_of_observations: U128,
    // cumulative price of token0 till block_timestamp
    pub price0cumulative: U128,
    // cumulative price of token1 till block_timestamp
    pub price1cumulative: U128,
    // price at block_timestamp(to know, open, close at time of querying)

    //pub mean_1min: (U128, U128),
    //pub mean_5min: (U128, U128),
   // pub mean_1h: (U128, U128),
    //pub mean_12h: (U128, U128),
    // move to PoolInfo
}

impl Twap {
    /// @notice Transforms a previous observation into a new observation, given the passage of time and the current tick and liquidity values
    /// @dev block_timestamp _must_ be chronologically equal to or greater than last.block_timestamp, safe for 0 or 1 overflows
    /// @param last The specified observation to be transformed
    /// @param block_timestamp The timestamp of the new observation
    /// @return Observation The newly populated observation
    pub fn transform(
        last: &Twap,
        block_timestamp: WrappedTimestamp,
        price0: u128,
        price1: u128
    ) -> Twap {
        let price0cumu = u128::try_from(last.price0cumulative).unwrap();
        let price1cumu = u128::try_from(last.price1cumulative).unwrap();
        return
            Twap {
                block_timestamp: block_timestamp,
                num_of_observations: U128(u128::try_from(last.num_of_observations).unwrap() + 1),
                price0cumulative: U128(price0cumu + price0),
                price1cumulative: U128(price1cumu + price1),
            };
    }

    /// @notice Initialize the oracle array by writing the first slot. Called once for the lifecycle of the observations array
    /// @param observation The stored oracle array
    /// @param time The time of the oracle initialization, via block.timestamp truncated to uint32
    /// @return length of TWAP array
    pub fn initialize(observation: &mut [Twap; 60000], time: WrappedTimestamp)
        -> U128
    {
        observation[0] = Twap {
            block_timestamp: time,
            num_of_observations: U128(0),
            price0cumulative: U128(0),
            price1cumulative: U128(0),
        };
        return U128(1);
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
        observation: &mut [Twap; 60000],
        index: usize,
        block_timestamp: WrappedTimestamp,
        price0: u128,
        price1: u128,
        max_length: usize
    ) -> usize {
        let last: &Twap = &observation[index];

        let updated_index: usize = (index + 1) % max_length;
        observation[updated_index] = Twap::transform(last, block_timestamp, price0, price1);
        
        return updated_index;
    }

    // Pivoted point binary search
    // Similar to rotated array from a certain pivot point
    // @param timestamp given timestamp
    // @returns index of first timestamp that is greater than or equal to given timestamp
    pub fn binary_search(
        observation: [Twap; 60000],
        last_updated_index: usize,
        max_length: usize,
        block_timestamp: WrappedTimestamp,
        pivoted: bool
    ) -> U128 {
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

            return U128(u128::try_from(start).unwrap());
        }

        return U128(u128::try_from(start).unwrap());
    }

    // function which will calculate mean using str "1min, 5min, 1h, 12h"
    //pub fn calculateMean(observation: [Twap; 60000], time: str, lastIndex: U128) -> U128 {
        
    //}
}