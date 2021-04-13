use near_sdk::{env, Balance};

use crate::util::*;

pub const STORAGE_PRICE_PER_BYTE: Balance = env::STORAGE_PRICE_PER_BYTE;

pub const MAX_ACCOUNT_LENGTH: u128 = 64;
pub const MIN_ACCOUNT_DEPOSIT_LENGTH: u128 = MAX_ACCOUNT_LENGTH + 16 + 4;
pub const MAX_LENGTH: usize = 65535;

pub const T_1MIN: u64 = to_nanoseconds(60);
pub const T_5MIN: u64 = to_nanoseconds(300);
pub const T_1H: u64 = to_nanoseconds(60 * 60);
pub const T_12H: u64 = to_nanoseconds(60 * 60 * 12);