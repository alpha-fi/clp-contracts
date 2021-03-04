#![allow(unused)]

mod test_utils;
use test_utils::*;

use near_sdk_sim::{
  STORAGE_AMOUNT,
  to_yocto,
  call,
  view
};
use near_sdk::json_types::{U128, U64};


#[test]
fn test_nep21_transer() {
    println!(
        "Note that we can use println! instead of env::log in simulation tests. To debug add '-- --nocapture' after 'cargo test': "
    );
    let (master_account, clp_contract, token, alice, carol) = deploy_clp();
    let contract = deploy_nep21(&token, U128(to_yocto("1000000")));
    println!("tokens deployed");

    let transfer_amount = to_yocto("100");
    let initial_balance = to_yocto("0");
    // send some to Alice
    let res = call!(
        token,
        contract.transfer(alice.account_id(), transfer_amount.into()),
        deposit = STORAGE_AMOUNT
    );
    println!("{:#?}\n Cost:\n{:#?}", res.status(), res.profile_data());
    assert!(res.is_ok());

    let val = view!(contract.get_balance(alice.account_id()));
    let value: U128 = val.unwrap_json();
    assert_eq!(initial_balance + transfer_amount, value.0);
}