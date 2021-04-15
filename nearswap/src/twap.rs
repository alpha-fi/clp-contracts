use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::collections::{Vector};
use near_sdk::{env};
use std::convert::{TryFrom,TryInto};

use std::fmt;
use crate::*;

pub const T_1MIN: u64 = to_nanoseconds(60);
pub const T_5MIN: u64 = to_nanoseconds(300);
pub const T_1H: u64 = to_nanoseconds(60 * 60);
pub const T_12H: u64 = to_nanoseconds(60 * 60 * 12);

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
    current_idx: u64,
    // bool to check if previous values are overwritten by new one
    // i.e MAX_LENGTH of array is full and we start storing observation from `0` index
    pivoted: bool,
    // maximum length of observation array
    max_length: u64,
    // observation array
    pub observations: Vector<Observation>,

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

    pub fn new(length: u64) -> Self {
        Self {
            current_idx: 0,
            pivoted: false,
            max_length: length,
            observations: Vector::new("twap".as_bytes().to_vec()),
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
    fn initialize(
        &mut self,
        time: u64,
        price1: u128,
        price2: u128
    ) -> u64
    {
        self.observations.push( &Observation {
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
    + `block_timestamp`: The timestamp (in nanoseconds) of the new observation.
    + `price1`: price of first token.
    + `price2`: price of second token.
    */
    fn write(
        &mut self,
        block_timestamp: u64,
        price1: u128,
        price2: u128
    ) -> u64 {
        let mut o = &self.observations.get(self.current_idx).unwrap();
        if(block_timestamp == o.block_timestamp) {
            self.observations.replace(self.current_idx, &Observation::transform(o, block_timestamp, price1, price2));
            return self.current_idx;
        }

        if self.current_idx + 1 >= self.max_length {
            self.pivoted = true;
            self.current_idx = 0;
        } else {
            self.current_idx += 1;
        }
        if self.current_idx < self.observations.len() {
            self.observations.replace(self.current_idx, &Observation::transform(o, block_timestamp, price1, price2));
        } else {
            self.observations.push(&Observation::transform(o, block_timestamp, price1, price2));
        }

        return self.current_idx;
    }

    /**
    Pivoted point binary search: searches array which is
    sorted and rotated from a particular point.
    Similar to rotated array from a certain pivot point.
    Parameters:
    + `block_timestamp`: timestamp in nonoseconds.
    */
    pub fn binary_search(
        &self,
        block_timestamp: u64,
    ) -> u64 {

        // edge case when all values are less than required
        if self.observations.get(self.current_idx).unwrap().block_timestamp
            < block_timestamp {
            panic!("Observation after this timestamp doesn't exist");
        }

        let mut start: u64 = 0;
        let mut end: u64 = self.current_idx + 1;
        let mut mid: u64;

        while start < end {
            mid = (start + end) / 2;
            if block_timestamp <= self.observations.get(mid).unwrap().block_timestamp {
                end = mid;
            } else {
                start = mid + 1;
            }
        }

        if self.pivoted && start == 0 {
            let res = start;
            start = self.current_idx + 1;
            end = self.observations.len();

            while start < end {
                mid = (start + end) / 2;
                if block_timestamp <= self.observations.get(mid).unwrap().block_timestamp {
                    end = mid;
                } else {
                    start = mid + 1;
                }
            }
            if start >= self.observations.len() {
                start = res;
            }
        }

        return start;
    }

    // function which will calculate mean using Mean enum
    pub fn calculate_mean(
        &self,
        time: Mean,
    ) -> (u128, u128) {
        let time_diff: u64 = match time {
            Mean::M_1MIN => T_1MIN, // 1 minute in nanoseconds
            Mean::M_5MIN => T_5MIN, // 5 minute in nanoseconds
            Mean::M_1H => T_1H,
            Mean::M_12H => T_12H,
            _ => 0
        };
        let req_timestamp;
        if self.observations.get(self.current_idx).unwrap().block_timestamp >= time_diff {
            req_timestamp = self.observations.get(self.current_idx).unwrap().block_timestamp - time_diff;
        } else {
            req_timestamp = self.observations.get(self.current_idx).unwrap().block_timestamp;
        }
         
        let left_index = self.binary_search(req_timestamp);
        let current_o = &self.observations.get(self.current_idx).unwrap();
        
        let total_observe;
        let price1cumu;
        let price2cumu;
        if left_index == 0 {
            total_observe = current_o.num_of_observations;
            price1cumu = current_o.price1_cumulative;
            price2cumu = current_o.price2_cumulative;
        } else {
            let prev_o = &self.observations.get(left_index - 1).unwrap();
            total_observe = current_o.num_of_observations
                                    - prev_o.num_of_observations;
            price1cumu = current_o.price1_cumulative
                                    - prev_o.price1_cumulative;
            price2cumu = current_o.price2_cumulative
                                    - prev_o.price2_cumulative;
        }
        let mean1 = price1cumu / total_observe;
        let mean2 = price2cumu / total_observe;
        return (mean1, mean2);
    }

    pub(crate) fn log_observation(&mut self, timestamp: u64, price1: u128, price2: u128) -> u64 {
        // update current index
        if self.observations.len() == 0 {
            self.current_idx = self.initialize(timestamp, price1, price2);
        } else {
            self.current_idx = self.write(
                timestamp,
                price1,
                price2
            );
        }

        self.update_mean();
        return self.current_idx;
    }

    pub fn update_mean(&mut self) {
        self.mean_1min = self.calculate_mean(Mean::M_1MIN);
        self.mean_5min = self.calculate_mean(Mean::M_5MIN);
        self.mean_1h = self.calculate_mean(Mean::M_1H);
        self.mean_12h = self.calculate_mean(Mean::M_12H);
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
    /// returns instance of Observation structure
    pub fn new() -> Self {
        return Self {
            block_timestamp: 1,
            num_of_observations: 0,
            price1_cumulative: 0,
            price2_cumulative: 0,
        }
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, BlockHeight, MockedBlockchain};

    fn init_blockchain() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
    }

    // returns twap with observation vector with timestamp [1, 2, 3, 4, 5, 6, 7, 9, 10]
    fn get_twap(timestamp: u64) -> Twap {

        let max_length = 10;
        let mut twap: Twap = Twap::new(max_length);
        let mut current_idx;

        // fill all places
        for i in 1..11 {
            let timestamp = i;
            current_idx = twap.log_observation(
                timestamp,
                1,
                1,
            );
        }

        return twap;
    }

    #[test]
    fn initialize_works() {
        init_blockchain();

        let mut twap: Twap = Twap::new(10);
        let current_idx = twap.log_observation(1, 1, 1);

        assert!(twap.observations.len() == 1, "Mismatch");
        assert!(twap.observations.get(0).unwrap().price1_cumulative == 1, "Mismatch");
        assert!(twap.observations.get(0).unwrap().price2_cumulative == 1, "Mismatch");
    }

    #[test]
    fn write_works() {
        init_blockchain();

        let mut twap: Twap = Twap::new(10);
        let mut current_idx = twap.log_observation(1, 1, 1);

        let timestamp = 12;
        current_idx = twap.log_observation(
            timestamp,
            100, 2
        );

        assert!(twap.observations.len() == 2, "Length Mismatch");
        assert!(twap.observations.get(1).unwrap().num_of_observations == 2);
        assert!(twap.observations.get(1).unwrap().price1_cumulative == 101, "price 1 Mismatch");
        assert!(twap.observations.get(1).unwrap().price2_cumulative == 3, "price 2 Mismatch");

        // write on same timestamp
        current_idx = twap.log_observation(
            timestamp,
            10, 10
        );

        // verify number of observations is 3 but observation length should be 2
        assert!(twap.observations.len() == 2, "length 2 Mismatch");

        assert!(twap.observations.get(0).unwrap().num_of_observations == 1);
        assert!(twap.observations.get(1).unwrap().num_of_observations == 3);

        // verify cumulative prices
        assert!(twap.observations.get(1).unwrap().price1_cumulative == 111, "updated price 1 Mismatch");
        assert!(twap.observations.get(1).unwrap().price2_cumulative == 13, "updated price 2 Mismatch");
    }

    #[test]
    fn overwrite_works() {
        init_blockchain();

        let mut twap: Twap = Twap::new(10);
        let mut current_idx;
        let max_length = 10;
        // fill all places
        for i in 1..11 {
            let timestamp = i + 1;
            current_idx = twap.log_observation(
                timestamp,
                1, 1
            );
        }

        assert!(twap.observations.len() == 10, "Mismatch length");

        // next observation should be written on 0th Index
        let mut last_timestamp = 10;
        current_idx = twap.log_observation(
            last_timestamp,
            1, 1
        );

        println!("sadsa {} ", twap.observations.get(0).unwrap().block_timestamp);
        assert!(twap.observations.len() == 10, "Mismatch, length 2");
        assert!(twap.observations.get(0).unwrap().block_timestamp == last_timestamp, "Mismatch overwrite");
        assert!(twap.observations.get(0).unwrap().num_of_observations == 11);

        // next observation should be written on 1st Index
        last_timestamp = 11;
        current_idx = twap.log_observation(
            last_timestamp,
            1, 1
        );

        println!("as {}", twap.observations.len());
        assert!(twap.observations.len() == 10, "Mismatch");
        assert!(current_idx == 1, "current index mismatch");

        assert!(twap.observations.get(1).unwrap().block_timestamp == last_timestamp, "Mismatch");
        assert!(twap.observations.get(1).unwrap().num_of_observations == 12);
    }

    #[test]
    fn simple_binary_search_works() {
        init_blockchain();

        let twap: Twap = get_twap(1);
        let max_length = 10;

        // current observation timestamp array [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        let mut returned_index = twap.binary_search(5);
        assert!(returned_index == 4, "Wrong Index");

        returned_index = twap.binary_search(0);
        assert!(returned_index == 0, "Wrong Index");

        returned_index = twap.binary_search(10);
        assert!(returned_index == 9, "Wrong Index");
    }

    #[test]
    #[should_panic(expected = "Observation after this timestamp doesn't exist")]
    fn binary_edge_case_works() {
        init_blockchain();

        let twap: Twap = get_twap(1);
        let max_length = 10;

        // current observation timestamp array [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        twap.binary_search(max_length + 2);
    }

    #[test]
    fn pivoted_binary_search_works() {
        init_blockchain();

        let mut twap: Twap = get_twap(1);
        let max_length = 10;

        // current array [1, 2, 3, 4, 5, 6, 8, 9, 10]
        // add more value (that should overwrite last updated value)
        let mut current_idx = twap.log_observation(
            13,
            10, 10
        );

        let mut result_index = twap.binary_search(
            11,
        );
        println!("SSS {} {}", result_index, current_idx);
        assert!(result_index == 0, "Wrong Index");

        current_idx = twap.log_observation(
            20,
            10, 10
        );
        current_idx = twap.log_observation(
            21,
            10, 10
        );
        // Updated array [13, 20, 21, 4, 5, 6, 7, 8, 9, 10]

        result_index = twap.binary_search(
            3,
        );

        println!("RESULT {}", result_index);
        assert!(twap.observations.get(0).unwrap().block_timestamp == 13, "First timestamp wrong");
        assert!(twap.observations.get(1).unwrap().block_timestamp == 20, "Second timestamp wrong");
        assert!(twap.observations.get(2).unwrap().block_timestamp == 21, "Second timestamp wrong");
        assert!(result_index == 3, "Wrong Index");

        result_index = twap.binary_search(
            15,
        );

        assert!(result_index == 1, "Wrong Index");

        result_index = twap.binary_search(
            21,
        );

        assert!(result_index == 2, "Wrong Index");

        result_index = twap.binary_search(
            10,
        );

        assert!(result_index == 9, "Wrong Index");
    }

    #[test]
    fn calculate_mean() {
        init_blockchain();

        let timestamp = 1;
        let max_length = 10;
        let mut twap: Twap = Twap::new(max_length);
        let mut current_idx = twap.log_observation(timestamp, 1, 1);

        let min_2_timestamp = timestamp + to_nanoseconds(120);

        twap.log_observation(min_2_timestamp, 3, 3);
        let mut res = twap.calculate_mean(Mean::M_1MIN);

        assert_eq!(3, res.0, "Wrong mean - 1");
        assert_eq!(3, res.1, "Wrong mean - 1");

        let min_8_timestamp = timestamp + to_nanoseconds(480);
        let min_10_timestamp = timestamp + to_nanoseconds(600);

        twap.log_observation(min_8_timestamp, 12, 12);
        twap.log_observation(min_10_timestamp, 10, 10);
        res = twap.calculate_mean(Mean::M_5MIN);

        assert_eq!(11 , res.0, "Wrong mean - 2");
        assert_eq!(11, res.1, "Wrong mean - 1");
    }

    // binary search edge cases
    #[test]
    fn calculate_mean_edge_cases() {
        init_blockchain();

        let timestamp = 1;
        let max_length = 10;
        let mut twap: Twap = Twap::new(max_length);
        let mut current_idx = twap.log_observation(timestamp, 1, 1);

        let min_2_timestamp = timestamp + to_nanoseconds(120);

        twap.log_observation(min_2_timestamp, 3, 3);
        // calculate mean for last 5 mins, though array starts from 1 min
        let mut res = twap.calculate_mean(Mean::M_5MIN);

        assert_eq!(3, res.0, "Wrong mean - 1");
        assert_eq!(3, res.1, "Wrong mean - 1");

        // calculate mean for last 12 hours mins, though array starts from 1 min
        let mut res = twap.calculate_mean(Mean::M_12H);

        assert_eq!(3, res.0, "Wrong mean - 1");
        assert_eq!(3, res.1, "Wrong mean - 1");

        let min_8_timestamp = timestamp + to_nanoseconds(480);
        let min_10_timestamp = timestamp + to_nanoseconds(600);

        twap.log_observation(min_8_timestamp, 12, 12);
        twap.log_observation(min_10_timestamp, 10, 10);

        for i in 2..9 {
            twap.log_observation(min_10_timestamp + to_nanoseconds(60 * (i + 10)), 5, 5);
        }
        // array
        // 18, 3, 8, 10, 12, 13, 14, 15, 16, 17
        res = twap.calculate_mean(Mean::M_1MIN);

        assert_eq!(5 , res.0, "Wrong mean - 2");
        assert_eq!(5, res.1, "Wrong mean - 1");
    }
}
