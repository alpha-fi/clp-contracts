#![allow(unused)]

mod clp_utils;
mod nep21_utils;
use clp_utils::*;
use nep21_utils::*;

use near_sdk_sim::{
    call, deploy, init_simulator, near_crypto::Signer, to_yocto, view, ContractAccount,
    UserAccount, STORAGE_AMOUNT,
};
use nearswap::util::*;
use nearswap::PoolInfo;
use near_sdk::json_types::{U128, U64};
use serde_json::json;
use std::convert::TryInto;

#[test]
pub fn create_pool() {
    let (master_account, clp_contract, token, alice, carol) = deploy_clp();
    println!("NearSwap Contract Deployed");

    let token_id = "token";
    let token_contract = deploy_nep21(&token, token_id.into(), U128(1_000_000 * NDENOM));
    println!("Token 1 deployed");
    
    println!("{} creates a pool", carol.account_id());
    // Uses default gas amount, `near_sdk_sim::DEFAULT_GAS`
    let res = call!(
        carol,
        clp_contract.create_pool(token_id.to_string().try_into().unwrap()),
        deposit = STORAGE_AMOUNT
    );
    println!("{:#?}\n Cost:\n{:#?}", res.status(), res.profile_data());
    assert!(res.is_ok());

    assert_eq!(
        get_pool_info(&clp_contract, &token_id.to_string()),
        PoolInfo {
            ynear: 0.into(),
            reserve: 0.into(),
            total_shares: 0.into()
        },
        "new pool should be empty"
    );
}