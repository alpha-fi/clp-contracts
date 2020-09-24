use near_sdk::{env, AccountId};

/// Prepaid gas costs. TODO: we need to adjust this value properly.
pub const SINGLE_CALL_GAS: u64 = 200_000_000_000_000; // this value equals max gas/tx
pub const NEP21_STORAGE_DEPOSIT: u128 = 1_000_000_000_000_000_000_000_000; // 1 NEAR

/** Ensures that an account `a` is valid and panics if it's not.
`name`: printed name of the account */
pub fn assert_account(a: &AccountId, name: &str) {
    assert!(
        env::is_valid_account_id(a.as_bytes()),
        format!("{} account ID is invalid", name)
    );
}

/// yoctoNEAR to NEAR
pub fn yton(near_amount: u128) -> u128 {
    return near_amount / 10u128.pow(24);
}
