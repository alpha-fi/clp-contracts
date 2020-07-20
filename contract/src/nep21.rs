use near_sdk::ext_contract;
use near_sdk::json_types::U128;

#[ext_contract(ext_nep21)]
pub trait NEP21 {
    // #[payable]
    fn transfer(&mut self, dest: AccountId, amount: U128);

    // #[payable]
    fn transfer_from(&mut self, from: AccountId, dest: AccountId, amount: U128);
}
