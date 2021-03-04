#![allow(unused)]

use near_sdk_sim::{
  STORAGE_AMOUNT,
  to_yocto,
  call,
  view,
  init_simulator,
  UserAccount,
  ContractAccount,
  deploy
};
use near_sdk::json_types::{U128, U64};

use nep21_mintable::FungibleTokenContract;

/// Load in contract bytes
near_sdk_sim::lazy_static! {
    static ref FUNGIBLE_TOKEN_BYTES: &'static [u8] = include_bytes!("../../../target/wasm32-unknown-unknown/release/nep21_mintable.wasm").as_ref();
}
// Deploy NEP-21 Contract
pub fn deploy_nep21(
    total_supply: U128
) -> (UserAccount, ContractAccount<FungibleTokenContract>, UserAccount) {
    let master_account = init_simulator(None);
    println!("deploy_nep21");
    // uses default values for deposit and gas
    let contract_user = deploy!(
        // Contract Proxy
        contract: FungibleTokenContract,
        // Contract account id
        contract_id: "token",
        // Bytes of contract
        bytes: &FUNGIBLE_TOKEN_BYTES,
        // User deploying the contract,
        signer_account: master_account,
        // init method
        init_method: new(master_account.account_id(), total_supply, 24)
    );
    let alice = master_account.create_user("alice".to_string(), to_yocto("1000000"));
    (master_account, contract_user, alice)
}

#[test]
fn test_nep21_transer() {
    println!(
        "Note that we can use println! instead of env::log in simulation tests. To debug add '-- --nocapture' after 'cargo test': "
    );
    let (master, contract, alice) = deploy_nep21(U128(to_yocto("1000000")));
    println!("tokens deployed");

    let transfer_amount = to_yocto("100");
    let initial_balance = to_yocto("0");
    // send some to Alice
    let res = call!(
        master,
        contract.transfer(alice.account_id(), transfer_amount.into()),
        deposit = STORAGE_AMOUNT
    );
    println!("{:#?}\n Cost:\n{:#?}", res.status(), res.profile_data());
    assert!(res.is_ok());

    let val = view!(contract.get_balance(alice.account_id()));
    let value: U128 = val.unwrap_json();
    assert_eq!(initial_balance + transfer_amount, value.0);
}