// Errors
// E1: pool already exists
// E2: all token arguments must be positive.
// E3: required amount of tokens to transfer is bigger then specified max.
// E4: computed amount of shares to receive is smaller then the minimum required by the user.
// E5: can't withdraw more shares then currently owned
// E6: computed amount of near or reserve tokens is smaller than user required minimums for shares redeemption.
// E7: computed amount of buying tokens is smaller than user required minimum.
// E8: computed amount of selling tokens is bigger than user required maximum.
// E9: assets (tokens) must be different in token to token swap.
// E10: Pool is empty and can't make a swap.
// E22: Only owner can call this function

pub const ERR02_POSITIVE_ARGS: &str = "E2: balance arguments must be >0";

pub const ERR11_NOT_ENOUGH_SHARES: &str = "E11: Insufficient amount of shares balance";
pub const ERR12_NOT_ENOUGH_NEAR: &str = "E12: Insufficient amount of NEAR attached";
pub const ERR13_NOT_ENOUGH_TOKENS_DEPOSITED: &str = "E13: Insufficient amount of tokens in deposit";
pub const ERR14_NOT_ENOUGH_NEAR_DEPOSITED: &str = "E14: Insufficient amount of NEAR in deposit";
pub const ERR15_NOT_ENOUGH_NEAR_DEPOSITED: &str = "E14: Insufficient amount of NEAR in deposit";

pub const ERR20_ACC_NOT_REGISTERED: &str = "E20: Account not registered";
pub const ERR21_ACC_STORAGE_TOO_LOW: &str =
    "E21: Not enough NEAR to cover storage. Deposit more NEAR";
pub const ERR22_ACC_ALREADY_REGISTERED: &str = "E22: Account already registered";
