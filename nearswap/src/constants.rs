use near_sdk::{env, Balance};

use crate::util::*;

pub const STORAGE_PRICE_PER_BYTE: Balance = env::STORAGE_PRICE_PER_BYTE;

pub const MAX_ACCOUNT_LENGTH: u128 = 64;
pub const MIN_ACCOUNT_DEPOSIT_LENGTH: u128 = MAX_ACCOUNT_LENGTH + 16 + 4;
