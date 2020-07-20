use near_sdk::{env, AccountId};

/** Ensures that an account `a` is valid and panics if it's not.
`name`: printed name of the account */
pub fn assert_account(a: &AccountId, name: &str) {
    assert!(
        env::is_valid_account_id(a.as_bytes()),
        format!("{} account ID is invalid", name)
    );
}
