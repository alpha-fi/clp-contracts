// pub const STORAGE_PRICE_PER_BYTE: Balance = env::STORAGE_PRICE_PER_BYTE;

/// Minimum Account Storage used for account registration.
/// 64 (AccountID bytes) + 2*8 (int32) + byte
pub const INIT_ACCOUNT_STORAGE: u64 = 64 + 16 + 4;

/// 1 NEAR in yocto = 1e24
pub const NDENOM: u128 = 1_000_000_000_000_000_000_000_000;

pub const NEP21_STORAGE_DEPOSIT: u128 = 4 * NDENOM / 100;
