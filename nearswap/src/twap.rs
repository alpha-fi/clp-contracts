use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance, Timestamp};
use std::convert::{TryFrom,TryInto};

use std::fmt;
use crate::*;
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
    /// @notice Transforms a previous observation into a new observation.
    /// @dev block_timestamp _must_ be chronologically equal to or greater than last.block_timestamp, safe for 0 or 1 overflows
    /// @param last The specified observation to be transformed
    /// @param block_timestamp The timestamp of the new observation
    /// @param price0 price of first token
    /// @param price1 price of second token
    /// @return Observation The newly populated observation
    pub fn transform(
        last: Twap,
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
        let last: Twap = observation[index];

        if(block_timestamp == u64::try_from(last.block_timestamp).unwrap()) {
            observation[index] = Twap::transform(last, block_timestamp, price0, price1);
            return index;
        }

        let updated_index: usize = (index + 1) % max_length;
        if updated_index < observation.len() {
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

        // edge case when all values are less than required
        if u64::try_from(observation[last_updated_index].block_timestamp).unwrap() 
            < u64::try_from(block_timestamp).unwrap() {
                panic!("Observation after this timestamp doesn't exist");
        }

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

    // function which calculates mean using str "1min, 5min, 1h, 12h"
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

#[cfg(test)]
mod tests {
    use super::Twap;
    use super::*;

    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, BlockHeight, MockedBlockchain};

    fn init_blockchain() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
    }

    // returns observation vector with timestamp [1, 2, 3, 4, 5, 6, 7, 9, 10]
    fn get_observations() -> (Vec<Twap>, usize) {
        let mut observation: Vec<Twap> = Vec::new();
        let mut last_updated_index = Twap::initialize(&mut observation, 1, 1, 1);

        let max_length = 10;
        // fill all places
        for i in 2..11 {
            let timestamp = i;
            last_updated_index = Twap::write(&mut observation,
                last_updated_index,
                timestamp,
                1, 1,
                max_length
            );
        }

        return (observation, last_updated_index);
    }

    #[test]
    fn initialize_works() {
        init_blockchain();

        let mut observation: Vec<Twap> = Vec::new();
        let last_updated_index = Twap::initialize(&mut observation, env::block_timestamp(), 1, 1);
        
        assert!(observation.len() == 1, "Mismatch");

        assert!(observation[0].price0cumulative == U128(1), "Mismatch");
        assert!(observation[0].price1cumulative == U128(1), "Mismatch");
    }

    #[test]
    fn write_works() {
        init_blockchain();

        let mut observation: Vec<Twap> = Vec::new();
        let mut last_updated_index = Twap::initialize(&mut observation, env::block_timestamp(), 1, 1);
        let max_length = 10;

        let timestamp = env::block_timestamp() + 12;
        last_updated_index = Twap::write(&mut observation,
            last_updated_index,
            timestamp,
            100, 2,
            max_length
        );

        assert!(observation.len() == 2, "Mismatch");

        assert!(observation[1].price0cumulative == U128(101), "Mismatch");
        assert!(observation[1].price1cumulative == U128(3), "Mismatch");

        // write on same timestamp
        last_updated_index = Twap::write(&mut observation,
            last_updated_index,
            timestamp,
            10, 10,
            max_length
        );

        // verify number of observations is 3 but observation length should be 2
        assert!(observation.len() == 2, "Mismatch");

        assert!(observation[0].num_of_observations == U128(1));
        assert!(observation[1].num_of_observations == U128(3));

        // verify cumulative prices
        assert!(observation[1].price0cumulative == U128(111), "Mismatch");
        assert!(observation[1].price1cumulative == U128(13), "Mismatch");
    }

    #[test]
    fn overwrite_works() {
        init_blockchain();

        let mut observation: Vec<Twap> = Vec::new();
        let mut last_updated_index = Twap::initialize(&mut observation, env::block_timestamp(), 1, 1);

        let max_length = 10;
        // fill all places
        for i in 1..10 {
            let timestamp = env::block_timestamp() + i;
            last_updated_index = Twap::write(&mut observation,
                last_updated_index,
                timestamp,
                1, 1,
                max_length
            );
        }

        assert!(observation.len() == 10, "Mismatch");

        // next observation should be written on 0th Index
        let mut last_timestamp = env::block_timestamp() + 10;
        last_updated_index = Twap::write(&mut observation,
            last_updated_index,
            last_timestamp,
            1, 1,
            max_length
        );

        assert!(observation.len() == 10, "Mismatch");
        assert!(observation[0].block_timestamp == U64(last_timestamp), "Mismatch");
        assert!(observation[0].num_of_observations == U128(11));

        // next observation should be written on 1st Index
        last_timestamp = env::block_timestamp() + 11;
        last_updated_index = Twap::write(&mut observation,
            last_updated_index,
            last_timestamp,
            1, 1,
            max_length
        );

        env_log!("as {}", observation.len());
        assert!(observation.len() == 10, "Mismatch");
        assert!(last_updated_index == 1, "current index mismatch");

        assert!(observation[1].block_timestamp == U64(last_timestamp), "Mismatch");
        assert!(observation[1].num_of_observations == U128(12));
    }

    #[test]
    fn simple_binary_search_works() {
        init_blockchain();

        let (observation, last_updated_index) = get_observations();
        let max_length = 10;

        // current observation timestamp array [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        let mut returned_index = Twap::binary_search(
            &observation,
            last_updated_index,
            max_length,
            5,
            false
        );

        assert!(returned_index == 4, "Wrong Index");

        returned_index = Twap::binary_search(
            &observation,
            last_updated_index,
            max_length,
            0,
            false
        );

        assert!(returned_index == 0, "Wrong Index");

        returned_index = Twap::binary_search(
            &observation,
            last_updated_index,
            max_length,
            10,
            false
        );

        assert!(returned_index == 9, "Wrong Index");
    }

    #[test]
    #[should_panic(expected = "Observation after this timestamp doesn't exist")]
    fn binary_edge_case_works() {
        init_blockchain();

        let (observation, last_updated_index) = get_observations();
        let max_length = 10;

        // current observation timestamp array [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        Twap::binary_search(
            &observation,
            last_updated_index,
            max_length,
            122,
            false
        );
    }

    #[test]
    fn pivoted_binary_search_works() {
        init_blockchain();

        let (mut observation, mut last_updated_index) = get_observations();
        let max_length = 10;

        // current array [1, 2, 3, 4, 5, 6, 8, 9, 10]
        // add more value (that should overwrite last updated value)

        last_updated_index = Twap::write(
            &mut observation,
            last_updated_index,
            13,
            10, 10,
            max_length
        );
        last_updated_index = Twap::write(
            &mut observation,
            last_updated_index,
            20,
            10, 10,
            max_length
        );
        last_updated_index = Twap::write(
            &mut observation,
            last_updated_index,
            21,
            10, 10,
            max_length
        );
        // Updated array [13, 20, 21, 4, 5, 6, 7, 8, 9, 10]

        let mut result_index = Twap::binary_search(
            &observation,
            last_updated_index,
            max_length,
            3,
            true
        );

        assert!(result_index == 3, "Wrong Index");

        result_index = Twap::binary_search(
            &observation,
            last_updated_index,
            max_length,
            15,
            true
        );

        assert!(result_index == 1, "Wrong Index");

        result_index = Twap::binary_search(
            &observation,
            last_updated_index,
            max_length,
            21,
            true
        );

        assert!(result_index == 2, "Wrong Index");

        result_index = Twap::binary_search(
            &observation,
            last_updated_index,
            max_length,
            10,
            true
        );

        assert!(result_index == 9, "Wrong Index");
    }
}