use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::collections::{Vector};
use near_sdk::{env};
use std::convert::{TryFrom,TryInto};

use std::fmt;
use crate::constants::*;
use crate::pool::*;
use crate::*;
use crate::util::*;

#[derive(Debug)]
pub enum Mean {
    M_1MIN,
    M_5MIN,
    M_1H,
    M_12H,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Twap {
    // last updated index in observation array
    current_idx: usize,
    // bool to check if previous values are overwritten by new one
    // i.e MAX_LENGTH of array is full and we start storing observation from `0` index
    pivoted: bool,
    // observation array
    pub observations: Vec<Observation>,

    // mean of last 1 minutes of readings
    // In Tuple first value indicates mean of token1 and second indicates mean of token2
    pub mean_1min: (u128, u128),
    // mean of last 5 minutes of readings
    // In Tuple first value indicates mean of token1 and second indicates mean of token2
    pub mean_5min: (u128, u128),
    // mean of last 1 hour of readings
    // In Tuple first value indicates mean of token1 and second indicates mean of token2
    pub mean_1h: (u128, u128),
    // mean of last 12 hours of readings
    // In Tuple first value indicates mean of token1 and second indicates mean of token2
    pub mean_12h: (u128, u128)
}

impl Twap {

    pub fn new() -> Self {
        Self {
            populated: 0,
            current_idx: 0,
            pivoted: false,
            observations: Vec::new(),
            mean_1min: (0, 0),
            mean_5min: (0, 0),
            mean_1h: (0, 0),
            mean_12h: (0, 0)
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
            block_timestamp: time,
            num_of_observations: 1,
            price1_cumulative: price1,
            price2_cumulative: price2,
        });
        self.current_idx = 0;
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
        let mut o = &self.observations[self.current_idx];
        if(block_timestamp == o.block_timestamp) {
            self.observations[self.current_idx] = Observation::transform(o, block_timestamp, price1, price2);
            return self.current_idx;
        }

        if self.current_idx + 1 >= max_length {
            self.pivoted = true;
        }

        let updated_index: usize = (self.current_idx + 1) % max_length;
        if updated_index < self.observations.len() {
            self.observations[updated_index] = Observation::transform(o, block_timestamp, price1, price2);
        } else {
            self.observations.push(Observation::transform(o, block_timestamp, price1, price2));
        }

        self.current_idx = updated_index;
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

        // edge case when all values are less than required
        if self.observations[self.current_idx].block_timestamp
            < block_timestamp {
            panic!("Observation after this timestamp doesn't exist");
        }

        let mut start: usize = 0;
        let mut end: usize = self.current_idx + 1;

        let mut mid: usize;

        while start < end {
            mid = (start + end) / 2;
            if block_timestamp <= self.observations[mid].block_timestamp {
                end = mid;
            } else {
                start = mid + 1;
            }
        }

        if self.pivoted && start == 0 {
            let res = start;
            start = self.current_idx + 1;
            end = max_length;

            while start < end {
                mid = (start + end) / 2;
                if block_timestamp <= self.observations[mid].block_timestamp {
                    end = mid;
                } else {
                    start = mid + 1;
                }
            }
            if start >= max_length {
                start = res;
            }
        }

        return start;
    }

    // function which will calculate mean using Mean enum
    pub fn calculate_mean(
        &self,
        time: Mean,
        max_length: usize,
    ) -> (u128, u128) {
        let time_diff: u64 = match time {
            Mean::M_1MIN => to_nanoseconds(60), // 1 minute in nanoseconds
            Mean::M_5MIN => to_nanoseconds(300), // 5 minute in nanoseconds
            Mean::M_1H => to_nanoseconds(60 * 60),
            Mean::M_12H => to_nanoseconds(12 * 60 * 60),
            _ => 0
        };
        let last_index = self.current_idx;
        let req_timestamp = self.observations[last_index].block_timestamp - time_diff;
        let left_index = self.binary_search(max_length, req_timestamp);
        
        if left_index == 0 {
            let total_observe = self.observations[last_index].num_of_observations;
            let price1cumu = self.observations[last_index].price1_cumulative;
            let price2cumu = self.observations[last_index].price2_cumulative;
            let mean1 = price1cumu / total_observe;
            let mean2 = price2cumu / total_observe;
            return (mean1, mean2);
        } else {
            let total_observe = self.observations[last_index].num_of_observations
                                    - self.observations[left_index - 1].num_of_observations;
            let price1cumu = self.observations[last_index].price1_cumulative
                                    - self.observations[left_index - 1].price1_cumulative;
            let price2cumu = self.observations[last_index].price2_cumulative
                                    - self.observations[left_index - 1].price2_cumulative;
            let mean1 = price1cumu / total_observe;
            let mean2 = price2cumu / total_observe;
            return (mean1, mean2);
        }
    }

    pub(crate) fn log_observation(&mut self, pool: PoolInfo) {
        // price1, price2 calculate
        let near = u128::try_from(pool.ynear).unwrap();
        let reserve = u128::try_from(pool.reserve).unwrap();
        let price1: u128 = near / reserve;
        let price2: u128 = reserve / near;

        // update current index
        if self.observations.len == 0 {
            self.current_idx = self.initialize(env::block_timestamp(), price1, price2);
        } else {
            self.current_idx = self.write(
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

        self.update_mean(len);
    }

    pub fn update_mean(&mut self, len: usize) {
        self.mean_1min = self.calculate_mean(Mean::M_1MIN, len);

        self.mean_5min = self.calculate_mean(Mean::M_5MIN, len);

        self.mean_1h = self.calculate_mean(Mean::M_1H, len);

        self.mean_12h = self.calculate_mean(Mean::M_12H, len);
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Observation {
    // timestamp
    pub block_timestamp: u64,
    // Number of observations till block_timestamp
    pub num_of_observations: u128,
    // cumulative price of token1 till block_timestamp
    pub price1_cumulative: u128,
    // cumulative price of token2 till block_timestamp
    pub price2_cumulative: u128,
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
        return
            Observation {
                block_timestamp: block_timestamp,
                num_of_observations: last.num_of_observations + 1,
                price1_cumulative: last.price1_cumulative + price1,
                price2_cumulative: last.price2_cumulative + price2,
            };
    }

    /// returns instance of Observation structure
    pub fn new() -> Self {
        return Self {
            block_timestamp: 1,
            num_of_observations: 0,
            price1_cumulative: 0,
            price2_cumulative: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Twap;
    use super::Observation;
    use super::*;

    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, BlockHeight, MockedBlockchain};

    fn init_blockchain() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
    }

    // returns twap with observation vector with timestamp [1, 2, 3, 4, 5, 6, 7, 9, 10]
    fn get_twap() -> Twap {

        let mut twap: Twap = Twap::new();
        let mut current_idx = twap.initialize(env::block_timestamp(), 1, 1);

        let max_length = 10;
        // fill all places
        for i in 2..11 {
            let timestamp = i;
            current_idx = twap.write(
                timestamp,
                1, 1,
                max_length
            );
        }

        return twap;
    }

    #[test]
    fn initialize_works() {
        init_blockchain();

        let mut twap: Twap = Twap::new();
        let current_idx = twap.initialize(env::block_timestamp(), 1, 1);

        assert!(twap.observations.len() == 1, "Mismatch");

        assert!(twap.observations[0].price1_cumulative == 1, "Mismatch");
        assert!(twap.observations[0].price2_cumulative == 1, "Mismatch");
    }

    #[test]
    fn write_works() {
        init_blockchain();

        let mut twap: Twap = Twap::new();
        let mut current_idx = twap.initialize(env::block_timestamp(), 1, 1);
        let max_length = 10;

        let timestamp = env::block_timestamp() + 12;
        current_idx = twap.write(
            timestamp,
            100, 2,
            max_length
        );

        assert!(twap.observations.len() == 2, "Length Mismatch");

        assert!(twap.observations[1].num_of_observations == 2);

        assert!(twap.observations[1].price1_cumulative == 101, "price 1 Mismatch");
        assert!(twap.observations[1].price2_cumulative == 3, "price 2 Mismatch");

        // write on same timestamp
        current_idx = twap.write(
            timestamp,
            10, 10,
            max_length
        );

        // verify number of observations is 3 but observation length should be 2
        assert!(twap.observations.len() == 2, "length 2 Mismatch");

        assert!(twap.observations[0].num_of_observations == 1);
        assert!(twap.observations[1].num_of_observations == 3);

        // verify cumulative prices
        assert!(twap.observations[1].price1_cumulative == 111, "updated price 1 Mismatch");
        assert!(twap.observations[1].price2_cumulative == 13, "updated price 2 Mismatch");
    }

    #[test]
    fn overwrite_works() {
        init_blockchain();

        let mut twap: Twap = Twap::new();
        let mut current_idx = twap.initialize(env::block_timestamp(), 1, 1);

        let max_length = 10;
        // fill all places
        for i in 1..10 {
            let timestamp = env::block_timestamp() + i;
            current_idx = twap.write(
                timestamp,
                1, 1,
                max_length
            );
        }

        assert!(twap.observations.len() == 10, "Mismatch");

        // next observation should be written on 0th Index
        let mut last_timestamp = env::block_timestamp() + 10;
        current_idx = twap.write(
            last_timestamp,
            1, 1,
            max_length
        );

        assert!(twap.observations.len() == 10, "Mismatch");
        assert!(twap.observations[0].block_timestamp == last_timestamp, "Mismatch");
        assert!(twap.observations[0].num_of_observations == 11);

        // next observation should be written on 1st Index
        last_timestamp = env::block_timestamp() + 11;
        current_idx = twap.write(
            last_timestamp,
            1, 1,
            max_length
        );

        env_log!("as {}", twap.observations.len());
        assert!(twap.observations.len() == 10, "Mismatch");
        assert!(current_idx == 1, "current index mismatch");

        assert!(twap.observations[1].block_timestamp == last_timestamp, "Mismatch");
        assert!(twap.observations[1].num_of_observations == 12);
    }

    #[test]
    fn simple_binary_search_works() {
        init_blockchain();

        let twap: Twap = get_twap();
        let max_length = 10;

        // current observation timestamp array [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        let mut returned_index = twap.binary_search(
            max_length,
            5,
        );

        assert!(returned_index == 4, "Wrong Index");

        returned_index = twap.binary_search(
            max_length,
            0,
        );

        assert!(returned_index == 0, "Wrong Index");

        returned_index = twap.binary_search(
            max_length,
            10,
        );

        assert!(returned_index == 9, "Wrong Index");
    }

    #[test]
    #[should_panic(expected = "Observation after this timestamp doesn't exist")]
    fn binary_edge_case_works() {
        init_blockchain();

        let twap: Twap = get_twap();
        let max_length = 10;

        // current observation timestamp array [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        twap.binary_search(
            max_length,
            122,
        );
    }

    #[test]
    fn pivoted_binary_search_works() {
        init_blockchain();

        let mut twap: Twap = get_twap();
        let max_length = 10;

        // current array [1, 2, 3, 4, 5, 6, 8, 9, 10]
        // add more value (that should overwrite last updated value)
        let mut current_idx = twap.write(
            13,
            10, 10,
            max_length
        );

        let mut result_index = twap.binary_search(
            max_length,
            11,
        );
        env_log!("SSS {} {}", result_index, current_idx);
        assert!(result_index == 0, "Wrong Index");

        current_idx = twap.write(
            20,
            10, 10,
            max_length
        );
        current_idx = twap.write(
            21,
            10, 10,
            max_length
        );
        // Updated array [13, 20, 21, 4, 5, 6, 7, 8, 9, 10]

        result_index = twap.binary_search(
            max_length,
            3,
        );

        env_log!("RESULT {}", result_index);
        assert!(twap.observations[0].block_timestamp == 13, "First timestamp wrong");
        assert!(twap.observations[1].block_timestamp == 20, "Second timestamp wrong");
        assert!(twap.observations[2].block_timestamp == 21, "Second timestamp wrong");
        assert!(result_index == 3, "Wrong Index");

        result_index = twap.binary_search(
            max_length,
            15,
        );

        assert!(result_index == 1, "Wrong Index");

        result_index = twap.binary_search(
            max_length,
            21,
        );

        assert!(result_index == 2, "Wrong Index");

        result_index = twap.binary_search(
            max_length,
            10,
        );

        assert!(result_index == 9, "Wrong Index");
    }

    #[test]
    fn calculate_mean() {
        init_blockchain();

        let timestamp = env::block_timestamp();
        let mut twap: Twap = Twap::new();
        let mut current_idx = twap.initialize(timestamp, 1, 1);
        let max_length = 10;

        let min_2_timestamp = timestamp + to_nanoseconds(120);

        twap.write(min_2_timestamp, 3, 3, max_length);
        let mut res = twap.calculate_mean(Mean::M_1MIN, twap.observations.len());

        assert_eq!(3, res.0, "Wrong mean - 1");
        assert_eq!(3, res.1, "Wrong mean - 1");

        let min_8_timestamp = timestamp + to_nanoseconds(480);
        let min_10_timestamp = timestamp + to_nanoseconds(600);

        twap.write(min_8_timestamp, 12, 12, max_length);
        twap.write(min_10_timestamp, 10, 10, max_length);
        res = twap.calculate_mean(Mean::M_5MIN, twap.observations.len());

        assert_eq!(11 , res.0, "Wrong mean - 2");
        assert_eq!(11, res.1, "Wrong mean - 1");
    }
}
