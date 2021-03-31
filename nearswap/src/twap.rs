use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, WrappedTimestamp};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance, Timestamp};

use std::fmt;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Twap {
    // timestamp
    pub blockTimestamp: WrappedTimestamp;
    // Number of observations till blockTimestamp
    pub numObservations: U128;
    // cumulative price of token0 till blockTimestamp
    pub price0cumulative: U128;
    // cumulative price of token1 till blockTimestamp
    pub price1cumulative: U128;
    // price at blockTimestamp(to know, open, close at time of querying)
    pub pivoted: bool;

    pub mean_1min: (U128, U128);
    pub mean_5min: (U128, U128);
    pub mean_1h: (U128, U128);
    pub mean_12h: (U128, U128);
    // move to PoolInfo
}

impl Twap {
    /// @notice Transforms a previous observation into a new observation, given the passage of time and the current tick and liquidity values
    /// @dev blockTimestamp _must_ be chronologically equal to or greater than last.blockTimestamp, safe for 0 or 1 overflows
    /// @param last The specified observation to be transformed
    /// @param blockTimestamp The timestamp of the new observation
    /// @return Observation The newly populated observation
    pub fn transform(
        last: Twap,
        blockTimestamp: WrappedTimestamp,
        price0: U128,
        price1: U128
    ) -> Twap {
        return
            Twap {
                blockTimestamp: blockTimestamp,
                numObservations: last.numObservations + 1,
                price0cumulative: last.price0cumulative + price0,
                price1cumulative: last.price1cumulative + price1,
            };
    }

    /// @notice Initialize the oracle array by writing the first slot. Called once for the lifecycle of the observations array
    /// @param observation The stored oracle array
    /// @param time The time of the oracle initialization, via block.timestamp truncated to uint32
    /// @return length of TWAP array
    pub fn initialize(observation: [Twap; 60000], time: WrappedTimestamp)
        -> U128
    {
        observation[0] = Twap {
            blockTimestamp: time,
            numObservations: 0,
            price0cumulative: 0,
            price1cumulative: 0,
            pivoted: false,
            // move
            mean_1min: (0, 0),
            mean_5min: (0, 0),
            mean_1h: (0, 0),
            mean_12h: (0, 0)
        };
        return U128(1);
    }

    /// @notice Writes an oracle observation to the array
    /// @dev Index represents the most recently written element.
    /// maxLength and index must be tracked externally.
    /// @param observation The stored TWAP array
    /// @param index The location of the most recently updated observation
    /// @param blockTimestamp The timestamp of the new observation
    /// @param maxLength The length of TWAP array
    /// @return updated index
    pub fn write(
        observation: [Twap; 60000],
        index: U128,
        blockTimestamp: WrappedTimestamp,
        price0: U128,
        price1: U128,
        maxLength: U128
    ) -> U128 {
        let last: Twap = observation[index];

        if index >= maxLength {
            pivoted = true;
        }

        let indexUpdated = (index + 1) % maxLength;
        observation[indexUpdated] = transform(last, blockTimestamp, price0, price1);
        return U128(indexUpdated);
    }

    // Pivoted point binary search
    // Similar to rotated array from a certain pivot point
    // @param timestamp given timestamp
    // @returns index of first timestamp that is greater than or equal to given timestamp
    pub fn binarySearch(
        observation: [TWAP; 60000],
        lastUpdatedIndex: U128,
        maxLength: U128,
        blockTimestamp: WrappedTimestamp
    ) -> U128 {
        let mut start = 0;
        let mut end = lastUpdatedIndex + 1;

        let mut mid = (start + end) / 2;

        while start < end {
            mid = (start + end) / 2;
            if blockTimestamp <= observation[mid].blockTimestamp {
                end = mid;
            } else {
                start = mid + 1;
            }
        }

        if(pivoted && start == 0) {
            let res = start;
            start = lastUpdatedIndex + 1;
            end = maxLength;

            let mut mid = (start + end) / 2;

            while start < end {
                mid = (start + end) / 2;
                if blockTimestamp <= observation[mid].blockTimestamp {
                    end = mid;
                } else {
                    start = mid + 1;
                }
            }

            if start == maxLength - 1 && observation[start].blockTimestamp < blockTimestamp {
                start = res;
            }

            return start;
        }

        return start;
    }

    // function which will calculate mean using str "1min, 5min, 1h, 12h"
    pub fn calculateMean(observation: [Twap; 60000], time: str, lastIndex: U128) -> U128 {
        
    }
}