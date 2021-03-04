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
fn add_liquidity() {
    let (master_account, clp_contract, token, alice, carol) = deploy_clp();
    let token_id = "token";
    let token_contract = deploy_nep21(&token, token_id.into(), U128(1_000_000 * NDENOM));
    // Creates Pool
    call!(
        carol,
        clp_contract.create_pool(token_id.to_string().try_into().unwrap()),
        deposit = STORAGE_AMOUNT
    );
    let near_deposit = 7_000 * NDENOM;
    let token_deposit = 14_000 * NDENOM;
    // Funds Alice
    call!(
        token,
        token_contract.transfer(alice.account_id(), token_deposit.into()),
        deposit = STORAGE_AMOUNT
    );
    println!(
        "{} adds liquidity to {}",
        alice.account_id(), token_id.to_string()
    );
    println!("creating allowance for CLP");
    let res = call!(
        alice,
        token_contract.inc_allowance(NEARSWAP_CONTRACT_ID.to_string(), token_deposit.into()),
        deposit = 2 * NEP21_STORAGE_DEPOSIT
    );
    println!("{:#?}\n Cost:\n{:#?}", res.status(), res.profile_data());
    assert!(res.is_ok());

    let val = view!(token_contract.get_allowance(
        alice.account_id(), NEARSWAP_CONTRACT_ID.to_string())
    );
    let value: U128 = val.unwrap_json();
    assert!(value == U128(token_deposit), "Allowance Error");

    //add_liquidity
    let res1 = call!(
        alice,
        clp_contract.add_liquidity(token_id.to_string(), U128(token_deposit), U128(near_deposit)),
        deposit = near_deposit + NEP21_STORAGE_DEPOSIT
    );
    
    // Verify Liquidity
    let bal = get_nep21_balance(&token_contract, &NEARSWAP_CONTRACT_ID.to_string());
    assert!(bal == U128(token_deposit), "Liquidity Error");

    let after_adding_info = get_pool_info(&clp_contract, &token_id.to_string());
    println!(
        "pool after add liq: {} {:?}",
        &token.account_id(),
        after_adding_info
    );
}