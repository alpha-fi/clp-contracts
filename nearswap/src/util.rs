// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_sdk::Gas;
use near_sdk::{env, AccountId, PromiseResult};
use uint::construct_uint;

use crate::constants::*;

/// Near denomination = 1e24. Usage: { amount: 50*E24 }
const NDENOM_ROUNDING: u128 = 500_000_000_000_000_000_000_000;

/// TGas denomination 1 Tera Gas => 1e12 yNEAR
pub const TGAS: Gas = 1_000_000_000_000;

/// Prepaid gas costs. TODO: we need to adjust this value properly.
pub const MAX_GAS: Gas = 200 * TGAS;

// TODO: should we make it customizable?
/// Price per 1 byte of storage from mainnet genesis config. 100e18
pub const STORAGE_BYTE_PRICE: u128 = 100_000_000_000_000_000_000;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct u256(4);
}

/** Ensures that an account `a` is valid and panics if it's not.
`name`: printed name of the account */
#[inline]
pub fn assert_account_is_valid(a: &AccountId) {
    assert!(
        env::is_valid_account_id(a.as_bytes()),
        format!("{} account ID is invalid", a)
    );
}

pub fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    match env::promise_result(0) {
        PromiseResult::Successful(_) => true,
        _ => false,
    }
}

// convert seconds into nanoseconds
pub const fn to_nanoseconds(time: u64) -> u64 {
    return time * 1000_000_000;
}

/// yoctoNEAR to NEAR. Rounds down.
#[inline]
pub fn yton(yocto_amount: u128) -> u128 {
    (yocto_amount + NDENOM_ROUNDING) / NDENOM
}

#[macro_export]
macro_rules! env_log {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        // io::_print(msg);
        println!("{}", msg);
        env::log(msg.as_bytes())
    }}
}
