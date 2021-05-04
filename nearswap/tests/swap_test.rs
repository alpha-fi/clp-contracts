// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

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
use nearswap::{NearSwapContract, PoolInfo};
use nep21_mintable::FungibleTokenContract;
use near_primitives::types::{AccountId, Balance};
use near_sdk::json_types::{U128, U64};
use serde_json::json;
use std::convert::TryInto;

#[test]
fn test_swap() {
    let (master_account, clp_contract, token, alice, carol) = deploy_clp();
    println!("NearSwap Contract Deployed");

    let token_id_1 = "nep_21_1";
    let token_contract = deploy_nep21(&token, token_id_1.into(), U128(1_000_000 * NDENOM));
    println!("Token 1 deployed");
    
    let near_deposit = 3_000 * NDENOM;
    let token_deposit = 30_000 * NDENOM;

    create_pool_add_liquidity(
        &clp_contract,
        &token_contract,
        &alice,
        &token,
        token_id_1.into(),
        near_deposit,
        token_deposit,
    );

    // get pool state before swap
    let pooli_before = get_pool_info(&clp_contract, &token_id_1.to_string());
    assert_eq!(
        pooli_before,
        PoolInfo {
            ynear: (near_deposit).into(),
            reserve: (token_deposit).into(),
            total_shares: (near_deposit).into()
        },
        "new pool balance should be from first deposit"
    );
    println!("pool_info:{}", pooli_before);

    println!("Sending nep21 to Carol");
    call!(
        token,
        token_contract.transfer(carol.account_id(), U128(to_yocto("19000"))),
        deposit = NDENOM
    );

    let carol_t_balance_pre =
        show_nep21_bal(&token_contract, &carol.account_id());

    println!("carol swaps some near for tokens");
    let carol_deposit_yoctos: u128 = to_yocto("10");
    let min_token_expected: u128 = to_yocto("98"); //1-10 relation near/token
    let res = call!(
        carol,
        clp_contract.swap_near_to_token_exact_in(token_id_1.to_string(), U128(min_token_expected)),
        deposit = carol_deposit_yoctos.into()
    );
    println!("{:#?}\n Cost:\n{:#?}", res.status(), res.profile_data());
    let log = res.logs();

    assert!(res.is_ok());
    println!("pool_info:{}", get_pool_info(&clp_contract, &token_id_1.to_string()));
    println!("let's see how many token carol has after the swap");
    let carol_t_balance_post =
        show_nep21_bal(&token_contract, &carol.account_id());

    let carol_received = carol_t_balance_post - carol_t_balance_pre;
    assert!(
        carol_received >= min_token_expected,
        "carol should have received at least min_token_expected"
    );

    let pooli_after = get_pool_info(&clp_contract, &token_id_1.to_string());
    assert_eq!(
        pooli_after,
        PoolInfo {
            ynear: (u128::from(pooli_before.ynear) + carol_deposit_yoctos).into(),
            reserve: (u128::from(pooli_before.reserve) - carol_received).into(),
            total_shares: pooli_before.total_shares,
        },
        "new pool balance after swap"
    );

    //--------------
    let bob = master_account.create_user("bob".to_string(), to_yocto("1000000"));
    let token2 = master_account.create_user("token2".to_string(), to_yocto("1000000"));
    let token_id_2 = "nep_21_2";
    let token_contract_2 = deploy_nep21(&token2, token_id_2.into(), U128(1_000_000 * NDENOM));
    println!("Token deployed");
    // bob creates another pool with nep21_2
    create_pool_add_liquidity(
        &clp_contract,
        &token_contract_2,
        &bob,
        &token2,
        token_id_2.into(),
        1000 * NDENOM,
        500 * NDENOM,
    );

    // carol tries to swap nep1 she owns with nep2 from bob's pool
    // Token to Token Swap
    println!(">> nep21-1 {:?}", pooli_after);
    println!(">> nep21-2 {:?}", get_pool_info(&clp_contract, &token_id_2.to_string()));

    // liquidity1: 3000 NEAR -- 30_000 nep21_1  (1:10)  -- originally
    // liquidity1: 3010 NEAR -- 29_900 nep21_1  (1:~10)  -- after the swap
    // liquidity2: 1000 NEAR --    500 nep21_2  (2:1)

    // token1:token2 = 20 : 1
    // buying 3 nep21_2 requires 60 nep21_1 + fees
    let buy_amount = (3 * NDENOM);
    let carol_allowance = 61 * NDENOM;
    call!(
        carol,
        token_contract_2.inc_allowance(NEARSWAP_CONTRACT_ID.to_string(), carol_allowance.into()),
        deposit = NDENOM
    );

    let value = view!(clp_contract.price_token_to_token_out(
        token_id_1.to_string(),
        token_id_2.to_string(),
        U128(buy_amount)
    ));
    let price: U128 = value.unwrap_json();
    println!(
        ">> price for {} {} is {:?} {}; allowance={}",
        buy_amount, &token_id_1.to_string(), price, &token_id_2.to_string(), carol_allowance
    );
    let carol_t_balance_pre_2 =
        show_nep21_bal(&token_contract_2, &carol.account_id());

    call!(
        carol,
        clp_contract.swap_tokens_exact_out(
            token_id_1.to_string(), token_id_2.to_string(), U128(buy_amount), U128(carol_allowance)
        ),
        deposit = STORAGE_AMOUNT
    );
    println!("{} {:?}", &token_id_1.to_string(), get_pool_info(&clp_contract, &token_id_1.to_string()));
    println!("{} {:?}", &token_id_2.to_string(), get_pool_info(&clp_contract, &token_id_2.to_string()));

    let carol_t_balance_post_2 =
        show_nep21_bal(&token_contract_2, &carol.account_id());

    let carol_received = carol_t_balance_post_2 - carol_t_balance_pre_2;
    assert!(
        carol_received >= buy_amount,
        "carol should have received at least min_token_expected"
    );
}
