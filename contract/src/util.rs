use near_sdk::{env, AccountId, PromiseResult};
use uint::construct_uint;

/// Near denomination = 1e24. Usage: { amount: 50*E24 }
pub const NDENOM: u128 = 1_000_000_000_000_000_000_000_000;

/// Prepaid gas costs. TODO: we need to adjust this value properly.
pub const MAX_GAS: u64 = 200_000_000_000_000;

/// nep21 may require NEAR deposit for storage to create a new balance
pub const NEP21_STORAGE_DEPOSIT: u128 = 4 * NDENOM / 100; //0.04 NEAR

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct u256(4);
}

/** Ensures that an account `a` is valid and panics if it's not.
`name`: printed name of the account */
pub fn assert_account(a: &AccountId, name: &str) {
    assert!(
        env::is_valid_account_id(a.as_bytes()),
        format!("{} account ID is invalid", name)
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

/// yoctoNEAR to NEAR. Rounds to nearest.
pub fn yton(yocto_amount: u128) -> u128 {
    (yocto_amount + (5 * 10u128.pow(23))) / 10u128.pow(24)
}

#[macro_export]
macro_rules! env_log {
    ($($arg:tt)*) => {{
        let res = format!($($arg)*);
        env::log(res.as_bytes())
    }}
}
