use std::convert::TryFrom;

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::AccountId;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount};

use near_sdk_sim::transaction::ExecutionStatus;
use nearswap::{NearSwap, PoolInfo};
use std::collections::HashMap;
use sample_token::ContractContract as SampleToken;
