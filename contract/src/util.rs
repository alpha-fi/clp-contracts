use near_sdk::{env, AccountId};

/// Prepaid gas costs. TODO: we need to adjust this value properly.
pub const MAX_GAS: u64 = 300_000_000_000_000;

// E24: 1e24 - the number you need to mulitply by to convert an amount in NEARS to YOCTO nears
// usage: { amount: 50*E24 } 
pub const E24 : u128 = 1_000_000_000_000_000_000_000_000;

// if the nep21 requires account creatim, the contract retains some near for storage backing
pub const NEP21_STORAGE_DEPOSIT: u128 = 4*E24/100; //0.04 NEAR


/** Ensures that an account `a` is valid and panics if it's not.
`name`: printed name of the account */
pub fn assert_account(a: &AccountId, name: &str) {
    assert!(
        env::is_valid_account_id(a.as_bytes()),
        format!("{} account ID is invalid", name)
    );
}

/// yoctoNEAR to NEAR. Rounds to nearest.
pub fn yton(yocto_amount: u128) -> u128 {
    (yocto_amount + (5 * 10u128.pow(23))) / 10u128.pow(24)
}

