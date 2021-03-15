use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{ext_contract, Gas};

/// Amount of gas for fungible token transfers.
pub const GAS_FOR_FT_TRANSFER: Gas = 10_000_000_000_000;

#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);

    fn ft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;

    /// Returns the total supply of the token in a decimal string representation.
    fn ft_total_supply(&self) -> U128;

    /// Returns the balance of the account. If the account doesn't exist must returns `"0"`.
    fn ft_balance_of(&self, account_id: ValidAccountId) -> U128;
}
